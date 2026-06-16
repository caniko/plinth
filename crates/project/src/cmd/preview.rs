use crate::PreviewArgs;
use anyhow::{Context, Result, anyhow};
use plinth_project::dev::StaticServer;
use std::{thread, time::Duration};

pub fn preview_site(args: PreviewArgs) -> Result<()> {
    if !args.dir.exists() {
        return Err(anyhow!(
            "{} does not exist; run `plinth-project build --out {}` first",
            args.dir.display(),
            args.dir.display()
        ));
    }
    let open_browser = args.serve.open_browser();
    let server = StaticServer::start(args.dir.clone(), args.serve.host, args.serve.port)
        .context("failed to start preview server")?;
    let url = server.base_url();
    println!("Serving built project site at {url}");
    if open_browser {
        open::that(&url).with_context(|| format!("failed to open {url}"))?;
    }
    if std::env::var_os("PLINTH_PROJECT_SERVE_ONCE").is_some() {
        return Ok(());
    }
    loop {
        thread::sleep(Duration::from_secs(60));
    }
}
