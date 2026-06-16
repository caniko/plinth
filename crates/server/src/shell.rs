use leptos::prelude::*;
use leptos_meta::MetaTags;
use plinth_client::App;

/// Shell function for Leptos SSR rendering.
/// Context (SiteConfig, database pool) is provided by leptos_routes_with_context.
pub fn shell(
    options: LeptosOptions,
    lang: String,
    default_theme: String,
    plausible_domain: String,
    plausible_script_url: String,
) -> impl IntoView {
    let theme_class = if default_theme == "light" { "" } else { "dark" };
    let theme_script = format!(
        "var t=localStorage.getItem('theme');if(t==='light'){{document.documentElement.classList.remove('dark')}}else if(!t&&'{}' === 'light'){{document.documentElement.classList.remove('dark')}};",
        default_theme
    );

    let plausible_enabled = !plausible_domain.is_empty() && !plausible_script_url.is_empty();

    view! {
        <!DOCTYPE html>
        <html lang={lang} class={theme_class}>
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <meta name="color-scheme" content="light dark"/>
                <meta name="darkreader-lock"/>
                <script>{theme_script}</script>
                {plausible_enabled.then(|| view! {
                    <script defer data-domain=plausible_domain src=plausible_script_url></script>
                })}
                <link rel="alternate" type_="application/rss+xml" title="Blog" href="/feeds/blog.xml"/>
                <link rel="alternate" type_="application/rss+xml" title="Projects" href="/feeds/projects.xml"/>
                <MetaTags/>
                <AutoReload options=options.clone()/>
                <HydrationScripts options islands=true/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}
