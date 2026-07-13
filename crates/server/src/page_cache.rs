//! Framework-neutral page invalidation events.
//!
//! Admin writes publish narrow invalidation events here instead of coupling
//! the persistence layer to a UI renderer. The Dioxus process can drain these
//! events into its external page cache; the legacy Leptos regeneration hooks
//! remain in place until the rollback window closes.

use std::sync::{Mutex, OnceLock};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Invalidation {
    SiteContent {
        key: String,
    },
    Blog {
        slug: String,
        tags: Vec<String>,
        series: Option<String>,
    },
    Portfolio {
        slug: String,
    },
    Activity,
    Todo {
        slug: String,
        tags: Vec<String>,
    },
}

static EVENTS: OnceLock<Mutex<Vec<Invalidation>>> = OnceLock::new();

fn events() -> &'static Mutex<Vec<Invalidation>> {
    EVENTS.get_or_init(|| Mutex::new(Vec::new()))
}

pub fn publish(event: Invalidation) {
    let mut events = events().lock().expect("page invalidation lock poisoned");
    events.push(event);
    // Do not let a disabled/failed consumer grow this process indefinitely.
    if events.len() > 1024 {
        let trim = events.len() - 1024;
        events.drain(..trim);
    }
}

pub fn drain() -> Vec<Invalidation> {
    std::mem::take(&mut *events().lock().expect("page invalidation lock poisoned"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn events_are_ordered_and_drained() {
        let _ = drain();
        publish(Invalidation::Portfolio {
            slug: "demo".into(),
        });
        publish(Invalidation::Activity);
        assert_eq!(drain().len(), 2);
        assert!(drain().is_empty());
    }
}
