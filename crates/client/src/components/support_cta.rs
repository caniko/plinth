use leptos::prelude::*;

use crate::app::use_site_config;

/// Platform icon component — returns the appropriate SVG for each donation platform
#[component]
fn PlatformIcon(platform: String) -> impl IntoView {
    match platform.as_str() {
        "kofi" => view! {
            // Coffee cup icon
            <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 24 24">
                <path d="M20 3H4v10a4 4 0 004 4h6a4 4 0 004-4v-1h2a2 2 0 002-2V5a2 2 0 00-2-2zm0 7h-2V5h2v5zM2 21h18v2H2v-2z"/>
            </svg>
        }.into_any(),
        "github_sponsors" => view! {
            // Heart icon (GitHub Sponsors style)
            <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 24 24">
                <path d="M12 21.35l-1.45-1.32C5.4 15.36 2 12.28 2 8.5 2 5.42 4.42 3 7.5 3c1.74 0 3.41.81 4.5 2.09C13.09 3.81 14.76 3 16.5 3 19.58 3 22 5.42 22 8.5c0 3.78-3.4 6.86-8.55 11.54L12 21.35z"/>
            </svg>
        }.into_any(),
        "liberapay" => view! {
            // Gift/recurring icon
            <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 24 24">
                <path d="M20 6h-2.18c.11-.31.18-.65.18-1a2.996 2.996 0 00-5.5-1.65l-.5.67-.5-.68C10.96 2.54 10.05 2 9 2 7.34 2 6 3.34 6 5c0 .35.07.69.18 1H4c-1.11 0-1.99.89-1.99 2L2 19c0 1.11.89 2 2 2h16c1.11 0 2-.89 2-2V8c0-1.11-.89-2-2-2zm-5-2c.55 0 1 .45 1 1s-.45 1-1 1-1-.45-1-1 .45-1 1-1zM9 4c.55 0 1 .45 1 1s-.45 1-1 1-1-.45-1-1 .45-1 1-1zm11 15H4v-2h16v2zm0-5H4V8h5.08L7 10.83 8.62 12 12 7.4l3.38 4.6L17 10.83 14.92 8H20v6z"/>
            </svg>
        }.into_any(),
        _ => view! {
            // Generic heart icon for custom platforms
            <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 24 24">
                <path d="M12 21.35l-1.45-1.32C5.4 15.36 2 12.28 2 8.5 2 5.42 4.42 3 7.5 3c1.74 0 3.41.81 4.5 2.09C13.09 3.81 14.76 3 16.5 3 19.58 3 22 5.42 22 8.5c0 3.78-3.4 6.86-8.55 11.54L12 21.35z"/>
            </svg>
        }.into_any(),
    }
}

/// Returns a human-readable label for a donation platform
fn platform_label(platform: &str, custom_label: &str) -> String {
    if !custom_label.is_empty() {
        return custom_label.to_string();
    }
    match platform {
        "kofi" => "Ko-fi".to_string(),
        "github_sponsors" => "Sponsor".to_string(),
        "liberapay" => "Liberapay".to_string(),
        _ => "Support".to_string(),
    }
}

/// Compact end-of-article CTA for donations.
/// Shows nothing if donations are disabled or no links are configured.
#[component]
pub fn SupportCta() -> impl IntoView {
    let config = use_site_config();
    let donation = config.donation;

    if !donation.enabled || donation.links.is_empty() {
        return None;
    }

    let cta_text = if donation.cta_text.is_empty() {
        "If you found this useful, consider supporting my work.".to_string()
    } else {
        donation.cta_text.clone()
    };

    // Show up to 2 platform buttons inline
    let links: Vec<_> = donation.links.into_iter().take(2).collect();

    Some(view! {
        <div class="mt-10 p-6 rounded-lg border border-gray-200 dark:border-amber-900/30 bg-white/50 dark:bg-amber-950/10">
            <p class="text-gray-600 dark:text-amber-400 mb-4">
                {cta_text}
            </p>
            <div class="flex flex-wrap gap-3">
                {links.into_iter().map(|link| {
                    let label = platform_label(&link.platform, &link.label);
                    let platform = link.platform.clone();
                    view! {
                        <a
                            href={link.url}
                            target="_blank"
                            rel="noopener noreferrer"
                            class="inline-flex items-center gap-2 px-4 py-2 rounded-lg text-sm font-medium bg-blue-50 dark:bg-amber-900/20 text-blue-700 dark:text-amber-200 hover:bg-blue-100 dark:hover:bg-amber-900/40 transition-colors"
                        >
                            <PlatformIcon platform={platform}/>
                            {label}
                        </a>
                    }
                }).collect::<Vec<_>>()}
            </div>
        </div>
    })
}
