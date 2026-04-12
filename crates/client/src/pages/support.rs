use leptos::either::Either;
use leptos::prelude::*;
use leptos_meta::*;

use crate::api;
use crate::app::use_site_config;

/// Returns a description for a donation platform
fn platform_description(platform: &str) -> &'static str {
    match platform {
        "kofi" => "One-time support — buy me a coffee and help keep this site going.",
        "github_sponsors" => {
            "Monthly sponsorship through GitHub — supports ongoing open-source work."
        }
        "liberapay" => {
            "Recurring donations through Liberapay — an open-source, privacy-friendly platform."
        }
        _ => "Support my work through this platform.",
    }
}

/// Returns an SVG icon for a donation platform (larger version for cards)
#[component]
fn PlatformCardIcon(platform: String) -> impl IntoView {
    match platform.as_str() {
        "kofi" => view! {
            <svg class="w-8 h-8" fill="currentColor" viewBox="0 0 24 24">
                <path d="M20 3H4v10a4 4 0 004 4h6a4 4 0 004-4v-1h2a2 2 0 002-2V5a2 2 0 00-2-2zm0 7h-2V5h2v5zM2 21h18v2H2v-2z"/>
            </svg>
        }.into_any(),
        "github_sponsors" => view! {
            <svg class="w-8 h-8" fill="currentColor" viewBox="0 0 24 24">
                <path d="M12 21.35l-1.45-1.32C5.4 15.36 2 12.28 2 8.5 2 5.42 4.42 3 7.5 3c1.74 0 3.41.81 4.5 2.09C13.09 3.81 14.76 3 16.5 3 19.58 3 22 5.42 22 8.5c0 3.78-3.4 6.86-8.55 11.54L12 21.35z"/>
            </svg>
        }.into_any(),
        "liberapay" => view! {
            <svg class="w-8 h-8" fill="currentColor" viewBox="0 0 24 24">
                <path d="M20 6h-2.18c.11-.31.18-.65.18-1a2.996 2.996 0 00-5.5-1.65l-.5.67-.5-.68C10.96 2.54 10.05 2 9 2 7.34 2 6 3.34 6 5c0 .35.07.69.18 1H4c-1.11 0-1.99.89-1.99 2L2 19c0 1.11.89 2 2 2h16c1.11 0 2-.89 2-2V8c0-1.11-.89-2-2-2zm-5-2c.55 0 1 .45 1 1s-.45 1-1 1-1-.45-1-1 .45-1 1-1zM9 4c.55 0 1 .45 1 1s-.45 1-1 1-1-.45-1-1 .45-1 1-1zm11 15H4v-2h16v2zm0-5H4V8h5.08L7 10.83 8.62 12 12 7.4l3.38 4.6L17 10.83 14.92 8H20v6z"/>
            </svg>
        }.into_any(),
        _ => view! {
            <svg class="w-8 h-8" fill="currentColor" viewBox="0 0 24 24">
                <path d="M12 21.35l-1.45-1.32C5.4 15.36 2 12.28 2 8.5 2 5.42 4.42 3 7.5 3c1.74 0 3.41.81 4.5 2.09C13.09 3.81 14.76 3 16.5 3 19.58 3 22 5.42 22 8.5c0 3.78-3.4 6.86-8.55 11.54L12 21.35z"/>
            </svg>
        }.into_any(),
    }
}

/// Returns a human-readable name for a platform
fn platform_name(platform: &str, custom_label: &str) -> String {
    if !custom_label.is_empty() {
        return custom_label.to_string();
    }
    match platform {
        "kofi" => "Ko-fi".to_string(),
        "github_sponsors" => "GitHub Sponsors".to_string(),
        "liberapay" => "Liberapay".to_string(),
        _ => "Support".to_string(),
    }
}

#[component]
pub fn SupportPage() -> impl IntoView {
    let config = use_site_config();
    let title_text = format!("Support - {}", config.name);
    let donation = config.donation.clone();

    let support_content = Resource::new(
        || (),
        |_| async move { api::get_site_content("support".to_string()).await },
    );

    view! {
        <Title text={title_text}/>
        <Meta name="description" content="Support my work — ways to contribute and help keep this site going"/>

        <div class="min-h-screen bg-gray-50 dark:bg-black">
            <div class="container mx-auto px-4 py-16 max-w-4xl">
                // Header
                <div class="mb-12">
                    <h1 class="text-5xl font-bold mb-4 text-gray-900 dark:text-amber-100">
                        "Support"
                    </h1>
                    <div class="h-1 w-20 bg-blue-600 rounded"></div>
                </div>

                // Custom content from site_content "support" key
                <Suspense fallback=|| ()>
                    {move || {
                        support_content.get().map(|result| {
                            match result {
                                Ok(Some(content)) => Either::Left(view! {
                                    <div class="prose prose-lg dark:prose-invert max-w-none mb-12"
                                         inner_html={content.html_content}>
                                    </div>
                                }),
                                _ => Either::Right(view! {
                                    <p class="text-lg text-gray-600 dark:text-amber-400 mb-12">
                                        "This site is free, ad-free, and open. If my work has been useful to you, consider supporting it through one of the platforms below."
                                    </p>
                                }),
                            }
                        })
                    }}
                </Suspense>

                // Donation platform cards
                {if donation.enabled && !donation.links.is_empty() {
                    Either::Left(view! {
                        <div class="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
                            {donation.links.iter().map(|link| {
                                let name = platform_name(&link.platform, &link.label);
                                let description = platform_description(&link.platform).to_string();
                                let url = link.url.clone();
                                let platform = link.platform.clone();
                                view! {
                                    <a
                                        href={url}
                                        target="_blank"
                                        rel="noopener noreferrer"
                                        class="block bg-white dark:bg-black rounded-lg shadow-lg p-6 border border-gray-200 dark:border-amber-900/30 hover:border-blue-400 dark:hover:border-amber-500 hover:shadow-xl transition-all group"
                                    >
                                        <div class="text-blue-600 dark:text-amber-300 mb-4 group-hover:text-blue-700 dark:group-hover:text-amber-200 transition-colors">
                                            <PlatformCardIcon platform={platform}/>
                                        </div>
                                        <h2 class="text-xl font-semibold mb-2 text-gray-900 dark:text-amber-100">
                                            {name}
                                        </h2>
                                        <p class="text-gray-600 dark:text-amber-400 text-sm">
                                            {description}
                                        </p>
                                    </a>
                                }
                            }).collect::<Vec<_>>()}
                        </div>
                    })
                } else {
                    Either::Right(view! {
                        <p class="text-gray-500 dark:text-amber-600">
                            "No donation links are configured."
                        </p>
                    })
                }}
            </div>
        </div>
    }
}
