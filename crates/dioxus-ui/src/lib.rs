use dioxus::prelude::*;
use dioxus_router::{Routable, Router};
use tartan_ui_core::Identity;
use tartan_ui_dioxus::{AppShell, EmptyState};

#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[route("/")]
    Home {},
    #[route("/projects")]
    Projects {},
    #[route("/about")]
    About {},
    #[route("/:..segments")]
    NotFound { segments: Vec<String> },
}

#[component]
pub fn App() -> Element {
    rsx! { Router::<Route> {} }
}

#[component]
fn Home() -> Element {
    rsx! {
        AppShell {
            title: "Plinth".to_string(),
            identity: None::<Identity>,
            EmptyState {
                heading: "Plinth".to_string(),
                message: "The Dioxus shell is ready for the shared portfolio, blog, and project loaders.".to_string(),
            }
        }
    }
}

#[component]
fn Projects() -> Element {
    rsx! {
        AppShell {
            title: "Plinth projects".to_string(),
            identity: None::<Identity>,
            EmptyState {
                heading: "Projects".to_string(),
                message: "Project data will be loaded from plinth-project after the SSR cutover.".to_string(),
            }
        }
    }
}

#[component]
fn About() -> Element {
    rsx! {
        AppShell {
            title: "About Plinth".to_string(),
            identity: None::<Identity>,
            EmptyState {
                heading: "About".to_string(),
                message: "Plinth keeps the same content model while the renderer moves to Dioxus.".to_string(),
            }
        }
    }
}

#[component]
fn NotFound(segments: Vec<String>) -> Element {
    rsx! {
        AppShell {
            title: "Plinth".to_string(),
            identity: None::<Identity>,
            EmptyState {
                heading: "Page not found".to_string(),
                message: format!("No route matches /{}", segments.join("/")),
            }
        }
    }
}
