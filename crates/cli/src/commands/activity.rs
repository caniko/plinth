use anyhow::{Context, Result};

use crate::api_client::ApiClient;
use crate::ui;
use plinth_forge::ActivityRef;
use plinth_shared::{
    ActivityKind, FetchedActivity, Forge, PublishActivityRequest, validate_activity_fields,
};

/// `plinth activity add` — fetch a PR/issue from the forge, embed it locally, publish it.
#[allow(clippy::too_many_arguments)]
pub async fn add(
    forge: Forge,
    repo: &str,
    pr: Option<u32>,
    issue: Option<u32>,
    impact: u8,
    featured: bool,
    api_client: &ApiClient,
) -> Result<()> {
    if !(1..=10).contains(&impact) {
        anyhow::bail!("--impact must be between 1 and 10 (got {impact})");
    }
    let (owner, name) = split_repo(repo)?;
    let (kind, number) = match (pr, issue) {
        (Some(n), None) => (ActivityKind::PullRequest, n),
        (None, Some(n)) => (ActivityKind::Issue, n),
        (Some(_), Some(_)) => anyhow::bail!("pass exactly one of --pr or --issue, not both"),
        (None, None) => anyhow::bail!("pass exactly one of --pr or --issue"),
    };
    if number == 0 {
        anyhow::bail!("PR/issue number must be greater than 0");
    }

    let sp = ui::spinner(&format!("Fetching {repo} #{number} from {forge:?}..."));
    let fetched = fetch(forge, &owner, &name, kind, number)
        .await
        .context("Failed to fetch activity from forge")?;
    sp.finish_and_clear();

    let embed_text = match &fetched.body {
        Some(body) if !body.trim().is_empty() => format!("{}\n\n{}", fetched.title, body),
        _ => fetched.title.clone(),
    };
    let sp = ui::spinner("Generating embedding...");
    let embedding = generate_embedding(&embed_text).await?;
    sp.finish_and_clear();
    ui::status("Embedded", &format!("{} dimensions", embedding.len()));

    let request = build_request(
        forge, &owner, &name, kind, number, impact, featured, embedding, &fetched,
    );
    request
        .validate()
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;

    let sp = ui::spinner("Publishing activity...");
    let resp = api_client.publish_activity(request).await?;
    sp.finish_and_clear();
    if let Some(id) = resp.id {
        ui::detail(&format!("id={id}"));
    }
    ui::success(&format!("Activity published: {}", resp.url));
    Ok(())
}

/// Pure builder, factored out so it is unit-testable without network or the runtime.
#[allow(clippy::too_many_arguments)]
pub(crate) fn build_request(
    forge: Forge,
    owner: &str,
    name: &str,
    kind: ActivityKind,
    number: u32,
    impact: u8,
    featured: bool,
    embedding: Vec<f32>,
    fetched: &FetchedActivity,
) -> PublishActivityRequest {
    PublishActivityRequest {
        forge,
        repo_owner: owner.to_string(),
        repo_name: name.to_string(),
        kind,
        number: number as i32,
        url: fetched.url.clone(),
        title: fetched.title.clone(),
        body: fetched.body.clone(),
        state: fetched.state,
        created_at: fetched.created_at,
        closed_at: fetched.closed_at,
        merged_at: fetched.merged_at,
        impact: impact as i16,
        additions: fetched.additions,
        deletions: fetched.deletions,
        comments_count: fetched.comments_count,
        labels: fetched.labels.clone(),
        repo_stars: fetched.repo_stars,
        embedding: Some(embedding),
        featured,
        published: true,
        content_hash: None,
    }
}

/// `plinth activity remove <id>` — DELETE by numeric id.
pub async fn remove(id: i64, api_client: &ApiClient) -> Result<()> {
    let sp = ui::spinner(&format!("Removing activity {id}..."));
    api_client.delete_activity(id).await?;
    sp.finish_and_clear();
    ui::success(&format!("Activity removed: {id}"));
    Ok(())
}

/// `plinth activity update <id> [--impact N] [--featured B]` — PATCH by numeric id.
pub async fn update(
    id: i64,
    impact: Option<u8>,
    featured: Option<bool>,
    api_client: &ApiClient,
) -> Result<()> {
    if impact.is_none() && featured.is_none() {
        anyhow::bail!("nothing to update: pass --impact and/or --featured");
    }
    if let Some(i) = impact {
        validate_activity_fields(i as i16, "owner", "name", 1)
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;
    }

    let sp = ui::spinner(&format!("Updating activity {id}..."));
    api_client
        .patch_activity(id, impact.map(i16::from), featured)
        .await?;
    sp.finish_and_clear();
    ui::success(&format!("Activity updated: {id}"));
    Ok(())
}

/// `plinth activity list` — GET.
pub async fn list(api_client: &ApiClient) -> Result<()> {
    let items = api_client.list_activities().await?;
    if items.is_empty() {
        ui::status("Info", "No activities found.");
        return Ok(());
    }
    ui::status("Found", &format!("{} activity item(s)", items.len()));
    println!();
    for item in items {
        println!(
            "  {} {}/{} #{}  impact={}  score={:.3}",
            ui::bold_style().apply_to(&item.title),
            item.repo_owner,
            item.repo_name,
            item.number,
            item.impact,
            item.score,
        );
    }
    Ok(())
}

/// "owner/name" -> (owner, name); both must be non-empty.
fn split_repo(repo: &str) -> Result<(String, String)> {
    let mut parts = repo.splitn(2, '/');
    let owner = parts.next().unwrap_or("").trim();
    let name = parts.next().unwrap_or("").trim();
    if owner.is_empty() || name.is_empty() {
        anyhow::bail!("--repo must be in owner/name form (got '{repo}')");
    }
    Ok((owner.to_string(), name.to_string()))
}

/// Build the right plinth-forge client by `--forge`, then call its single `fetch` entrypoint.
async fn fetch(
    forge: Forge,
    owner: &str,
    name: &str,
    kind: ActivityKind,
    number: u32,
) -> Result<FetchedActivity> {
    use plinth_forge::{CodebergClient, ForgeClient, GitHubClient};
    let client: Box<dyn ForgeClient> = match forge {
        Forge::GitHub => Box::new(GitHubClient::new(std::env::var("GITHUB_TOKEN").ok())?),
        Forge::Codeberg => Box::new(CodebergClient::new(std::env::var("CODEBERG_TOKEN").ok())?),
    };
    let r = ActivityRef {
        forge,
        owner: owner.to_string(),
        repo: name.to_string(),
        kind,
        number: number as i32,
    };
    let fetched = client.fetch(&r).await?;
    Ok(fetched)
}

/// Generate a 384-dim embedding with fastembed.
async fn generate_embedding(content: &str) -> Result<Vec<f32>> {
    use fastembed::{EmbeddingModel, TextEmbedding};
    let mut end = content.len().min(5000);
    while end > 0 && !content.is_char_boundary(end) {
        end -= 1;
    }
    let truncated = content[..end].to_string();
    tokio::task::spawn_blocking(move || -> Result<Vec<f32>> {
        let mut init_options = fastembed::TextInitOptions::default();
        init_options.model_name = EmbeddingModel::AllMiniLML6V2;
        init_options.show_download_progress = false;
        let mut model = TextEmbedding::try_new(init_options)?;
        let embeddings = model.embed(vec![truncated], None)?;
        embeddings
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("Failed to generate embedding"))
    })
    .await?
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    use plinth_shared::ActivityState;

    fn sample_fetched() -> FetchedActivity {
        FetchedActivity {
            forge: Forge::GitHub,
            repo_owner: "cli".to_string(),
            repo_name: "cli".to_string(),
            kind: ActivityKind::PullRequest,
            number: 9000,
            url: "https://github.com/cli/cli/pull/9000".to_string(),
            title: "Fix the thing".to_string(),
            body: Some("Body text".to_string()),
            state: ActivityState::Merged,
            created_at: Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap(),
            closed_at: Some(Utc.with_ymd_and_hms(2025, 1, 2, 0, 0, 0).unwrap()),
            merged_at: Some(Utc.with_ymd_and_hms(2025, 1, 2, 0, 0, 0).unwrap()),
            additions: Some(10),
            deletions: Some(3),
            comments_count: Some(2),
            labels: vec!["bug".to_string()],
            repo_stars: Some(1234),
        }
    }

    #[test]
    fn build_request_maps_fetched_and_flags() {
        let fetched = sample_fetched();
        let req = build_request(
            Forge::GitHub,
            "cli",
            "cli",
            ActivityKind::PullRequest,
            9000,
            7,
            true,
            vec![0.1_f32; 384],
            &fetched,
        );
        assert_eq!(req.forge, Forge::GitHub);
        assert_eq!(req.repo_owner, "cli");
        assert_eq!(req.repo_name, "cli");
        assert_eq!(req.kind, ActivityKind::PullRequest);
        assert_eq!(req.number, 9000);
        assert_eq!(req.impact, 7);
        assert!(req.featured);
        assert_eq!(req.url, "https://github.com/cli/cli/pull/9000");
        assert_eq!(req.state, ActivityState::Merged);
        assert_eq!(req.merged_at, fetched.merged_at);
        assert_eq!(req.labels, vec!["bug".to_string()]);
        assert_eq!(req.embedding.as_ref().map(Vec::len), Some(384));
        assert!(req.published);
        assert_eq!(req.content_hash, None);
    }

    #[test]
    fn split_repo_rejects_bad_input() {
        assert!(split_repo("cli/cli").is_ok());
        assert!(split_repo("noslash").is_err());
        assert!(split_repo("/name").is_err());
        assert!(split_repo("owner/").is_err());
    }
}
