use crate::InitArgs;
use anyhow::{Context, Result, anyhow};
use std::fs;

pub fn init_site(args: &InitArgs) -> Result<()> {
    if args.out_config.exists() && !args.force {
        return Err(anyhow!(
            "{} already exists; pass --force to replace it",
            args.out_config.display()
        ));
    }
    if let Some(parent) = args.out_config.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let template = initial_config_template(&args.title, &args.description);
    fs::write(&args.out_config, template)
        .with_context(|| format!("failed to write {}", args.out_config.display()))?;
    println!("Created {}", args.out_config.display());
    Ok(())
}

pub fn initial_config_template(title: &str, description: &str) -> String {
    format!(
        r##"[site]
title = "{}"
description = "{}"
base_url = "/"
footer_note = "{}"

[theme]
# Select a preset palette: gruvbox-hard-dark, catppuccin-latte,
# catppuccin-frappe, catppuccin-macchiato, or catppuccin-mocha.
# preset = "catppuccin-latte"
# Override individual preset colours as needed:
# paper = "#eff1f5"

[[nav]]
label = "Home"
href = "/"

[[pages]]
slug = "index"
title = "{}"
description = "{}"

[[pages.sections]]
type = "hero"
title = "{}"
tagline = "{}"
subtitle = "{}"
"##,
        toml_escape(title),
        toml_escape(description),
        toml_escape(description),
        toml_escape(title),
        toml_escape(description),
        toml_escape(title),
        toml_escape(description),
        toml_escape(description)
    )
}

fn toml_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}
