use crate::{
    AssetJson, InspectArgs, InspectJson, PageJson, PersonJson, ProjectJson, emit_json,
    resolve_config_path,
};
use anyhow::{Context, Result};
use plinth_project::{Asset, ProjectSection, ProjectSite, load_project_site, project_watch_paths};
use std::path::{Path, PathBuf};

pub fn inspect_site(args: &InspectArgs) -> Result<()> {
    let config = resolve_config_path(args.config.config.clone())?;
    let site = load_project_site(&config).context("load project site")?;
    let watch_paths = project_watch_paths(&config).context("load project watch paths")?;

    if args.json.json {
        emit_json(&inspect_json(&config, &site, watch_paths))?;
    } else {
        print_inspection(&config, &site, &watch_paths);
    }
    Ok(())
}

fn inspect_json<'a>(
    config: &'a Path,
    site: &'a ProjectSite,
    watch_paths: Vec<PathBuf>,
) -> InspectJson<'a> {
    InspectJson {
        config,
        title: &site.title,
        description: &site.description,
        base_url: &site.base_url,
        pages: site
            .pages
            .iter()
            .map(|page| PageJson {
                slug: &page.slug,
                title: &page.title,
                description: &page.description,
                sections: page.sections.iter().map(section_name).collect(),
            })
            .collect(),
        assets: site.assets.iter().map(asset_json).collect(),
        people: site
            .people
            .iter()
            .map(|person| PersonJson {
                id: &person.id,
                name: &person.name,
                url: &person.url,
            })
            .collect(),
        projects: site
            .projects
            .iter()
            .map(|project| ProjectJson {
                title: &project.title,
                url: &project.url,
                source_url: project.source_url.as_deref(),
                demo_url: project.demo_url.as_deref(),
            })
            .collect(),
        watch_paths,
    }
}

fn print_inspection(config: &Path, site: &ProjectSite, watch_paths: &[PathBuf]) {
    println!("Config: {}", config.display());
    println!("Title: {}", site.title);
    println!("Description: {}", site.description);
    println!("Base URL: {}", site.base_url);
    println!("Pages:");
    for page in &site.pages {
        let sections = page
            .sections
            .iter()
            .map(section_name)
            .collect::<Vec<_>>()
            .join(", ");
        println!("  - {} ({}) [{}]", page.title, page.slug, sections);
    }
    println!("Assets:");
    if site.assets.is_empty() {
        println!("  - none");
    } else {
        for asset in &site.assets {
            println!("  - {} -> {}", asset.source.display(), asset.target);
        }
    }
    println!("People:");
    if site.people.is_empty() {
        println!("  - none");
    } else {
        for person in &site.people {
            println!("  - {} ({})", person.name, person.id);
        }
    }
    println!("Projects:");
    if site.projects.is_empty() {
        println!("  - none");
    } else {
        for project in &site.projects {
            println!("  - {} ({})", project.title, project.url);
        }
    }
    println!("Watch paths:");
    for path in watch_paths {
        println!("  - {}", path.display());
    }
}

fn asset_json(asset: &Asset) -> AssetJson<'_> {
    AssetJson {
        source: &asset.source,
        target: &asset.target,
    }
}

fn section_name(section: &ProjectSection) -> &'static str {
    match section {
        #[cfg(feature = "brick-hero")]
        ProjectSection::Hero(_) => "hero",
        #[cfg(feature = "brick-feature-grid")]
        ProjectSection::FeatureGrid(_) => "feature_grid",
        #[cfg(feature = "brick-install")]
        ProjectSection::Install(_) => "install",
        #[cfg(feature = "brick-person-mention")]
        ProjectSection::PersonMention(_) => "person_mention",
        #[cfg(feature = "brick-workflow-steps")]
        ProjectSection::WorkflowSteps(_) => "workflow_steps",
        #[cfg(feature = "brick-audience-grid")]
        ProjectSection::AudienceGrid(_) => "audience_grid",
        #[cfg(feature = "brick-trust-panel")]
        ProjectSection::TrustPanel(_) => "trust_panel",
        #[cfg(feature = "brick-screenshot-grid")]
        ProjectSection::ScreenshotGrid(_) => "screenshot_grid",
        #[cfg(feature = "brick-capability-matrix")]
        ProjectSection::CapabilityMatrix(_) => "capability_matrix",
        #[cfg(feature = "brick-comparison")]
        ProjectSection::Comparison(_) => "comparison",
        #[cfg(feature = "brick-content")]
        ProjectSection::Content(_) => "content",
        #[cfg(feature = "brick-custom")]
        ProjectSection::Custom(_) => "custom",
    }
}
