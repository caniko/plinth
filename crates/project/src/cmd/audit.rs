use crate::{
    AuditInstallArgs, RubricAudit, ScreenshotAudit, SiteAuditReport, VIEWPORTS, Viewport,
    WEBSITE_RUBRIC_PRESET, resolve_config_path, write_json_report,
};
use anyhow::bail;
use anyhow::{Context, Result};
use plinth_project::{
    ProjectSection, ProjectSite, RenderOptions, dev::StaticServer, install_ux_report,
    load_project_site, render_install_fragment, render_static,
};
use std::fs;
use std::path::Path;
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
            run_visual_rubric(&args.rubric_bin, &path)
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

fn run_visual_rubric(rubric_bin: &Path, image: &Path) -> RubricAudit {
    let output = Command::new(rubric_bin)
        .arg("configured")
        .arg("--json")
        .arg("--image")
        .arg(image)
        .arg("--preset")
        .arg(WEBSITE_RUBRIC_PRESET)
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
