use leptos::prelude::*;
use plinth_shared::{SiteConfig, SiteContent};

#[cfg(all(feature = "csr", not(feature = "ssr")))]
pub(crate) fn encode_segment(value: &str) -> String {
    use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};

    utf8_percent_encode(value, NON_ALPHANUMERIC).to_string()
}

#[cfg(all(feature = "csr", target_arch = "wasm32"))]
fn api_url(path: &str) -> String {
    let base = option_env!("PLINTH_CSR_API_BASE")
        .unwrap_or("")
        .trim_end_matches('/');
    if base.is_empty() {
        path.to_string()
    } else {
        format!("{base}{path}")
    }
}

#[cfg(all(feature = "csr", not(feature = "ssr"), target_arch = "wasm32"))]
async fn fetch_json_inner<T>(path: String) -> Result<T, ServerFnError>
where
    T: serde::de::DeserializeOwned,
{
    let url = api_url(&path);
    let response = gloo_net::http::Request::get(&url)
        .send()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    if !response.ok() {
        return Err(ServerFnError::new(format!(
            "GET {url} returned HTTP {}",
            response.status()
        )));
    }

    response
        .json()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[cfg(all(feature = "csr", not(feature = "ssr"), target_arch = "wasm32"))]
pub(crate) async fn fetch_json<T>(path: &str) -> Result<T, ServerFnError>
where
    T: serde::de::DeserializeOwned,
{
    send_wrapper::SendWrapper::new(fetch_json_inner(path.to_string())).await
}

#[cfg(all(feature = "csr", not(feature = "ssr"), not(target_arch = "wasm32")))]
pub(crate) async fn fetch_json<T>(path: &str) -> Result<T, ServerFnError>
where
    T: serde::de::DeserializeOwned,
{
    let _ = path;
    Err(ServerFnError::new(
        "CSR REST fetches are only available in wasm32 builds",
    ))
}

// ── Core server functions (always present) ──────────────────────────────────

/// Fetch the [`SiteConfig`] from the server context (SSR) or from
/// `GET /api/config` (CSR).
#[cfg(any(not(feature = "csr"), feature = "ssr"))]
#[server(GetSiteConfig, "/api")]
pub async fn get_site_config() -> Result<SiteConfig, ServerFnError> {
    Ok(expect_context::<SiteConfig>())
}

/// CSR fallback — fetches `SiteConfig` from `GET /api/config`.
#[cfg(all(feature = "csr", not(feature = "ssr")))]
pub async fn get_site_config() -> Result<SiteConfig, ServerFnError> {
    fetch_json("/api/config").await
}

/// Fetch a [`SiteContent`] value by its string `key` (e.g. `"about"`,
/// `"support"`, `"home-intro"`). SSR queries the database directly; CSR
/// calls `GET /api/content/{key}`.
#[cfg(any(not(feature = "csr"), feature = "ssr"))]
#[server(GetSiteContentFn, "/api")]
pub async fn get_site_content(key: String) -> Result<Option<SiteContent>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let db = expect_context::<sqlx::PgPool>();
        query_site_content(&db, key)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))
    }
    #[cfg(not(feature = "ssr"))]
    {
        let _ = key;
        unreachable!("server fn body only runs under ssr")
    }
}

/// CSR fallback — fetches site content from `GET /api/content/{key}`.
#[cfg(all(feature = "csr", not(feature = "ssr")))]
pub async fn get_site_content(key: String) -> Result<Option<SiteContent>, ServerFnError> {
    fetch_json(&format!("/api/content/{}", encode_segment(&key))).await
}

// ── SSR internal helpers (shared across brick modules) ──────────────────────

#[cfg(feature = "ssr")]
#[allow(dead_code)]
pub(crate) fn postgres_id(table: &str, value: i64) -> Option<String> {
    Some(format!("{table}:{value}"))
}

#[cfg(feature = "ssr")]
#[allow(dead_code)]
pub(crate) fn decode_error(message: impl Into<String>) -> sqlx::Error {
    sqlx::Error::Decode(Box::new(std::io::Error::new(
        std::io::ErrorKind::InvalidData,
        message.into(),
    )))
}

#[cfg(feature = "ssr")]
#[allow(dead_code)]
pub(crate) fn as_u32(value: i32, column: &str) -> Result<u32, sqlx::Error> {
    value
        .try_into()
        .map_err(|_| decode_error(format!("{column} contained negative value {value}")))
}

// ── Core row/query helpers ──────────────────────────────────────────────────

#[cfg(feature = "ssr")]
#[allow(dead_code)]
fn row_site_content(row: sqlx::postgres::PgRow) -> Result<SiteContent, sqlx::Error> {
    use sqlx::Row;

    Ok(SiteContent {
        id: postgres_id("site_content", row.try_get("id")?),
        key: row.try_get("key")?,
        title: row.try_get("title")?,
        content: row.try_get("content")?,
        html_content: row.try_get("html_content")?,
        updated_at: row.try_get("updated_at")?,
    })
}

#[cfg(feature = "ssr")]
#[allow(dead_code)]
async fn query_site_content(
    db: &sqlx::PgPool,
    key: String,
) -> Result<Option<SiteContent>, sqlx::Error> {
    let row = sqlx::query("SELECT * FROM site_content WHERE key = $1 LIMIT 1")
        .bind(key)
        .fetch_optional(db)
        .await?;

    row.map(row_site_content).transpose()
}
