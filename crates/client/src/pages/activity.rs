use leptos::either::EitherOf3;
use leptos::prelude::*;
use leptos_meta::*;

use crate::api;
use crate::app::use_site_config;
use crate::components::ErrorMessage;

fn forge_label(f: &plinth_shared::Forge) -> &'static str {
    match f {
        plinth_shared::Forge::GitHub => "GitHub",
        plinth_shared::Forge::Codeberg => "Codeberg",
    }
}

fn state_label(s: &plinth_shared::ActivityState) -> &'static str {
    match s {
        plinth_shared::ActivityState::Merged => "Merged",
        plinth_shared::ActivityState::Closed => "Closed",
        plinth_shared::ActivityState::Open => "Open",
    }
}

/// Activity listing page — curated external contributions (PRs, issues)
/// across GitHub and Codeberg, ranked by impact and recency.
#[component]
pub fn ActivityPage() -> impl IntoView {
    let config = use_site_config();
    let page_title = "Activity".to_string();
    let title_text = format!("{} - {}", page_title, config.name);

    let canonical_url = if config.base_url.is_empty() {
        "/activity".to_string()
    } else {
        format!("{}/activity", config.base_url)
    };

    let items = Resource::new(|| (), |_| async move { api::get_activity_list().await });

    view! {
        <Title text={title_text}/>
        <Meta name="description" content="Curated external contributions across GitHub and Codeberg, ranked by impact and recency."/>
        <Link rel="canonical" href={canonical_url}/>

        <div class="min-h-screen bg-gray-50 dark:bg-black">
            <div class="container mx-auto px-4 py-16">
                <div class="mb-12">
                    <h1 class="text-5xl font-bold mb-4 text-gray-900 dark:text-amber-100">
                        {page_title}
                    </h1>
                    <p class="text-xl text-gray-600 dark:text-amber-400">
                        "Contributions I have landed on other people\u{2019}s projects."
                    </p>
                    <div class="h-1 w-20 bg-blue-600 rounded mt-4"></div>
                </div>

                <Suspense fallback=move || view! {
                    <div class="text-center py-12">
                        <p class="text-gray-600 dark:text-amber-400">"Loading activity..."</p>
                    </div>
                }>
                    {move || {
                        items.get().map(|result| {
                            match result {
                                Ok(items) => {
                                    if items.is_empty() {
                                        EitherOf3::A(view! {
                                            <div class="text-center py-12">
                                                <p class="text-gray-600 dark:text-amber-400">
                                                    "No activity yet. Check back soon!"
                                                </p>
                                            </div>
                                        })
                                    } else {
                                        EitherOf3::B(view! {
                                            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8">
                                                {items.into_iter().map(|item| {
                                                    let id = item.id;
                                                    let repo = format!("{}/{}", item.repo_owner, item.repo_name);
                                                    let ref_date = item.reference_date();
                                                    view! {
                                                        <a
                                                            href={format!("/activity/{}", id)}
                                                            class="card card-dark block hover:scale-105 transition-transform"
                                                        >
                                                            <div class="p-6">
                                                                <div class="flex items-center justify-between mb-2 text-sm text-gray-500 dark:text-amber-400">
                                                                    <span>{forge_label(&item.forge)}</span>
                                                                    <span class="px-2 py-0.5 rounded-full bg-blue-100 dark:bg-amber-900/30 text-blue-800 dark:text-amber-200">
                                                                        {state_label(&item.state)}
                                                                    </span>
                                                                </div>
                                                                <h2 class="text-2xl font-bold mb-2 text-gray-900 dark:text-amber-100">
                                                                    {item.title.clone()}
                                                                </h2>
                                                                <p class="text-gray-600 dark:text-amber-400 mb-4">
                                                                    {repo} " #" {item.number}
                                                                </p>
                                                                <div class="flex items-center justify-between text-sm">
                                                                    <span class="text-gray-500 dark:text-amber-400">
                                                                        {ref_date.format("%b %Y").to_string()}
                                                                    </span>
                                                                    <span class="px-3 py-1 bg-yellow-100 dark:bg-yellow-900/40 text-yellow-800 dark:text-yellow-200 rounded-full">
                                                                        "Impact " {item.impact}
                                                                    </span>
                                                                </div>
                                                            </div>
                                                        </a>
                                                    }
                                                }).collect::<Vec<_>>()}
                                            </div>
                                        })
                                    }
                                },
                                Err(_) => EitherOf3::C(view! { <ErrorMessage/> }),
                            }
                        })
                    }}
                </Suspense>
            </div>
        </div>
    }
}
