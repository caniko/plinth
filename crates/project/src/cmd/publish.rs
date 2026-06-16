use crate::{
    PageManifest, PublishArgs, PublishManifest, emit_check, emit_json, resolve_config_path,
    write_json_report,
};
use anyhow::{Context, Result, anyhow};
use plinth_project::{ProjectSite, RenderOptions, load_project_site, render_static, validate_site};
use std::{fs, path::Path};

pub fn publish_site(args: &PublishArgs) -> Result<()> {
    let config = resolve_config_path(args.config.config.clone())?;
    let site = load_project_site(&config).context("load project site")?;
    let report = validate_site(&site);
    if report.has_errors() {
        emit_check(&config, &report, args.json.json)?;
        return Err(anyhow!("project-site diagnostics failed"));
    }
    render_static(&site, &RenderOptions::new(&args.out)).context("render project site")?;
    let manifest = publish_manifest(&site, &args.out)?;
    let manifest_path = args.out.join("plinth-project-manifest.json");
    write_json_report(&manifest_path, &manifest)?;
    if args.json.json {
        emit_json(&manifest)?;
    } else {
        println!("Published {} -> {}", config.display(), args.out.display());
        println!("Wrote {}", manifest_path.display());
    }
    Ok(())
}

pub(crate) fn publish_manifest<'a>(
    site: &'a ProjectSite,
    out: &Path,
) -> Result<PublishManifest<'a>> {
    let mut files = collect_files(out)?;
    files.retain(|file| file != "plinth-project-manifest.json");
    Ok(PublishManifest {
        generator: "plinth-project",
        version: env!("CARGO_PKG_VERSION"),
        title: &site.title,
        pages: site
            .pages
            .iter()
            .map(|page| PageManifest {
                slug: &page.slug,
                title: &page.title,
                path: if page.slug == "index" {
                    "index.html".into()
                } else {
                    format!("{}/index.html", page.slug)
                },
            })
            .collect(),
        files,
    })
}

fn collect_files(root: &Path) -> Result<Vec<String>> {
    let mut files = Vec::new();
    collect_files_inner(root, root, &mut files)?;
    files.sort();
    Ok(files)
}

fn collect_files_inner(root: &Path, dir: &Path, files: &mut Vec<String>) -> Result<()> {
    for entry in fs::read_dir(dir).with_context(|| format!("failed to read {}", dir.display()))? {
        let entry = entry.with_context(|| format!("failed to read {}", dir.display()))?;
        let path = entry.path();
        if path.is_dir() {
            collect_files_inner(root, &path, files)?;
        } else if path.is_file() {
            let relative = path
                .strip_prefix(root)
                .context("generated file was outside output directory")?;
            files.push(relative.to_string_lossy().replace('\\', "/"));
        }
    }
    Ok(())
}
