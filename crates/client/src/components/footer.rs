use leptos::prelude::*;

use crate::app::use_site_config;

/// Site footer
#[component]
pub fn Footer() -> impl IntoView {
    let config = use_site_config();

    let github = config.social.github.clone();
    let gitlab = config.social.gitlab.clone();
    let codeberg = config.social.codeberg.clone();
    let mastodon = config.social.mastodon.clone();
    let bluesky = config.social.bluesky.clone();
    let email = config.author.email.clone();

    view! {
        <footer class="mt-auto border-t border-gray-200 dark:border-amber-900/30">
            <div class="container mx-auto px-4 py-3 flex flex-wrap items-center justify-between gap-x-6 gap-y-1">
                <div class="flex flex-wrap gap-4 text-gray-500 dark:text-amber-500">
                    {(!github.is_empty()).then(|| view! {
                        <a href={github} target="_blank" rel="noopener" aria-label="GitHub" class="hover:text-blue-600 dark:hover:text-amber-200 transition-colors">
                            <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 24 24">
                                <path d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.531 1.032 1.531 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z"/>
                            </svg>
                        </a>
                    })}
                    {(!gitlab.is_empty()).then(|| view! {
                        <a href={gitlab} target="_blank" rel="noopener" aria-label="GitLab" class="hover:text-blue-600 dark:hover:text-amber-200 transition-colors">
                            <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 24 24">
                                <path d="M22.65 14.39L12 22.13 1.35 14.39a.84.84 0 01-.3-.94l1.22-3.78 2.44-7.51A.42.42 0 014.82 2a.43.43 0 01.58 0 .42.42 0 01.11.18l2.44 7.49h8.1l2.44-7.51A.42.42 0 0118.6 2a.43.43 0 01.58 0 .42.42 0 01.11.18l2.44 7.51L23 13.45a.84.84 0 01-.35.94z"/>
                            </svg>
                        </a>
                    })}
                    {(!codeberg.is_empty()).then(|| view! {
                        <a href={codeberg} target="_blank" rel="noopener" aria-label="Codeberg" class="hover:text-blue-600 dark:hover:text-amber-200 transition-colors">
                            <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 24 24">
                                <path d="M11.955.49A12 12 0 000 12.49a12.1 12.1 0 001.652 6.092l9.606-9.605a1.074 1.074 0 011.394 0l9.606 9.605A12.1 12.1 0 0023.91 12.49 12 12 0 0011.955.49zm.145 6.51a1.593 1.593 0 011.122.472l7.762 7.762a10.034 10.034 0 01-8.884 5.396 10.034 10.034 0 01-8.884-5.396l7.762-7.762A1.593 1.593 0 0112.1 7z"/>
                            </svg>
                        </a>
                    })}
                    {(!mastodon.is_empty()).then(|| view! {
                        <a href={mastodon} target="_blank" rel="noopener" aria-label="Mastodon" class="hover:text-blue-600 dark:hover:text-amber-200 transition-colors">
                            <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 24 24">
                                <path d="M23.268 5.313c-.35-2.578-2.617-4.61-5.304-5.004C17.51.242 15.792 0 11.813 0h-.03c-3.98 0-4.835.242-5.288.309C3.882.692 1.496 2.518.917 5.127.64 6.412.61 7.837.661 9.143c.074 1.874.088 3.745.26 5.611.118 1.24.325 2.47.62 3.68.55 2.237 2.777 4.098 4.96 4.857 2.336.792 4.849.923 7.256.38.265-.061.527-.132.786-.213.585-.184 1.27-.39 1.774-.753a.057.057 0 00.023-.043v-1.809a.052.052 0 00-.02-.041.053.053 0 00-.046-.01 20.282 20.282 0 01-4.709.547c-2.73 0-3.463-1.284-3.674-1.818a5.593 5.593 0 01-.319-1.433.053.053 0 01.066-.054 19.648 19.648 0 004.634.534h.568c1.816-.03 3.63-.15 5.418-.45 .044-.01.088-.02.13-.034 2.4-.493 4.68-2.04 4.914-5.96.009-.156.034-1.62.034-1.782 0-.547.196-3.876-.026-5.924zM19.033 16H16.07v-6.46c0-1.362-.572-2.054-1.716-2.054-1.266 0-1.899.82-1.899 2.442v3.33H9.51V9.928c0-1.622-.634-2.442-1.899-2.442-1.144 0-1.717.692-1.717 2.054V16H2.978V9.71c0-1.362.347-2.446 1.04-3.252.716-.806 1.653-1.22 2.814-1.22 1.343 0 2.36.516 3.037 1.548l.654 1.097.654-1.097c.677-1.032 1.694-1.548 3.037-1.548 1.161 0 2.098.414 2.814 1.22.694.806 1.04 1.89 1.04 3.252V16z"/>
                            </svg>
                        </a>
                    })}
                    {(!bluesky.is_empty()).then(|| view! {
                        <a href={bluesky} target="_blank" rel="noopener" aria-label="Bluesky" class="hover:text-blue-600 dark:hover:text-amber-200 transition-colors">
                            <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 24 24">
                                <path d="M12 10.8c-1.087-2.114-4.046-6.053-6.798-7.995C2.566.944 1.561 1.266.902 1.565.139 1.908 0 3.08 0 3.768c0 .69.378 5.65.624 6.479.785 2.627 3.6 3.492 6.156 3.121-4.295.652-7.545 2.243-2.84 7.377C8.22 25.698 10.548 18.852 12 15.89c1.452 2.962 3.049 9.108 8.06 4.855 4.705-5.134 1.455-6.725-2.84-7.377 2.556.371 5.371-.494 6.156-3.121.246-.828.624-5.79.624-6.479 0-.688-.139-1.86-.902-2.203-.659-.3-1.664-.62-4.3 1.24C16.046 4.748 13.087 8.687 12 10.8z"/>
                            </svg>
                        </a>
                    })}
                    {(!email.is_empty()).then(|| view! {
                        <a href={format!("mailto:{}", email)} aria-label="Email" class="hover:text-blue-600 dark:hover:text-amber-200 transition-colors">
                            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 8l7.89 5.26a2 2 0 002.22 0L21 8M5 19h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z"/>
                            </svg>
                        </a>
                    })}
                </div>
                <p class="text-xs text-gray-400 dark:text-amber-600">
                    "Built with "
                    <a href={config.footer.project_url} target="_blank" rel="noopener" class="hover:text-blue-600 dark:hover:text-amber-200 transition-colors">
                        {config.footer.project_name}
                    </a>
                </p>
            </div>
        </footer>
    }
}
