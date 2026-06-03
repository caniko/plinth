use leptos::either::EitherOf3;
use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::hooks::use_params_map;

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

#[component]
pub fn ActivityDetailPage() -> impl IntoView {
    let params = use_params_map();
    let id = move || {
        params
            .with(|p| p.get("id").and_then(|s| s.parse::<i64>().ok()))
            .unwrap_or(0)
    };

    let item = Resource::new(
        id,
        |id| async move { api::get_activity_item_by_id(id).await },
    );

    view! {
        <Suspense fallback=move || view! {
            <div class="min-h-screen bg-gray-50 dark:bg-black">
                <div class="container mx-auto px-4 py-16 max-w-4xl text-center">
                    <p class="text-gray-600 dark:text-amber-400">"Loading contribution..."</p>
                </div>
            </div>
        }>
            {move || {
                item.get().map(|result| {
                    match result {
                        Ok(Some(item)) => {
                            let config = use_site_config();
                            let repo = format!("{}/{}", item.repo_owner, item.repo_name);
                            let ref_date = item.merged_at
                                .or(item.closed_at)
                                .unwrap_or(item.created_at);
                            EitherOf3::A(view! {
                                <Title text={format!("{} - {}", item.title, config.name)}/>
                                <Meta name="description" content={item.title.clone()}/>

                                <div class="min-h-screen bg-gray-50 dark:bg-black">
                                    <article class="container mx-auto px-4 py-16 max-w-4xl">
                                        <a href="/activity" class="inline-flex items-center text-blue-600 dark:text-amber-300 hover:underline mb-8">
                                            "\u{2190} Back to Activity"
                                        </a>

                                        <header class="mb-8">
                                            <div class="flex flex-wrap items-center gap-3 mb-4 text-sm text-gray-600 dark:text-amber-400">
                                                <span class="px-3 py-1 bg-blue-100 dark:bg-amber-900/30 text-blue-800 dark:text-amber-200 rounded-full">
                                                    {forge_label(&item.forge)}
                                                </span>
                                                <span class="px-3 py-1 bg-blue-100 dark:bg-amber-900/30 text-blue-800 dark:text-amber-200 rounded-full">
                                                    {state_label(&item.state)}
                                                </span>
                                                <span class="px-3 py-1 bg-yellow-100 dark:bg-yellow-900/40 text-yellow-800 dark:text-yellow-200 rounded-full">
                                                    "Impact " {item.impact}
                                                </span>
                                            </div>

                                            <h1 class="text-4xl font-bold mb-4 text-gray-900 dark:text-amber-100 leading-tight">
                                                {item.title.clone()}
                                            </h1>

                                            <p class="text-lg text-gray-600 dark:text-amber-400 mb-2">
                                                {repo} " #" {item.number}
                                            </p>
                                            <p class="text-gray-500 dark:text-amber-400 mb-6">
                                                {ref_date.format("%B %e, %Y").to_string()}
                                            </p>

                                            <a
                                                href={item.url.clone()}
                                                target="_blank"
                                                rel="noopener noreferrer"
                                                class="btn-primary inline-flex items-center gap-2"
                                            >
                                                "View on " {forge_label(&item.forge)} " \u{2197}"
                                            </a>
                                        </header>

                                        {item.body.as_ref().map(|body| view! {
                                            <div class="prose prose-lg dark:prose-invert max-w-none bg-white dark:bg-black rounded-lg shadow-lg p-8 mb-12">
                                                <p class="whitespace-pre-wrap">{body.clone()}</p>
                                            </div>
                                        })}

                                        <footer class="mt-8 pt-8 border-t border-gray-200 dark:border-amber-900/50">
                                            <a href="/activity" class="btn-secondary">
                                                "\u{2190} All Activity"
                                            </a>
                                        </footer>
                                    </article>
                                </div>
                            })
                        },
                        Ok(None) => EitherOf3::B(view! {
                            <Title text="Contribution Not Found"/>
                            <div class="min-h-screen bg-gray-50 dark:bg-black">
                                <div class="container mx-auto px-4 py-16 max-w-4xl text-center">
                                    <h1 class="text-4xl font-bold mb-4 text-gray-900 dark:text-amber-100">
                                        "Contribution Not Found"
                                    </h1>
                                    <a href="/activity" class="btn-primary">"View All Activity"</a>
                                </div>
                            </div>
                        }),
                        Err(_) => EitherOf3::C(view! { <ErrorMessage/> }),
                    }
                })
            }}
        </Suspense>
    }
}
