use leptos::either::EitherOf3;
use leptos::prelude::*;
use leptos_meta::*;

use crate::api;
use crate::app::use_site_config;
use crate::components::ErrorMessage;

#[component]
pub fn SeriesListPage() -> impl IntoView {
    let config = use_site_config();
    let title_text = format!("Series - {}", config.name);

    let all_series = Resource::new(|| (), |_| async move { api::get_all_series().await });

    view! {
        <Title text={title_text}/>
        <Meta name="description" content="Browse all blog series"/>

        <div class="min-h-screen bg-gray-50 dark:bg-black">
            <div class="container mx-auto px-4 py-16 max-w-5xl">
                <div class="mb-12">
                    <h1 class="text-5xl font-bold mb-4 text-gray-900 dark:text-amber-100">
                        "Series"
                    </h1>
                    <p class="text-xl text-gray-600 dark:text-amber-400">
                        "Multi-part deep dives and recurring topics"
                    </p>
                    <div class="h-1 w-20 bg-blue-600 rounded mt-4"></div>
                </div>

                <Suspense fallback=move || view! {
                    <div class="text-center py-12">
                        <p class="text-gray-600 dark:text-amber-400">"Loading series..."</p>
                    </div>
                }>
                    {move || {
                        all_series.get().map(|result| {
                            match result {
                                Ok(series_list) if !series_list.is_empty() => {
                                    EitherOf3::A(view! {
                                        <div class="space-y-6">
                                            {series_list.into_iter().map(|series| {
                                                view! {
                                                    <a
                                                        href={format!("/series/{}", series.slug)}
                                                        class="block bg-white dark:bg-black rounded-lg shadow-md hover:shadow-lg transition-all p-6"
                                                    >
                                                        <h3 class="text-2xl font-bold text-gray-900 dark:text-amber-100 mb-2">
                                                            {series.title}
                                                        </h3>
                                                        <div class="flex items-center gap-4 text-sm text-gray-500 dark:text-amber-600">
                                                            <span>{series.post_count} " parts"</span>
                                                            <span>"\u{2022}"</span>
                                                            <span>{series.total_reading_time} " min total"</span>
                                                            {series.latest_published_at.map(|date| view! {
                                                                <span>"\u{2022}"</span>
                                                                <span>"Latest: " {date.format("%b %d, %Y").to_string()}</span>
                                                            })}
                                                        </div>
                                                    </a>
                                                }
                                            }).collect::<Vec<_>>()}
                                        </div>
                                    })
                                },
                                Ok(_) => EitherOf3::B(view! {
                                    <div class="text-center py-12">
                                        <p class="text-gray-600 dark:text-amber-400">
                                            "No series yet. Check back soon!"
                                        </p>
                                    </div>
                                }),
                                Err(_) => EitherOf3::C(view! {
                                    <ErrorMessage/>
                                }),
                            }
                        })
                    }}
                </Suspense>
            </div>
        </div>
    }
}
