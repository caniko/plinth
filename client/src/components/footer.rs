use leptos::*;

/// Site footer
#[component]
pub fn Footer() -> impl IntoView {
    let current_year = 2026; // In production, could use chrono to get actual year

    view! {
        <footer class="bg-white dark:bg-gray-900 border-t border-gray-200 dark:border-gray-800 mt-auto">
            <div class="container mx-auto px-4 py-8">
                <div class="grid grid-cols-1 md:grid-cols-3 gap-8">
                    // About section
                    <div>
                        <h3 class="text-lg font-semibold mb-4 text-gray-900 dark:text-white">
                            "About"
                        </h3>
                        <p class="text-gray-600 dark:text-gray-400">
                            "A personal website showcasing my work, thoughts, and experiences in software engineering."
                        </p>
                    </div>

                    // Quick Links
                    <div>
                        <h3 class="text-lg font-semibold mb-4 text-gray-900 dark:text-white">
                            "Quick Links"
                        </h3>
                        <ul class="space-y-2">
                            <li>
                                <a href="/" class="text-gray-600 dark:text-gray-400 hover:text-blue-600 dark:hover:text-blue-400 transition-colors">
                                    "Home"
                                </a>
                            </li>
                            <li>
                                <a href="/about" class="text-gray-600 dark:text-gray-400 hover:text-blue-600 dark:hover:text-blue-400 transition-colors">
                                    "About"
                                </a>
                            </li>
                            <li>
                                <a href="/portfolio" class="text-gray-600 dark:text-gray-400 hover:text-blue-600 dark:hover:text-blue-400 transition-colors">
                                    "Portfolio"
                                </a>
                            </li>
                            <li>
                                <a href="/blog" class="text-gray-600 dark:text-gray-400 hover:text-blue-600 dark:hover:text-blue-400 transition-colors">
                                    "Blog"
                                </a>
                            </li>
                        </ul>
                    </div>

                    // Social/Contact (placeholder)
                    <div>
                        <h3 class="text-lg font-semibold mb-4 text-gray-900 dark:text-white">
                            "Connect"
                        </h3>
                        <p class="text-gray-600 dark:text-gray-400">
                            "Contact information and social links can go here."
                        </p>
                    </div>
                </div>

                // Copyright
                <div class="mt-8 pt-8 border-t border-gray-200 dark:border-gray-800 text-center">
                    <p class="text-gray-600 dark:text-gray-400">
                        "© " {current_year} " Personal Website. Built with "
                        <a href="https://leptos.dev" target="_blank" rel="noopener" class="text-blue-600 dark:text-blue-400 hover:underline">
                            "Leptos"
                        </a>
                        " and "
                        <a href="https://www.rust-lang.org" target="_blank" rel="noopener" class="text-blue-600 dark:text-blue-400 hover:underline">
                            "Rust"
                        </a>
                        "."
                    </p>
                </div>
            </div>
        </footer>
    }
}
