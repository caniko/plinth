use super::{AuditSiteArgs, ProjectSite, VIEWPORTS, safe_route_name};
use anyhow::{Context, Result, ensure};
use plinth_project::{resolve_output_path, validate_output_directory};
use std::path::{Path, PathBuf};

pub(super) fn capture_job_path(args: &AuditSiteArgs) -> PathBuf {
    args.capture_manifest
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("capture_job.json")
}

pub(super) fn validate_site(
    site: &ProjectSite,
    args: &AuditSiteArgs,
    routes: &[String],
) -> Result<()> {
    if args.shared_capture {
        let job = capture_job_path(args);
        let mut directories = vec![args.capture_output.as_path()];
        if let Some(cache) = &args.rubric_cache {
            directories.push(cache);
        }
        let files = [
            &args.report,
            &args.capture_manifest,
            &args.visual_report,
            &job,
        ];
        validate(site, &args.out, &directories, &files, &[])?;
        // The producer chooses content-addressed capture names. Keep its writable
        // trees separate from metadata, including the job it must read.
        for directory in directories {
            for file in files {
                disjoint(directory, file)?;
            }
        }
        Ok(())
    } else {
        let captures = routes
            .iter()
            .flat_map(|route| {
                VIEWPORTS.iter().map(move |viewport| {
                    args.screenshots
                        .join(safe_route_name(route))
                        .join(format!("{}.png", viewport.name))
                })
            })
            .collect::<Vec<_>>();
        validate(
            site,
            &args.out,
            &[&args.screenshots],
            &[&args.report],
            &captures,
        )
    }
}

pub(super) fn validate(
    site: &ProjectSite,
    out: &Path,
    directories: &[&Path],
    reports: &[&PathBuf],
    captures: &[PathBuf],
) -> Result<()> {
    // This shared protection also rejects a report path that aliases an input.
    // Complete the entire preflight before render_static performs any cleanup.
    for path in std::iter::once(out)
        .chain(directories.iter().copied())
        .chain(reports.iter().map(|path| path.as_path()))
    {
        validate_output_directory(site, path)
            .with_context(|| format!("validate audit destination {}", path.display()))?;
    }
    for (index, directory) in directories.iter().enumerate() {
        disjoint(out, directory)?;
        for other in &directories[index + 1..] {
            disjoint(directory, other)?;
        }
    }
    for (index, report) in reports.iter().enumerate() {
        disjoint(out, report)?;
        for other in &reports[index + 1..] {
            disjoint(report, other)?;
        }
        for capture in captures {
            disjoint(report, capture)?;
        }
    }
    Ok(())
}

fn disjoint(first: &Path, second: &Path) -> Result<()> {
    let first = resolve_output_path(first)?;
    let second = resolve_output_path(second)?;
    ensure!(
        !first.starts_with(&second) && !second.starts_with(&first),
        "audit destinations overlap: {} and {}",
        first.display(),
        second.display()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AuditCommands, Cli, Commands};
    use clap::Parser;

    #[test]
    fn default_audit_layouts_allow_reports_beside_captures() {
        let site = ProjectSite::new("example", "example site");
        for extra in [vec![], vec!["--shared-capture"]] {
            let cli =
                Cli::try_parse_from(["plinth-project", "audit", "site"].into_iter().chain(extra))
                    .unwrap();
            let Commands::Audit(AuditCommands::Site(args)) = cli.command else {
                panic!("expected site audit");
            };
            validate_site(&site, &args, &["/".into(), "/people/".into()]).unwrap();
        }
    }

    #[cfg(feature = "brick-install")]
    #[test]
    fn install_audit_reports_cannot_overwrite_inputs_or_captures() {
        let dir = tempfile::tempdir().unwrap();
        let config = dir.path().join("project.toml");
        std::fs::write(&config, "authoring input").unwrap();
        let mut site = ProjectSite::new("example", "example site");
        site.source_paths.push(config.clone());
        let out = dir.path().join("public");
        let screenshots = dir.path().join("screenshots");
        let captures = vec![screenshots.join("desktop.png")];
        let report = screenshots.join("report.json");
        validate(&site, &out, &[&screenshots], &[&report], &captures).unwrap();
        assert!(validate(&site, &out, &[&screenshots], &[&config], &captures).is_err());
        assert!(validate(&site, &out, &[&out], &[&report], &captures).is_err());
        assert!(validate(&site, &out, &[&screenshots], &[&captures[0]], &captures).is_err());
        assert_eq!(std::fs::read_to_string(config).unwrap(), "authoring input");
    }
}
