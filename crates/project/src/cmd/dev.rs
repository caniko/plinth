use crate::{DevArgs, ServeArgs, resolve_config_path};
use anyhow::{Context, Result};
use plinth_project::{
    dev::{ServeOptions, serve_development, start_development_server},
    load_project_site, project_watch_paths,
};
use std::path::PathBuf;

struct ServeRenderedArgs {
    config: PathBuf,
    out: PathBuf,
    host: String,
    port: u16,
    open_browser: bool,
    watch: bool,
}

pub fn dev_site(args: DevArgs) -> Result<()> {
    let config = resolve_config_path(args.config.config)?;
    let open_browser = args.serve.open_browser();
    serve_rendered_site(ServeRenderedArgs {
        config,
        out: args.out,
        host: args.serve.host,
        port: args.serve.port,
        open_browser,
        watch: !args.no_watch,
    })
}

pub fn serve_site(args: ServeArgs) -> Result<()> {
    let config = resolve_config_path(args.config)?;
    serve_rendered_site(ServeRenderedArgs {
        config,
        out: args.out,
        host: args.host,
        port: args.port,
        open_browser: !args.no_open,
        watch: args.watch,
    })
}

fn serve_rendered_site(args: ServeRenderedArgs) -> Result<()> {
    let mut options = ServeOptions::new(&args.out);
    options.host = args.host;
    options.port = args.port;
    options.open_browser = args.open_browser;
    options.watch = args.watch;
    options.reload = args.watch;
    options.watch_paths = project_watch_paths(&args.config).context("load project watch paths")?;

    if std::env::var_os("PLINTH_PROJECT_SERVE_ONCE").is_some() {
        let (_server, _reload) =
            start_development_server(&options, &|| load_project_site(&args.config))
                .context("start project-site server")?;
        return Ok(());
    }

    serve_development(options, || load_project_site(&args.config)).context("serve project site")
}
