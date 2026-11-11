//! External page-cache primitives used by the Dioxus HTTP boundary.
//!
//! Dioxus' incremental renderer is intentionally not used here: page
//! freshness is a Plinth concern and must remain explicit when a brick writes
//! content.  This small cache is framework-neutral so a Redis/file-backed
//! implementation can replace the in-process store without changing routes.

use std::{
    collections::HashMap,
    fs,
    path::PathBuf,
    sync::{
        Arc, RwLock,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

/// Rendering policy for a public route.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PagePolicy {
    /// Content-backed pages may be served from the external cache.
    CachedContent,
    /// Data must be read on every request (activity, todos, and 404s).
    FreshContent,
    /// The home page is the only route allowed to use streaming SSR.
    StreamingHome,
}

/// A normalized cache key. Query strings are retained because search/filter
/// parameters change the representation, while fragments never reach HTTP.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PageKey(String);

impl PageKey {
    pub fn from_request(method: &str, path: &str, query: Option<&str>) -> Option<Self> {
        if method != "GET"
            || path == "/api"
            || path.starts_with("/api/")
            || path == "/admin"
            || path.starts_with("/admin/")
        {
            return None;
        }

        let path = if path.is_empty() { "/" } else { path };
        let key = match query.filter(|query| !query.is_empty()) {
            Some(query) => format!("{path}?{query}"),
            None => path.to_string(),
        };
        Some(Self(key))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Classify routes before consulting the cache. The route enum is accepted as
/// an optional hint so callers cannot accidentally cache an activity/todo page
/// merely because its URL looks like a content page.
pub fn policy(path: &str) -> PagePolicy {
    match path {
        "/" => PagePolicy::StreamingHome,
        "/activity" | "/todos" => PagePolicy::FreshContent,
        path if path.starts_with("/activity/") || path.starts_with("/todos/") => {
            PagePolicy::FreshContent
        }
        _ => PagePolicy::CachedContent,
    }
}

#[derive(Debug, Clone)]
struct Entry {
    body: Arc<[u8]>,
    expires_at: Instant,
    tags: Vec<String>,
}

/// Bounded in-process cache. The API intentionally mirrors an external cache
/// (`get`, `put`, `invalidate`) so the backing store can be swapped for Redis
/// without changing invalidation call sites.
#[derive(Clone)]
pub struct PageCache {
    entries: Arc<RwLock<HashMap<PageKey, Entry>>>,
    ttl: Duration,
    capacity: usize,
    directory: Option<PathBuf>,
    generation: Arc<AtomicU64>,
    #[cfg(feature = "server")]
    inflight: Arc<tokio::sync::Mutex<HashMap<PageKey, Arc<tokio::sync::Notify>>>>,
}

impl PageCache {
    pub fn new(ttl: Duration, capacity: usize) -> Self {
        Self::with_directory(ttl, capacity, None)
    }

    /// Build a cache that optionally survives process restarts. The directory
    /// is deliberately supplied by the launcher so immutable Nix assets and
    /// mutable rendered responses cannot share a namespace.
    pub fn with_directory(ttl: Duration, capacity: usize, directory: Option<PathBuf>) -> Self {
        if let Some(directory) = &directory {
            let _ = fs::create_dir_all(directory);
        }
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            ttl,
            capacity: capacity.max(1),
            directory,
            generation: Arc::new(AtomicU64::new(0)),
            #[cfg(feature = "server")]
            inflight: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
        }
    }

    pub fn from_env(ttl: Duration, capacity: usize) -> Self {
        let build = std::env::current_exe().ok().map(|exe| {
            let modified = fs::metadata(&exe)
                .and_then(|metadata| metadata.modified())
                .ok()
                .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
                .map_or(0, |age| age.as_secs());
            format!("{}:{modified}", exe.display())
        });
        Self::with_directory(
            ttl,
            capacity,
            std::env::var_os("PLINTH_RENDER_CACHE_DIR")
                .map(|root| build_directory(PathBuf::from(root), build.as_deref().unwrap_or(""))),
        )
    }

    pub fn get(&self, key: &PageKey) -> Option<Arc<[u8]>> {
        let mut entries = self.entries.write().expect("page cache lock poisoned");
        if let Some(entry) = entries.get(key) {
            if entry.expires_at > Instant::now() {
                return Some(entry.body.clone());
            }
            entries.remove(key);
        }
        drop(entries);

        let body = self.read_disk(key)?;
        let mut entries = self.entries.write().expect("page cache lock poisoned");
        self.insert_memory(&mut entries, key.clone(), body.clone(), Vec::new());
        Some(body)
    }

    pub fn put(&self, key: PageKey, body: impl Into<Arc<[u8]>>, tags: Vec<String>) {
        let body = body.into();
        let mut entries = self.entries.write().expect("page cache lock poisoned");
        self.insert_memory(&mut entries, key.clone(), body.clone(), tags);
        drop(entries);
        self.write_disk(&key, &body);
    }

    /// Publish a render only if no invalidation happened after it claimed the
    /// miss. Holding the entry lock through the generation check and disk write
    /// makes invalidation and publication one ordered operation.
    #[cfg(feature = "server")]
    pub fn put_if_generation(
        &self,
        generation: u64,
        key: PageKey,
        body: impl Into<Arc<[u8]>>,
        tags: Vec<String>,
    ) -> bool {
        let body = body.into();
        let mut entries = self.entries.write().expect("page cache lock poisoned");
        if self.generation.load(Ordering::Acquire) != generation {
            return false;
        }
        self.insert_memory(&mut entries, key.clone(), body.clone(), tags);
        self.write_disk(&key, &body);
        true
    }

    fn insert_memory(
        &self,
        entries: &mut HashMap<PageKey, Entry>,
        key: PageKey,
        body: Arc<[u8]>,
        tags: Vec<String>,
    ) {
        if entries.len() >= self.capacity
            && !entries.contains_key(&key)
            && let Some(oldest) = entries
                .iter()
                .min_by_key(|(_, entry)| entry.expires_at)
                .map(|(key, _)| key.clone())
        {
            entries.remove(&oldest);
        }
        entries.insert(
            key,
            Entry {
                body,
                expires_at: Instant::now() + self.ttl,
                tags,
            },
        );
    }

    fn disk_path(&self, key: &PageKey) -> Option<PathBuf> {
        self.directory
            .as_ref()
            .map(|directory| directory.join(hex_key(key.as_str())).with_extension("html"))
    }

    fn read_disk(&self, key: &PageKey) -> Option<Arc<[u8]>> {
        let path = self.disk_path(key)?;
        let bytes = fs::read(&path).ok()?;
        if bytes.len() < 8 {
            let _ = fs::remove_file(path);
            return None;
        }
        let mut timestamp = [0; 8];
        timestamp.copy_from_slice(&bytes[..8]);
        let created = UNIX_EPOCH + Duration::from_secs(u64::from_le_bytes(timestamp));
        if SystemTime::now()
            .duration_since(created)
            .ok()
            .is_none_or(|age| age > self.ttl)
        {
            let _ = fs::remove_file(path);
            return None;
        }
        Some(Arc::<[u8]>::from(bytes[8..].to_vec()))
    }

    fn write_disk(&self, key: &PageKey, body: &[u8]) {
        let Some(path) = self.disk_path(key) else {
            return;
        };
        let Some(parent) = path.parent() else {
            return;
        };
        if fs::create_dir_all(parent).is_err() {
            return;
        }
        let created = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            .to_le_bytes();
        let mut encoded = Vec::with_capacity(8 + body.len());
        encoded.extend_from_slice(&created);
        encoded.extend_from_slice(body);
        let temporary = path.with_extension(format!("tmp-{}", std::process::id()));
        if fs::write(&temporary, encoded).is_ok() {
            let _ = fs::rename(temporary, path);
        }
    }

    pub fn invalidate_key(&self, key: &PageKey) -> bool {
        self.generation.fetch_add(1, Ordering::AcqRel);
        let removed = self
            .entries
            .write()
            .expect("page cache lock poisoned")
            .remove(key)
            .is_some();
        if let Some(path) = self.disk_path(key) {
            let _ = fs::remove_file(path);
        }
        removed
    }

    pub fn invalidate_tags(&self, tags: &[String]) -> usize {
        self.generation.fetch_add(1, Ordering::AcqRel);
        let mut entries = self.entries.write().expect("page cache lock poisoned");
        let before = entries.len();
        entries.retain(|_, entry| !entry.tags.iter().any(|tag| tags.contains(tag)));
        before - entries.len()
    }

    pub fn clear(&self) {
        self.generation.fetch_add(1, Ordering::AcqRel);
        self.entries
            .write()
            .expect("page cache lock poisoned")
            .clear();
        if let Some(directory) = &self.directory
            && let Ok(files) = fs::read_dir(directory)
        {
            for file in files.flatten() {
                if file
                    .path()
                    .extension()
                    .is_some_and(|extension| extension == "html")
                {
                    let _ = fs::remove_file(file.path());
                }
            }
        }
    }

    /// Claim a cache miss for rendering. A concurrent request waits for the
    /// owner to publish a complete response, then reuses it. If the owner
    /// cannot cache a response, the waiter retries and becomes the next owner.
    #[cfg(feature = "server")]
    pub async fn claim_render(&self, key: &PageKey) -> Result<u64, Arc<[u8]>> {
        loop {
            let waiter = {
                let mut inflight = self.inflight.lock().await;
                if let Some(notify) = inflight.get(key) {
                    // Register the notification while holding the map lock.
                    // Creating it after releasing the lock can lose a
                    // notify_waiters call between those two operations.
                    Some(notify.clone().notified_owned())
                } else {
                    inflight.insert(key.clone(), Arc::new(tokio::sync::Notify::new()));
                    None
                }
            };

            let Some(notify) = waiter else {
                return Ok(self.generation.load(Ordering::Acquire));
            };
            notify.await;
            if let Some(body) = self.get(key) {
                return Err(body);
            }
        }
    }

    /// Release a render claim after the response has either been cached or
    /// deliberately left uncached.
    #[cfg(feature = "server")]
    pub async fn release_render(&self, key: &PageKey) {
        let notify = self.inflight.lock().await.remove(key);
        if let Some(notify) = notify {
            notify.notify_waiters();
        }
    }
}

/// Rendered pages embed one build's hydration data and asset URLs, so each
/// build (the executable's path and modification time; a Nix store path is
/// unique per build) gets its own directory, and other builds' entries are
/// removed. Serving an earlier build's page with this build's WebAssembly
/// breaks hydration.
fn build_directory(root: PathBuf, build: &str) -> PathBuf {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    build.hash(&mut hasher);
    let name = format!("build-{:016x}", hasher.finish());
    if let Ok(entries) = fs::read_dir(&root) {
        for entry in entries.flatten() {
            if entry.file_name() != name.as_str() {
                let path = entry.path();
                let _ = if path.is_dir() {
                    fs::remove_dir_all(path)
                } else {
                    fs::remove_file(path)
                };
            }
        }
    }
    root.join(name)
}

fn hex_key(value: &str) -> String {
    value.bytes().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_keys_are_safe_and_normalized() {
        assert_eq!(
            PageKey::from_request("GET", "/posts", Some("page=2"))
                .unwrap()
                .as_str(),
            "/posts?page=2"
        );
        assert!(PageKey::from_request("POST", "/posts", None).is_none());
        assert!(PageKey::from_request("GET", "/api/posts", None).is_none());
        assert!(PageKey::from_request("GET", "/api", None).is_none());
        assert!(PageKey::from_request("GET", "/admin", None).is_none());
        assert_eq!(
            PageKey::from_request("GET", "", None).unwrap().as_str(),
            "/"
        );
    }

    #[test]
    fn route_policy_keeps_fresh_pages_out_of_cache() {
        assert_eq!(policy("/"), PagePolicy::StreamingHome);
        assert_eq!(policy("/activity/42"), PagePolicy::FreshContent);
        assert_eq!(policy("/todos"), PagePolicy::FreshContent);
        assert_eq!(policy("/posts/example"), PagePolicy::CachedContent);
    }

    #[test]
    fn invalidation_removes_matching_tags_only() {
        let cache = PageCache::new(Duration::from_secs(60), 8);
        cache.put(
            PageKey("/posts/a".into()),
            Arc::<[u8]>::from(&b"a"[..]),
            vec!["post:a".into()],
        );
        cache.put(
            PageKey("/posts/b".into()),
            Arc::<[u8]>::from(&b"b"[..]),
            vec!["post:b".into()],
        );
        assert_eq!(cache.invalidate_tags(&["post:a".into()]), 1);
        assert!(cache.get(&PageKey("/posts/a".into())).is_none());
        assert!(cache.get(&PageKey("/posts/b".into())).is_some());
    }

    #[test]
    fn each_build_gets_its_own_cache_directory() {
        let root = std::env::temp_dir().join(format!("plinth-build-cache-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("build-old")).unwrap();
        fs::write(root.join("legacy.html"), b"old").unwrap();
        let first = build_directory(root.clone(), "/nix/store/a-plinth/bin/plinth-web:1");
        fs::create_dir_all(&first).unwrap();
        assert_eq!(
            first,
            build_directory(root.clone(), "/nix/store/a-plinth/bin/plinth-web:1")
        );
        assert!(first.exists(), "the current build's entries stay");
        assert!(!root.join("build-old").exists() && !root.join("legacy.html").exists());
        let second = build_directory(root.clone(), "/nix/store/b-plinth/bin/plinth-web:1");
        assert_ne!(first, second);
        assert!(
            !first.exists(),
            "a new build drops the previous build's pages"
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn disk_cache_reuses_completed_response() {
        let directory =
            std::env::temp_dir().join(format!("plinth-page-cache-{}", std::process::id()));
        let _ = fs::remove_dir_all(&directory);
        let first = PageCache::with_directory(Duration::from_secs(60), 8, Some(directory.clone()));
        first.put(
            PageKey("/about".into()),
            Arc::<[u8]>::from(&b"rendered"[..]),
            Vec::new(),
        );
        let second = PageCache::with_directory(Duration::from_secs(60), 8, Some(directory.clone()));
        assert_eq!(
            second.get(&PageKey("/about".into())).as_deref(),
            Some(&b"rendered"[..])
        );
        second.clear();
        let _ = fs::remove_dir_all(directory);
    }

    #[cfg(feature = "server")]
    #[tokio::test]
    async fn invalidation_cannot_publish_an_old_render() {
        let cache = PageCache::new(Duration::from_secs(60), 8);
        let key = PageKey("/posts/a".into());
        let generation = cache.claim_render(&key).await.unwrap();
        cache.clear();
        assert!(!cache.put_if_generation(
            generation,
            key.clone(),
            Arc::<[u8]>::from(&b"stale"[..]),
            Vec::new(),
        ));
        cache.release_render(&key).await;
        assert!(cache.get(&key).is_none());
    }
}
