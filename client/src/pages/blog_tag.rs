use leptos::*;
use leptos_meta::*;
use leptos_router::*;

#[cfg(feature = "ssr")]
use server::server_fns::get_blog_posts_by_tag;

#[cfg(not(feature = "ssr"))]
use leptos::*;

#[component]
pub fn BlogTagPage() -> impl IntoView {
    let params = use_params_map();
    let tag = move || params.with(|p| p.get("tag").cloned().unwrap_or_default());

    // Load blog posts filtered by tag via server function
    let blog_posts = create_resource(
        tag,
        |tag| async move {
            #[cfg(feature = "ssr")]
            { get_blog_posts_by_tag(tag).await }

            #[cfg(not(feature = "ssr"))]
            { server::server_fns::get_blog_posts_by_tag(tag).await }
        },
    );

    view! {
        <Title text={move || format!("Posts tagged '{}' - Blog", tag())}/>
        <Meta name="description" content={move || format!("All blog posts tagged with {}", tag())}/>

        <div class="min-h-screen bg-gray-50 dark:bg-gray-900">
            <div class="container mx-auto px-4 py-16">
                // Back to blog link
                <a href="/blog" class="inline-flex items-center text-blue-600 dark:text-blue-400 hover:underline mb-8">
                    "← Back to All Posts"
                </a>

                // Header
                <div class="mb-12">
                    <h1 class="text-5xl font-bold mb-4 text-gray-900 dark:text-white">
                        "Posts tagged: "
                        <span class="text-blue-600 dark:text-blue-400">{tag}</span>
                    </h1>
                    <div class="h-1 w-20 bg-blue-600 rounded"></div>
                </div>

                // Posts listing
                <Suspense fallback=move || view! {
                    <div class="text-center py-12">
                        <p class="text-gray-600 dark:text-gray-400">"Loading posts..."</p>
                    </div>
                }>
                    {move || {
                        blog_posts.get().map(|result| {
                            match result {
                                Ok(posts) => {
                                    if posts.is_empty() {
                                        view! {
                                            <div class="text-center py-12">
                                                <p class="text-gray-600 dark:text-gray-400 mb-4">
                                                    {format!("No posts found with tag '{}'", tag())}
                                                </p>
                                                <a href="/blog" class="btn-primary">
                                                    "View All Posts"
                                                </a>
                                            </div>
                                        }.into_view()
                                    } else {
                                        view! {
                                            <div>
                                                <p class="text-gray-600 dark:text-gray-400 mb-8">
                                                    {format!("Found {} post{}", posts.len(), if posts.len() == 1 { "" } else { "s" })}
                                                </p>
                                                <div class="grid grid-cols-1 gap-8">
                                                    {posts.into_iter().map(|post| {
                                                        let slug = post.slug.clone();
                                                        view! {
                                                            <article class="card card-dark hover:shadow-xl transition-shadow">
                                                                <a href={format!("/blog/{}", slug)} class="block">
                                                                    <h2 class="text-3xl font-bold mb-3 text-gray-900 dark:text-white hover:text-blue-600 dark:hover:text-blue-400 transition-colors">
                                                                        {post.title.clone()}
                                                                    </h2>

                                                                    <div class="flex flex-wrap items-center gap-4 text-sm text-gray-600 dark:text-gray-400 mb-4">
                                                                        <time>{post.published_at.format("%B %d, %Y").to_string()}</time>
                                                                        <span>"•"</span>
                                                                        <span>{post.reading_time_minutes} " min read"</span>
                                                                        <span>"•"</span>
                                                                        <span>{post.author.clone()}</span>
                                                                    </div>

                                                                    <p class="text-gray-700 dark:text-gray-300 mb-4 line-clamp-3">
                                                                        {post.description.clone()}
                                                                    </p>

                                                                    <div class="flex flex-wrap gap-2">
                                                                        {post.tags.iter().map(|t| view! {
                                                                            <span class="px-3 py-1 bg-blue-100 dark:bg-blue-900 text-blue-800 dark:text-blue-200 rounded-full text-sm">
                                                                                {t.clone()}
                                                                            </span>
                                                                        }).collect_view()}
                                                                    </div>
                                                                </a>
                                                            </article>
                                                        }
                                                    }).collect_view()}
                                                </div>
                                            </div>
                                        }.into_view()
                                    }
                                }
                                Err(e) => view! {
                                    <div class="text-center py-12">
                                        <p class="text-red-600 dark:text-red-400">
                                            "Error loading posts: " {e.to_string()}
                                        </p>
                                    </div>
                                }.into_view()
                            }
                        })
                    }}
                </Suspense>
            </div>
        </div>
    }
}
