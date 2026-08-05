use crate::{
    AuditInstallArgs, AuditSiteArgs, FullSiteAuditReport, PLINTH_SITE_BEAUTY_PRESET,
    PageAuditReport, RubricAudit, ScreenshotAudit, SiteAuditReport, VIEWPORTS, Viewport,
    WEBSITE_RUBRIC_PRESET, resolve_config_path, write_json_report,
};
use anyhow::{Context, Result};
use anyhow::{bail, ensure};
use plinth_project::{
    ProjectSection, ProjectSite, RenderOptions, dev::StaticServer, install_ux_report,
    load_project_site, render_install_fragment, render_static,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn audit_install(args: &AuditInstallArgs) -> Result<()> {
    let config = resolve_config_path(args.config.config.clone())?;
    let site = load_project_site(&config).context("load project site")?;
    render_static(&site, &RenderOptions::new(&args.out)).context("render project site")?;
    let install = install_ux_report(&site).context("site does not define an install section")?;
    let audit_path = write_install_audit_page(&args.out, &site)?;
    create_clean_dir(&args.screenshots)?;

    let server = StaticServer::start(args.out.clone(), "127.0.0.1", 0)
        .context("failed to start static server")?;
    let audit_url = format!("{}{}", server.base_url(), audit_path);
    let mut screenshots = Vec::new();

    for viewport in VIEWPORTS {
        let path = args.screenshots.join(format!("{}.png", viewport.name));
        capture_screenshot(&args.browser, &audit_url, viewport, &path)?;
        let rubric = if args.fake_ai {
            RubricAudit::Pass {
                reason: "fake rubric enabled".into(),
                anomalies: Vec::new(),
            }
        } else if args.skip_ai {
            RubricAudit::Skipped {
                reason: "AI rubric skipped by flag".into(),
            }
        } else {
            run_visual_rubric(&args.rubric_bin, &path, WEBSITE_RUBRIC_PRESET)
        };

        screenshots.push(ScreenshotAudit {
            name: viewport.name.into(),
            width: viewport.width,
            height: viewport.height,
            path,
            rubric,
        });
    }

    drop(server);

    write_json_report(
        &args.report,
        &SiteAuditReport {
            install,
            screenshots,
        },
    )
}

pub fn audit_site(args: &AuditSiteArgs) -> Result<()> {
    let config = resolve_config_path(args.config.config.clone())?;
    let site = load_project_site(&config).context("load project site")?;
    render_static(&site, &RenderOptions::new(&args.out)).context("render project site")?;
    create_clean_dir(&args.screenshots)?;

    let routes = if args.routes.is_empty() {
        rendered_routes(&site)
    } else {
        normalize_routes(&args.routes)?
    };

    if args.shared_capture {
        return audit_site_shared(args, config, routes);
    }

    let server = StaticServer::start(args.out.clone(), "127.0.0.1", 0)
        .context("failed to start static server")?;
    let served_url = server.base_url();
    let mut pages = Vec::new();
    let mut gate_failed = false;

    for route in routes {
        let mut screenshots = Vec::new();
        let route_dir = args.screenshots.join(safe_route_name(&route));
        for viewport in VIEWPORTS {
            let path = route_dir.join(format!("{}.png", viewport.name));
            let url = format!("{}{}", served_url, route.trim_start_matches('/'));
            capture_screenshot(&args.browser, &url, viewport, &path)?;
            let rubric = if args.fake_ai {
                RubricAudit::Pass {
                    reason: "fake rubric enabled".into(),
                    anomalies: Vec::new(),
                }
            } else if args.skip_ai {
                RubricAudit::Skipped {
                    reason: "AI rubric skipped by flag".into(),
                }
            } else {
                run_visual_rubric(&args.rubric_bin, &path, PLINTH_SITE_BEAUTY_PRESET)
            };
            gate_failed |= rubric_blocks_gate(&rubric);

            screenshots.push(ScreenshotAudit {
                name: viewport.name.into(),
                width: viewport.width,
                height: viewport.height,
                path,
                rubric,
            });
        }

        pages.push(PageAuditReport { route, screenshots });
    }

    let report = FullSiteAuditReport {
        config,
        out: args.out.clone(),
        served_url,
        preset: PLINTH_SITE_BEAUTY_PRESET,
        pages,
    };

    drop(server);

    write_json_report(&args.report, &report)?;
    println!(
        "Wrote Plinth visual audit report to {}",
        args.report.display()
    );
    println!("Screenshots: {}", args.screenshots.display());

    if gate_failed && !args.no_fail_on_rubric {
        bail!(
            "Plinth visual audit failed; inspect {} and screenshots under {}",
            args.report.display(),
            args.screenshots.display()
        );
    }

    Ok(())
}

#[derive(Serialize)]
struct SharedCaptureJob {
    schema_version: u32,
    target: String,
    revision: String,
    dirty: bool,
    cells: Vec<SharedCaptureCell>,
}

#[derive(Serialize)]
struct SharedCaptureCell {
    id: String,
    path: String,
    state: String,
    viewport: SharedViewport,
    theme: String,
    locale: String,
    presets: Vec<String>,
}

#[derive(Serialize)]
struct SharedViewport {
    width: u32,
    height: u32,
    dpr: f32,
}

#[derive(Debug, Deserialize)]
struct SharedRunReport {
    #[serde(default)]
    served_url: Option<String>,
    cells: Vec<SharedRunCell>,
}

#[derive(Debug, Deserialize)]
struct SharedRunCell {
    id: String,
    image: PathBuf,
    rubric: serde_json::Value,
}

fn audit_site_shared(args: &AuditSiteArgs, config: PathBuf, routes: Vec<String>) -> Result<()> {
    let revision = git_revision()?;
    let dirty = git_dirty()?;
    let cells = routes
        .iter()
        .flat_map(|route| {
            VIEWPORTS.iter().map(move |viewport| SharedCaptureCell {
                id: format!("{}/{}/default/en-US", safe_route_name(route), viewport.name),
                path: route.clone(),
                state: route.clone(),
                viewport: SharedViewport {
                    width: viewport.width,
                    height: viewport.height,
                    dpr: 1.0,
                },
                theme: "default".into(),
                locale: "en-US".into(),
                presets: vec![PLINTH_SITE_BEAUTY_PRESET.into()],
            })
        })
        .collect::<Vec<_>>();
    let job = SharedCaptureJob {
        schema_version: 1,
        target: format!(
            "{}/plinth-project",
            revision_target_name(&config).unwrap_or_else(|| "plinth".into())
        ),
        revision,
        dirty,
        cells,
    };
    let job_path = args
        .capture_manifest
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("capture_job.json");
    let invocation_dir = std::env::current_dir().context("read audit working directory")?;
    let absolute_path = |path: &Path| {
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            invocation_dir.join(path)
        }
    };
    let out = absolute_path(&args.out);
    let command_job = absolute_path(&job_path);
    let capture_output = absolute_path(&args.capture_output);
    let capture_manifest = absolute_path(&args.capture_manifest);
    let visual_report = absolute_path(&args.visual_report);
    write_json_report(&job_path, &job)?;

    let mut command = if let Some(project) = &args.rubric_project {
        let project = project
            .canonicalize()
            .with_context(|| format!("resolve rubric project {}", project.display()))?;
        let mut command = Command::new("nix");
        command
            .arg("develop")
            .arg("--no-write-lock-file")
            .arg(&project)
            .arg("-c")
            .arg("cargo")
            .arg("run")
            .arg("--locked")
            .arg("--manifest-path")
            .arg(project.join("Cargo.toml"))
            .arg("--features")
            .arg("audit")
            .arg("--bin")
            .arg("visual-rubric")
            .arg("--");
        command.current_dir(&project);
        command
    } else {
        Command::new(&args.rubric_bin)
    };
    command
        .arg("capture")
        .arg("--workspace")
        .arg(&invocation_dir)
        .arg("--root")
        .arg(out)
        .arg("--job")
        .arg(&command_job)
        .arg("--output")
        .arg(capture_output)
        .arg("--manifest")
        .arg(capture_manifest)
        .arg("--report")
        .arg(visual_report)
        .arg("--browser")
        .arg(&args.browser)
        .arg("--rubric-workers")
        .arg(args.rubric_workers.to_string())
        .arg("--preset")
        .arg(PLINTH_SITE_BEAUTY_PRESET);
    if let Some(cache) = &args.rubric_cache {
        command.arg("--cache-dir").arg(absolute_path(cache));
    }
    if args.fake_ai {
        command.arg("--fake-pass");
    }
    if args.skip_ai {
        command.arg("--skip-ai");
    }
    let output = command.output().with_context(|| {
        format!(
            "failed to run shared visual-rubric producer {}; set VISUAL_RUBRIC_BIN or pass --rubric-bin",
            args.rubric_bin.display()
        )
    })?;
    if !output.status.success() && !args.no_fail_on_rubric {
        bail!(
            "shared visual-rubric producer failed with status {}: {}{}",
            output.status,
            String::from_utf8_lossy(&output.stderr),
            String::from_utf8_lossy(&output.stdout)
        );
    }

    let visual_report = read_shared_report(&args.visual_report)?;
    let cells_by_id = visual_report
        .cells
        .into_iter()
        .map(|cell| (cell.id.clone(), cell))
        .collect::<std::collections::BTreeMap<_, _>>();
    let mut pages = Vec::new();
    for route in routes {
        let screenshots = VIEWPORTS
            .iter()
            .map(|viewport| {
                let id = format!(
                    "{}/{}/default/en-US",
                    safe_route_name(&route),
                    viewport.name
                );
                let cell = cells_by_id
                    .get(&id)
                    .with_context(|| format!("shared report is missing capture cell {id}"))?;
                Ok(ScreenshotAudit {
                    name: viewport.name.into(),
                    width: viewport.width,
                    height: viewport.height,
                    path: cell.image.clone(),
                    rubric: rubric_from_shared_value(&cell.rubric),
                })
            })
            .collect::<Result<Vec<_>>>()?;
        pages.push(PageAuditReport { route, screenshots });
    }

    let report = FullSiteAuditReport {
        config,
        out: args.out.clone(),
        served_url: visual_report
            .served_url
            .unwrap_or_else(|| "shared-capture".into()),
        preset: PLINTH_SITE_BEAUTY_PRESET,
        pages,
    };
    write_json_report(&args.report, &report)?;
    println!(
        "Wrote Plinth shared visual audit report to {}",
        args.report.display()
    );
    println!("Capture manifest: {}", args.capture_manifest.display());

    Ok(())
}

fn read_shared_report(path: &Path) -> Result<SharedRunReport> {
    let bytes = fs::read(path).with_context(|| format!("read shared report {}", path.display()))?;
    serde_json::from_slice(&bytes)
        .with_context(|| format!("parse shared report {}", path.display()))
}

fn rubric_from_shared_value(value: &serde_json::Value) -> RubricAudit {
    let status = value
        .get("status")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("error");
    match status {
        "pass" => RubricAudit::Pass {
            reason: value
                .get("reason")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default()
                .into(),
            anomalies: value
                .get("anomalies")
                .and_then(serde_json::Value::as_array)
                .map(|values| {
                    values
                        .iter()
                        .filter_map(serde_json::Value::as_str)
                        .map(str::to_owned)
                        .collect()
                })
                .unwrap_or_default(),
        },
        "fail" => RubricAudit::Fail {
            reason: value
                .get("reason")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default()
                .into(),
            anomalies: value
                .get("anomalies")
                .and_then(serde_json::Value::as_array)
                .map(|values| {
                    values
                        .iter()
                        .filter_map(serde_json::Value::as_str)
                        .map(str::to_owned)
                        .collect()
                })
                .unwrap_or_default(),
        },
        "skipped" => RubricAudit::Skipped {
            reason: value
                .get("reason")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("AI rubric skipped")
                .into(),
        },
        _ => RubricAudit::Error {
            message: value
                .get("message")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("shared rubric returned an error")
                .into(),
        },
    }
}

fn git_revision() -> Result<String> {
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .context("read project git revision")?;
    ensure!(
        output.status.success(),
        "git rev-parse HEAD failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(String::from_utf8(output.stdout)
        .context("git revision was not UTF-8")?
        .trim()
        .to_owned())
}

fn git_dirty() -> Result<bool> {
    let output = Command::new("git")
        .args(["status", "--porcelain=v1"])
        .output()
        .context("read project git status")?;
    ensure!(
        output.status.success(),
        "git status failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(!output.stdout.is_empty())
}

fn revision_target_name(_config: &Path) -> Option<String> {
    std::env::current_dir()
        .ok()?
        .file_name()?
        .to_str()
        .map(str::to_owned)
}

fn write_install_audit_page(out: &Path, site: &ProjectSite) -> Result<String> {
    let install = site
        .pages
        .iter()
        .flat_map(|page| &page.sections)
        .find_map(|section| match section {
            ProjectSection::Install(install) => Some(install),
            _ => None,
        })
        .context("site does not define an install section")?;
    let audit_path = Path::new("__audit").join("install.html");
    let body = render_install_fragment(install);
    let html = format!(
        "<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><title>Install Audit</title><link rel=\"stylesheet\" href=\"/style.css\"></head><body><main>{body}</main></body></html>"
    );
    let target = out.join(&audit_path);
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    fs::write(&target, html).with_context(|| format!("failed to write {}", target.display()))?;
    Ok(audit_path.to_string_lossy().replace('\\', "/"))
}

fn capture_screenshot(browser: &Path, url: &str, viewport: &Viewport, output: &Path) -> Result<()> {
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let status = Command::new(browser)
        .arg("--headless")
        .arg("--disable-gpu")
        .arg("--hide-scrollbars")
        .arg("--no-sandbox")
        .arg(format!(
            "--window-size={},{}",
            viewport.width, viewport.height
        ))
        .arg(format!("--screenshot={}", output.display()))
        .arg(url)
        .status()
        .with_context(|| {
            format!(
                "failed to run browser {}; set PLINTH_PROJECT_BROWSER or pass --browser",
                browser.display()
            )
        })?;
    if !status.success() {
        bail!(
            "browser {} failed for {} screenshot with status {status}",
            browser.display(),
            viewport.name
        );
    }
    if !output.exists() {
        bail!(
            "browser {} reported success but did not write {}",
            browser.display(),
            output.display()
        );
    }
    Ok(())
}

fn run_visual_rubric(rubric_bin: &Path, image: &Path, preset: &str) -> RubricAudit {
    let output = Command::new(rubric_bin)
        .arg("configured")
        .arg("--json")
        .arg("--image")
        .arg(image)
        .arg("--preset")
        .arg(preset)
        .output();
    let output = match output {
        Ok(output) => output,
        Err(error) => {
            return RubricAudit::Error {
                message: format!(
                    "failed to run visual-rubric binary {}; set VISUAL_RUBRIC_BIN or pass --rubric-bin: {error}",
                    rubric_bin.display()
                ),
            };
        }
    };

    if !output.status.success() {
        return RubricAudit::Error {
            message: format!(
                "visual-rubric exited with status {}: {}{}",
                output.status,
                String::from_utf8_lossy(&output.stderr),
                String::from_utf8_lossy(&output.stdout)
            ),
        };
    }

    #[derive(serde::Deserialize)]
    struct VisualRubricCliVerdict {
        verdict: String,
        reason: String,
        #[serde(default)]
        anomalies: Vec<String>,
    }

    match serde_json::from_slice::<VisualRubricCliVerdict>(&output.stdout) {
        Ok(verdict) if verdict.verdict == "pass" => RubricAudit::Pass {
            reason: verdict.reason,
            anomalies: verdict.anomalies,
        },
        Ok(verdict) => RubricAudit::Fail {
            reason: verdict.reason,
            anomalies: verdict.anomalies,
        },
        Err(error) => RubricAudit::Error {
            message: format!(
                "failed to parse visual-rubric JSON from {:?}: {error}",
                String::from_utf8_lossy(&output.stdout)
            ),
        },
    }
}

fn create_clean_dir(path: &Path) -> Result<()> {
    if path.exists() {
        fs::remove_dir_all(path).with_context(|| format!("failed to clean {}", path.display()))?;
    }
    fs::create_dir_all(path).with_context(|| format!("failed to create {}", path.display()))
}

fn rendered_routes(site: &ProjectSite) -> Vec<String> {
    site.pages
        .iter()
        .map(|page| route_from_slug(&page.slug))
        .collect()
}

fn normalize_routes(routes: &[String]) -> Result<Vec<String>> {
    routes
        .iter()
        .map(|route| {
            ensure!(!route.trim().is_empty(), "audit route must not be empty");
            let trimmed = route.trim();
            let with_leading = if trimmed.starts_with('/') {
                trimmed.to_owned()
            } else {
                format!("/{trimmed}")
            };
            Ok(if with_leading == "/" || with_leading.ends_with('/') {
                with_leading
            } else {
                format!("{with_leading}/")
            })
        })
        .collect()
}

fn route_from_slug(slug: &str) -> String {
    if slug == "index" || slug.is_empty() {
        "/".into()
    } else {
        format!("/{}/", slug.trim_matches('/'))
    }
}

fn safe_route_name(route: &str) -> String {
    let normalized = route.trim_matches('/');
    if normalized.is_empty() {
        return "index".into();
    }
    normalized
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

fn rubric_blocks_gate(rubric: &RubricAudit) -> bool {
    matches!(rubric, RubricAudit::Fail { .. } | RubricAudit::Error { .. })
}

#[cfg(test)]
mod tests {
    use super::{normalize_routes, route_from_slug, safe_route_name};

    #[test]
    fn route_from_slug_matches_render_paths() {
        assert_eq!(route_from_slug("index"), "/");
        assert_eq!(route_from_slug("docs"), "/docs/");
        assert_eq!(route_from_slug("/nested/path/"), "/nested/path/");
    }

    #[test]
    fn explicit_routes_are_normalized_for_static_server_urls() {
        let routes = normalize_routes(&["/".into(), "docs".into(), "/guide/".into()]).unwrap();
        assert_eq!(routes, vec!["/", "/docs/", "/guide/"]);
    }

    #[test]
    fn route_names_are_stable_for_screenshot_directories() {
        assert_eq!(safe_route_name("/"), "index");
        assert_eq!(safe_route_name("/docs/"), "docs");
        assert_eq!(
            safe_route_name("/docs/api reference/"),
            "docs_api_reference"
        );
    }
}
