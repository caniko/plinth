use std::fs;
use std::io::{Read as _, Write as _};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::path::{Path, PathBuf};
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU64, Ordering},
    mpsc,
};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use notify::{RecommendedWatcher, RecursiveMode, Watcher as _};
use thiserror::Error;

use crate::{ProjectSite, RenderError, RenderOptions, render_static};

/// The HTTP endpoint path used for live-reload polling.
pub const RELOAD_ENDPOINT: &str = "/__plinth_project_reload";

/// Configuration for the development server.
#[derive(Clone, Debug)]
pub struct ServeOptions {
    /// Directory containing the rendered static output.
    pub output_dir: PathBuf,
    /// Host interface to bind to (default `"127.0.0.1"`).
    pub host: String,
    /// Port to listen on (default `1111`; falls back to ephemeral when
    /// `1111` is taken).
    pub port: u16,
    /// Whether to open the site in the default browser on start.
    pub open_browser: bool,
    /// Whether to watch source paths and auto-render on changes.
    pub watch: bool,
    /// Whether to inject the live-reload script and serve the reload
    /// endpoint.
    pub reload: bool,
    /// Directories to watch for changes when `watch` is enabled.
    pub watch_paths: Vec<PathBuf>,
}

impl ServeOptions {
    #[must_use]
    pub fn new(output_dir: impl Into<PathBuf>) -> Self {
        Self {
            output_dir: output_dir.into(),
            host: "127.0.0.1".into(),
            port: 1111,
            open_browser: true,
            watch: false,
            reload: false,
            watch_paths: Vec::new(),
        }
    }
}

/// Errors that can occur during development-server operation.
#[derive(Debug, Error)]
pub enum DevServerError {
    /// Static rendering failed.
    #[error("failed to render site: {0}")]
    Render(#[from] RenderError),
    /// An I/O operation (bind, read, write) failed.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    /// The file-system watcher could not be started.
    #[error("failed to start file watcher: {0}")]
    Watch(#[from] notify::Error),
    /// The user-supplied build callback returned an error.
    #[error("failed to build site: {0}")]
    Build(String),
    /// The browser could not be opened to the given URL.
    #[error("failed to open {url}: {source}")]
    Open { url: String, source: std::io::Error },
}

/// Thread-safe monotonically-increasing version counter for live-reload
/// signalling.
#[derive(Clone)]
pub struct ReloadState {
    version: Arc<AtomicU64>,
}

impl ReloadState {
    fn new() -> Self {
        Self {
            version: Arc::new(AtomicU64::new(1)),
        }
    }

    fn bump(&self) {
        self.version.fetch_add(1, Ordering::AcqRel);
    }

    fn version(&self) -> u64 {
        self.version.load(Ordering::Acquire)
    }
}

/// A minimal single-threaded HTTP server that serves a directory of static
/// files, with optional live-reload support.
pub struct StaticServer {
    stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
    host: String,
    port: u16,
}

impl StaticServer {
    /// Starts serving files from `root` on the given `host` and `port`.
    ///
    /// The server runs in a background thread until dropped.
    pub fn start(
        root: PathBuf,
        host: impl Into<String>,
        port: u16,
    ) -> Result<Self, DevServerError> {
        Self::start_with_reload(root, host, port, None)
    }

    fn start_with_reload(
        root: PathBuf,
        host: impl Into<String>,
        port: u16,
        reload: Option<ReloadState>,
    ) -> Result<Self, DevServerError> {
        let host = host.into();
        let listener = bind_listener(&host, port)?;
        listener.set_nonblocking(true)?;
        let local = listener.local_addr()?;
        let port = local.port();
        let stop = Arc::new(AtomicBool::new(false));
        let thread_stop = Arc::clone(&stop);
        let thread_host = host.clone();
        let handle = thread::spawn(move || {
            while !thread_stop.load(Ordering::Acquire) {
                match listener.accept() {
                    Ok((stream, _)) => {
                        let _ = serve_static_request(stream, &root, reload.as_ref());
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(_) => break,
                }
            }
        });

        Ok(Self {
            stop,
            handle: Some(handle),
            host: thread_host,
            port,
        })
    }

    /// Returns the full base URL of the running server.
    #[must_use]
    pub fn base_url(&self) -> String {
        format!("http://{}:{}/", self.host, self.port)
    }

    /// Returns the port the server is listening on.
    #[must_use]
    pub fn port(&self) -> u16 {
        self.port
    }
}

impl Drop for StaticServer {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        let _ = TcpStream::connect((self.host.as_str(), self.port));
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

/// Runs the full development-server lifecycle.
///
/// Renders the site, starts the static server, optionally opens the
/// browser, and blocks while watching for file changes (if `watch` is
/// enabled) or indefinitely otherwise.
pub fn serve_development<F, E>(options: ServeOptions, build_site: F) -> Result<(), DevServerError>
where
    F: Fn() -> Result<ProjectSite, E>,
    E: std::fmt::Display,
{
    let (server, reload_state) = start_development_server(&options, &build_site)?;

    if options.watch {
        watch_and_render(options, build_site, reload_state)?;
    } else {
        loop {
            thread::sleep(Duration::from_mins(1));
        }
    }

    drop(server);
    Ok(())
}

/// Renders the site, starts the static server, opens the browser, and
/// returns the server handle and reload state for external lifecycle
/// management.
pub fn start_development_server<F, E>(
    options: &ServeOptions,
    build_site: &F,
) -> Result<(StaticServer, ReloadState), DevServerError>
where
    F: Fn() -> Result<ProjectSite, E>,
    E: std::fmt::Display,
{
    let reload_state = ReloadState::new();
    render_dev_site(options, build_site, &reload_state)?;

    let server_reload = options.reload.then_some(reload_state.clone());
    let server = StaticServer::start_with_reload(
        options.output_dir.clone(),
        options.host.clone(),
        options.port,
        server_reload,
    )?;
    let url = server.base_url();
    println!("Serving project site at {url}");

    if options.open_browser {
        open::that(&url).map_err(|source| DevServerError::Open {
            url: url.clone(),
            source,
        })?;
    }

    Ok((server, reload_state))
}

fn render_dev_site<F, E>(
    options: &ServeOptions,
    build_site: &F,
    reload_state: &ReloadState,
) -> Result<(), DevServerError>
where
    F: Fn() -> Result<ProjectSite, E>,
    E: std::fmt::Display,
{
    let site = build_site().map_err(|error| DevServerError::Build(error.to_string()))?;
    let render_options = if options.reload {
        RenderOptions::new(&options.output_dir).with_dev_reload(RELOAD_ENDPOINT)
    } else {
        RenderOptions::new(&options.output_dir)
    };
    render_static(&site, &render_options)?;
    reload_state.bump();
    Ok(())
}

fn watch_and_render<F, E>(
    options: ServeOptions,
    build_site: F,
    reload_state: ReloadState,
) -> Result<(), DevServerError>
where
    F: Fn() -> Result<ProjectSite, E>,
    E: std::fmt::Display,
{
    let (tx, rx) = mpsc::channel();
    let mut watcher = RecommendedWatcher::new(
        move |result| {
            let _ = tx.send(result);
        },
        notify::Config::default(),
    )?;

    for path in &options.watch_paths {
        watcher.watch(path, RecursiveMode::Recursive)?;
        println!("Watching {}", path.display());
    }

    let mut last_render = Instant::now();
    loop {
        match rx.recv() {
            Ok(Ok(_event)) => {
                if last_render.elapsed() < Duration::from_millis(150) {
                    continue;
                }
                last_render = Instant::now();
                match render_dev_site(&options, &build_site, &reload_state) {
                    Ok(()) => println!("Rerendered project site"),
                    Err(error) => eprintln!("failed to rerender project site: {error}"),
                }
            }
            Ok(Err(error)) => eprintln!("project site watcher error: {error}"),
            Err(_) => break,
        }
    }

    Ok(())
}

fn bind_listener(host: &str, port: u16) -> Result<TcpListener, DevServerError> {
    if port == 1111 {
        let primary = format!("{host}:{port}");
        if let Ok(listener) = TcpListener::bind(&primary) {
            return Ok(listener);
        }
        let fallback = format!("{host}:0");
        return TcpListener::bind(&fallback).map_err(DevServerError::Io);
    }

    let addr = format!("{host}:{port}");
    let addr = addr
        .to_socket_addrs()?
        .next()
        .ok_or_else(|| std::io::Error::other(format!("could not resolve {host}:{port}")))?;
    TcpListener::bind(addr).map_err(DevServerError::Io)
}

fn serve_static_request(
    mut stream: TcpStream,
    root: &Path,
    reload: Option<&ReloadState>,
) -> Result<(), std::io::Error> {
    let mut buf = [0; 2048];
    let n = stream.read(&mut buf)?;
    let request = String::from_utf8_lossy(&buf[..n]);
    let path = request
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .unwrap_or("/");

    if path.split('?').next() == Some(RELOAD_ENDPOINT) {
        let body = reload.map_or(0, ReloadState::version).to_string();
        return write_response(&mut stream, "text/plain; charset=utf-8", body.as_bytes());
    }

    let file = resolve_static_path(root, path);
    if let Ok(bytes) = fs::read(&file) {
        write_response(&mut stream, content_type(&file), &bytes)?;
    } else {
        let body = b"not found";
        let header = format!(
            "HTTP/1.1 404 Not Found\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            body.len()
        );
        stream.write_all(header.as_bytes())?;
        stream.write_all(body)?;
    }
    Ok(())
}

fn write_response(
    stream: &mut TcpStream,
    content_type: &str,
    body: &[u8],
) -> Result<(), std::io::Error> {
    let header = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    stream.write_all(header.as_bytes())?;
    stream.write_all(body)
}

fn resolve_static_path(root: &Path, request_path: &str) -> PathBuf {
    let clean = request_path
        .split('?')
        .next()
        .unwrap_or("/")
        .trim_start_matches('/');
    if clean.is_empty() {
        return root.join("index.html");
    }
    if clean.contains("..") {
        return root.join("__invalid__");
    }
    let path = root.join(clean);
    if request_path.ends_with('/') {
        path.join("index.html")
    } else {
        path
    }
}

fn content_type(path: &Path) -> &'static str {
    match path.extension().and_then(|extension| extension.to_str()) {
        Some("css") => "text/css; charset=utf-8",
        Some("html") => "text/html; charset=utf-8",
        Some("png") => "image/png",
        Some("svg") => "image/svg+xml",
        Some("webp") => "image/webp",
        _ => "application/octet-stream",
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Read as _, Write as _};
    use std::net::TcpStream;

    use super::{StaticServer, resolve_static_path};

    #[test]
    fn resolves_static_paths_without_traversal() {
        let root = std::path::Path::new("/site");
        assert_eq!(resolve_static_path(root, "/"), root.join("index.html"));
        assert_eq!(
            resolve_static_path(root, "/comparison/"),
            root.join("comparison").join("index.html")
        );
        assert_eq!(
            resolve_static_path(root, "/../secret"),
            root.join("__invalid__")
        );
    }

    #[test]
    fn static_server_binds_ephemeral_port_and_serves_files() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("index.html"), "home").unwrap();
        std::fs::create_dir_all(dir.path().join("nested")).unwrap();
        std::fs::write(dir.path().join("nested").join("index.html"), "nested").unwrap();
        std::fs::write(dir.path().join("style.css"), "body{}").unwrap();

        let server = StaticServer::start(dir.path().to_path_buf(), "127.0.0.1", 0).unwrap();
        assert!(server.port() > 0);

        assert!(request(server.port(), "/").contains("home"));
        assert!(request(server.port(), "/nested/").contains("nested"));
        assert!(request(server.port(), "/style.css").contains("Content-Type: text/css"));
        assert!(request(server.port(), "/../secret").contains("404 Not Found"));
    }

    fn request(port: u16, path: &str) -> String {
        let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
        write!(stream, "GET {path} HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n").unwrap();
        let mut response = String::new();
        stream.read_to_string(&mut response).unwrap();
        response
    }
}
