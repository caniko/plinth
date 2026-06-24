use crate::comfyui::{self, GeneratedImage};
use crate::config::Config;
use crate::format::{self, Article};
use crate::immich;
use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use inquire::{Confirm, Select, Text};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

/// Cluster name → workflow mapping
const CLUSTER_WORKFLOWS: &[(&str, &str)] = &[
    ("nix_infrastructure", "flux_schnell"),
    ("rust_tools", "flux_schnell"),
    ("publishing", "flux_schnell"),
    ("ai_neural", "flux_schnell"),
    ("gaming", "flux_schnell"),
];

pub async fn run(
    cfg: Config,
    articles_filter: Option<String>,
    count: u32,
    _workflow_override: Option<String>,
    output_dir: PathBuf,
    dry_run: bool,
) -> Result<()> {
    // Resolve articles directory
    let resolved_dir = if cfg.articles_dir.is_relative() {
        let cwd = std::env::current_dir()?;
        cwd.join(&cfg.articles_dir)
    } else {
        cfg.articles_dir.clone()
    };

    // 1. Print banner
    println!("\n  {} pcomfy — article image generator\n", console::style("◆").cyan());
    println!("  Articles directory: {}", console::style(resolved_dir.display()).cyan());

    // 2. Scan articles
    let mut all_articles = format::scan_articles(&resolved_dir)?;

    // Filter by specific slugs if provided
    if let Some(filter) = articles_filter {
        let slugs: Vec<&str> = filter.split(',').map(|s| s.trim()).collect();
        all_articles.retain(|a| slugs.contains(&a.slug.as_str()));
    }

    // Find articles without images
    let needs_images: Vec<&Article> = all_articles.iter().filter(|a| !a.has_images()).collect();
    let with_images = all_articles.iter().filter(|a| a.has_images()).count();

    // Skip count for articles with images
    let articles_to_process: Vec<&Article> = if needs_images.is_empty() {
        println!("  All {with_images} articles already have images. Nothing to do.");
        return Ok(());
    } else {
        println!(
            "  {} articles need images, {} already have them.\n",
            console::style(needs_images.len()).yellow(),
            with_images
        );
        needs_images
    };

    if dry_run {
        println!("\nArticles needing images ({}, dry run):", articles_to_process.len());
        for a in &articles_to_process {
            println!("  · {} — {}", a.slug, a.title);
        }
        return Ok(());
    }

    // 3. Probe ComfyUI
    println!("🔌 Proving ComfyUI...");
    comfyui::probe(&cfg.comfyui_url).await?;
    println!("  ✓ ComfyUI reachable at {}", console::style(&cfg.comfyui_url).cyan());

    // 4. Probe Immich
    if !cfg.immich_api_key.is_empty() {
        println!("🔌 Proving Immich...");
        match immich::probe(&cfg.immich_url).await {
            Ok(version) => println!("  ✓ Immich reachable (version {version})"),
            Err(e) => println!("  {} Immich: {e}", console::style("⚠").yellow()),
        }
    } else {
        println!("  {} No Immich API key configured — images won't be uploaded.",
            console::style("⚠").yellow());
    }

    // 5. Process each article
    let mut results: HashMap<String, Vec<String>> = HashMap::new();
    std::fs::create_dir_all(&output_dir)?;

    for (idx, article) in articles_to_process.iter().enumerate() {
        println!(
            "\n{} {}/{}: {} — {}",
            "─".repeat(50),
            console::style(idx + 1).cyan(),
            articles_to_process.len(),
            console::style(&article.slug).cyan().bold(),
            article.title
        );

        let proceed = Confirm::new(&format!(
            "Generate images for this article?"
        ))
        .with_default(true)
        .prompt()?;

        if !proceed {
            println!("  Skipped.");
            continue;
        }

        let cluster = format::detect_cluster(&article.tags);
        let prompt = format::prompt_for_article(article);
        let workflow_name = CLUSTER_WORKFLOWS
            .iter()
            .find(|(c, _)| *c == cluster)
            .map(|(_, w)| *w)
            .unwrap_or("flux_schnell");

        println!("  Cluster:  {}", console::style(cluster).cyan());
        println!("  Workflow: {}", console::style(workflow_name).cyan());
        println!("  Prompt:   {}", console::style(&prompt).dim());

        // Build a minimal workflow JSON with the prompt embedded.
        // In production, this would load a real workflow template and
        // inject the prompt + seed into the CLIPTextEncode node.
        let workflow = build_workflow(workflow_name, &prompt);

        // Generate images
        let article_dir = output_dir.join(&article.slug);
        std::fs::create_dir_all(&article_dir)?;

        let pb = ProgressBar::new(count as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} images")
                .unwrap()
                .progress_chars("█▓▒░"),
        );

        let mut generated_files: Vec<PathBuf> = Vec::new();

        for i in 0..count {
            // Vary the seed for each image in the batch
            let mut batch_workflow = workflow.clone();
            if let Some(map) = batch_workflow.as_object_mut() {
                if let Some(sampler) = map.get_mut("9") {
                    if let Some(inputs) = sampler.get_mut("inputs") {
                        if let Some(inputs_map) = inputs.as_object_mut() {
                            inputs_map.insert("seed".into(), serde_json::json!(42 + i as i64));
                        }
                    }
                }
            }

            match comfyui::submit_prompt(&cfg.comfyui_url, batch_workflow).await {
                Ok(response) => {
                    match comfyui::poll_history(&cfg.comfyui_url, &response.prompt_id, Duration::from_secs(2)).await
                    {
                        Ok(history) => {
                            let images = collect_images(&history.outputs);
                            for (ji, img) in images.iter().enumerate() {
                                let ext = PathBuf::from(&img.filename)
                                    .extension()
                                    .map(|e| e.to_string_lossy().to_string())
                                    .unwrap_or_else(|| "png".into());
                                let out_path = article_dir.join(format!("{}-{:02}.{ext}", article.slug, i * 4 + ji + 1));

                                match comfyui::download_image(
                                    &cfg.comfyui_url,
                                    &img.filename,
                                    &img.subfolder,
                                    &img.image_type,
                                )
                                .await
                                {
                                    Ok(bytes) => {
                                        std::fs::write(&out_path, &bytes)?;
                                        generated_files.push(out_path);
                                    }
                                    Err(e) => eprintln!("  Download error: {e}"),
                                }
                            }
                        }
                        Err(e) => eprintln!("  Poll error: {e}"),
                    }
                }
                Err(e) => eprintln!("  Prompt error: {e}"),
            }

            pb.inc(1);
        }

        pb.finish();

        if generated_files.is_empty() {
            println!("  No images generated.");
            continue;
        }

        // Show generated files
        println!("\n  Generated images:");
        for path in &generated_files {
            println!("    {}  {}",
                console::style("✓").green(),
                console::style(path.display()).dim()
            );
        }

        // Open for review
        #[cfg(target_os = "linux")]
        {
            let open_review = Confirm::new("Open images for review?")
                .with_default(true)
                .prompt()?;
            if open_review {
                for path in &generated_files {
                    let _ = open::that(path);
                }
            }
        }

        // Select which to keep
        let choices: Vec<String> = generated_files
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let filename = p.file_name().unwrap().to_string_lossy();
                format!("[{}] {filename}", i + 1)
            })
            .collect();

        let selection = Select::new("Keep which images?", choices.clone())
            .prompt().ok();

        let kept_files = match selection {
            Some(sel) => {
                // Parse the selected index
                if let Some(idx_str) = sel.trim_start_matches('[').split(']').next() {
                    if let Ok(n) = idx_str.parse::<usize>() {
                        if n > 0 && n <= generated_files.len() {
                            vec![generated_files[n - 1].clone()]
                        } else {
                            vec![]
                        }
                    } else {
                        vec![]
                    }
                } else {
                    generated_files.clone()
                }
            }
            None => generated_files.clone(),
        };

        if kept_files.is_empty() {
            println!("  No images kept. Moving on.");
            continue;
        }

        // Upload to Immich and collect URLs
        let mut image_urls: Vec<String> = Vec::new();

        if !cfg.immich_api_key.is_empty() {
            let upload_pb = ProgressBar::new(kept_files.len() as u64);
            upload_pb.set_style(
                ProgressStyle::default_bar()
                    .template("{spinner:.green} [{bar:40.cyan/blue}] uploading {pos}/{len}")
                    .unwrap()
                    .progress_chars("█▓▒░"),
            );

            for path in &kept_files {
                let bytes = std::fs::read(path)?;
                let filename = path.file_name().unwrap().to_string_lossy();

                match immich::upload_image(&cfg.immich_url, &cfg.immich_api_key, &bytes, &filename).await {
                    Ok(result) => {
                        let proxy = immich::proxy_url(&result.asset_id, result.width, result.height);
                        image_urls.push(proxy);
                    }
                    Err(e) => eprintln!("  Upload error: {e}"),
                }
                upload_pb.inc(1);
            }

            upload_pb.finish();
        }

        if image_urls.is_empty() && !kept_files.is_empty() {
            println!("\n  Images saved but not uploaded (no Immich API key).");
            println!("  Local files:");
            for path in &kept_files {
                println!("    {}", path.display());
            }
            continue;
        }

        // Print image URLs
        println!("\n  Image URLs:");
        for url in &image_urls {
            println!("    {}", console::style(url).cyan());
        }

        // Ask about placement
        let placement_options = vec![
            "hero — at top of article",
            "inline — after '# Title'",
            "gallery — at end of article",
            "skip — don't add to article yet",
        ];
        let placement_choice = Select::new("Placement:", placement_options.clone())
            .prompt()?;

        let placement = match placement_choice.chars().next() {
            Some('h') => "hero",
            Some('i') => "inline",
            Some('g') => "gallery",
            _ => "skip",
        };

        if placement != "skip" && !image_urls.is_empty() {
            results.insert(article.slug.clone(), image_urls.clone());
        }
    }

    // 6. Write image references to article files
    if results.is_empty() {
        println!("\nNo images to write.");
        return Ok(());
    }

    let write_files = Confirm::new("Write image references to article files?")
        .with_default(true)
        .prompt()?;

    if write_files {
        for (slug, urls) in &results {
            if let Some(url) = urls.first() {
                let article_path = resolved_dir.join(format!("{slug}.md"));
                if article_path.exists() {
                    format::write_hero_image(&article_path, url).with_context(|| {
                        format!("Failed to write hero image to {slug}")
                    })?;
                    println!("  ✓ {} — hero image added", console::style(slug).green());
                }
            }
        }
    }

    // 7. Summary
    println!("\n{}", "─".repeat(50));
    println!("Summary:");
    for (slug, urls) in &results {
        println!(
            "  {} → {} image(s)",
            console::style(slug).cyan(),
            urls.len()
        );
    }
    println!("\nRun `git diff` to review, then commit and deploy.");

    Ok(())
}

/// Build a minimal workflow JSON for the given workflow name and prompt.
fn build_workflow(workflow_name: &str, prompt: &str) -> serde_json::Value {
    // Minimal flux_schnell-like workflow — in production this would
    // load a real workflow template from the comfyui_workflow_templates
    // package and inject the prompt into the CLIPTextEncode node.
    serde_json::json!({
        "6": {
            "class_type": "CLIPTextEncode",
            "inputs": {
                "text": prompt,
                "clip": ["11", 0]
            }
        },
        "8": {
            "class_type": "EmptyLatentImage",
            "inputs": {
                "width": 1920,
                "height": 1080,
                "batch_size": 1
            }
        },
        "9": {
            "class_type": "KSampler",
            "inputs": {
                "seed": 42,
                "steps": 20,
                "cfg": 7.0,
                "sampler_name": "euler",
                "scheduler": "normal",
                "denoise": 1.0,
                "model": ["10", 0],
                "positive": ["6", 0],
                "negative": ["7", 0],
                "latent_image": ["8", 0]
            }
        },
        "10": {
            "class_type": "CheckpointLoaderSimple",
            "inputs": {
                "ckpt_name": "flux2_dev_fp8.safetensors"
            }
        },
        "7": {
            "class_type": "CLIPTextEncode",
            "inputs": {
                "text": "",
                "clip": ["11", 0]
            }
        },
        "11": {
            "class_type": "CLIPLoader",
            "inputs": {
                "clip_name": "t5xxl_fp16.safetensors",
                "type": "flux"
            }
        },
        "12": {
            "class_type": "VAELoader",
            "inputs": {
                "vae_name": "flux2-vae.safetensors"
            }
        },
        "13": {
            "class_type": "VAEDecode",
            "inputs": {
                "samples": ["9", 0],
                "vae": ["12", 0]
            }
        },
        "14": {
            "class_type": "SaveImage",
            "inputs": {
                "filename_prefix": "pcomfy",
                "images": ["13", 0]
            }
        }
    })
}

/// Collect images from ComfyUI history outputs.
fn collect_images(outputs: &std::collections::HashMap<String, comfyui::HistoryOutput>) -> Vec<GeneratedImage> {
    let mut images = Vec::new();
    for output in outputs.values() {
        if let Some(imgs) = &output.images {
            images.extend(imgs.clone());
        }
        if let Some(gifs) = &output.gifs {
            images.extend(gifs.clone());
        }
    }
    images
}
