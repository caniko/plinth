//! The embeddable workspace component.
//!
//! It renders only the workspace body: the host app supplies the sidebar,
//! topbar and status bar, and appends [`crate::STYLE`] to its stylesheet.

use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    rc::Rc,
    time::Duration,
};

use chrono::Local;
use dioxus::prelude::*;

use crate::{
    Draft, Field, FieldError, MAX_SLUG_LEN, PostSummary, PublishError, Receipt, Replaces, Site,
    Target, draft::parse_tags, drafts, list_posts, markdown, publish, slugify,
};

const AUTOSAVE_DEBOUNCE: Duration = Duration::from_millis(1000);

#[derive(Clone, Debug, PartialEq)]
enum PostsState {
    Loading,
    Loaded(Vec<PostSummary>),
    Failed(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BodyView {
    Write,
    Preview,
}

/// The workspace body: site context on the left, the post editor on the right.
#[component]
pub fn BlogWorkspace(sites: Vec<Site>, drafts_dir: Option<PathBuf>) -> Element {
    let initial_site = if sites.is_empty() { None } else { Some(0usize) };
    let selected = use_signal(move || initial_site);
    let mut draft = use_signal(Draft::default);
    let mut tags_input = use_signal(String::new);
    let mut slug_touched = use_signal(|| false);
    let mut attempted = use_signal(|| false);
    // Numbers each send: its error panel mounts afresh (focus and speech even
    // when the same error comes back), and Dismiss acts only on its own one.
    let mut attempt = use_signal(|| 0u64);
    let mut touched = use_signal(HashSet::<Field>::new);
    let mut body_view = use_signal(|| BodyView::Write);
    let mut posts = use_signal(|| PostsState::Loading);
    let mut reload = use_signal(|| 0u32);
    let mut result = use_signal(|| None::<Result<Receipt, PublishError>>);
    let mut in_flight = use_signal(|| false);
    let mut dirty = use_signal(|| 0u64);
    let mut saving_suppressed = use_signal(|| true);
    let mut saved_at = use_signal(|| None::<String>);
    let mut save_error = use_signal(|| None::<String>);
    let mut restored_draft = use_signal(|| false);
    // The slug "Keep as a new post" moved the draft to, said once in the status.
    let mut kept_slug = use_signal(|| None::<String>);
    // The last discarded draft, until the next edit or site switch: Undo.
    let mut discarded = use_signal(|| None::<Draft>);
    // Set between a site switch and the effect that installs that site's
    // draft: until then the editor still holds the previous site's content,
    // so nothing may be sent or discarded.
    let mut restoring = use_signal(|| false);
    // Mounted field handles, for moving focus to the field that needs attention.
    let mut mounts = use_signal(HashMap::<Field, Rc<MountedData>>::new);
    // The text fields are uncontrolled (`initial_value`): echoing every
    // keystroke back through `value` races the webview and drops characters
    // while typing. Bumping the epoch remounts them when the content is
    // replaced wholesale (site switch, restore, "Write another").
    let mut epoch = use_signal(|| 0u32);
    let mut focus_title = use_signal(|| false);
    // Unsent drafts by site id for as long as the workspace is mounted, so a
    // site switch keeps the draft even without (or with a failing) drafts_dir.
    let mut held = use_signal(HashMap::<String, Draft>::new);
    // The preview follows the body's scroll position (0.0 top, 1.0 bottom).
    let mut preview_mount = use_signal(|| None::<Rc<MountedData>>);
    // The editor heading: errors and Dismiss bring it into view (below 860 px
    // the result sits right under it), so the site and title stay in sight.
    let mut head_mount = use_signal(|| None::<Rc<MountedData>>);
    let mut body_scroll = use_signal(|| 0.0f64);
    let sync_preview = move || async move {
        let Some(preview) = preview_mount.peek().clone() else {
            return;
        };
        if let (Ok(content), Ok(frame)) = (
            preview.get_scroll_size().await,
            preview.get_client_rect().await,
        ) {
            let top = (content.height - frame.height()).max(0.0) * *body_scroll.peek();
            let _ = preview
                .scroll(
                    dioxus::html::geometry::PixelsVector2D::new(0.0, top),
                    ScrollBehavior::Instant,
                )
                .await;
        }
    };

    // Restore (or clear) the editor whenever the target site changes.
    {
        let drafts_dir = drafts_dir.clone();
        let sites = sites.clone();
        use_effect(move || {
            let restored = selected()
                .and_then(|index| sites.get(index))
                .and_then(|site| {
                    held.peek().get(&site.id).cloned().or_else(|| {
                        drafts_dir
                            .as_ref()
                            .and_then(|dir| drafts::load(dir, &site.id))
                    })
                })
                .unwrap_or_default();
            // A restored slug that no longer matches the title was edited by
            // hand; keep it instead of regenerating it on the next keystroke.
            restored_draft.set(restored != Draft::default());
            kept_slug.set(None);
            discarded.set(None);
            let slug_is_derived = slugify(&restored.title) == restored.slug;
            slug_touched.set(!restored.slug.is_empty() && !slug_is_derived);
            tags_input.set(restored.tags.join(", "));
            draft.set(restored);
            body_view.set(BodyView::Write);
            let next = *epoch.peek() + 1;
            epoch.set(next);
            attempted.set(false);
            touched.set(HashSet::new());
            result.set(None);
            saved_at.set(None);
            save_error.set(None);
            saving_suppressed.set(true);
            restoring.set(false);
        });
    }

    // Load the selected site's posts, and again when Retry is pressed.
    let mut load_generation = use_signal(|| 0u64);
    {
        let sites = sites.clone();
        use_effect(move || {
            let index = selected();
            let Some(site) = index.and_then(|index| sites.get(index)).cloned() else {
                return;
            };
            let _ = reload();
            // Every load (site switch or reload) takes a number; only the
            // latest may apply, so A -> B -> A cannot let the first A win.
            let generation = {
                let mut latest = load_generation.write();
                *latest += 1;
                *latest
            };
            posts.set(PostsState::Loading);
            spawn(async move {
                let loaded = list_posts(&site).await;
                if *load_generation.peek() != generation {
                    return;
                }
                match loaded {
                    Ok(list) => posts.set(PostsState::Loaded(list)),
                    Err(error) => posts.set(PostsState::Failed(format_error(&error))),
                }
            });
        });
    }

    // Debounced autosave. `dirty` only moves on a real edit, so restoring a
    // draft on selection never rewrites the file.
    {
        let drafts_dir = drafts_dir.clone();
        let sites = sites.clone();
        use_effect(move || {
            let revision = dirty();
            if revision == 0 || saving_suppressed() {
                return;
            }
            let (Some(index), Some(dir)) = (selected(), drafts_dir.clone()) else {
                return;
            };
            let Some(site_id) = sites.get(index).map(|site| site.id.clone()) else {
                return;
            };
            spawn(async move {
                tokio::time::sleep(AUTOSAVE_DEBOUNCE).await;
                if dirty() != revision || saving_suppressed() || selected() != Some(index) {
                    return;
                }
                let snapshot = draft();
                match drafts::save(&dir, &site_id, &snapshot) {
                    Ok(()) => {
                        // The file is now the newest copy; an older held
                        // snapshot must not win on the next restore.
                        held.write().remove(&site_id);
                        saving_suppressed.set(true);
                        saved_at.set(Some(Local::now().format("%H:%M").to_string()));
                        save_error.set(None);
                    }
                    Err(error) => save_error.set(Some(error.to_string())),
                }
            });
        });
    }

    {
        let drafts_dir = drafts_dir.clone();
        let sites = sites.clone();
        // Leaving the workspace inside the autosave debounce must not lose
        // the last edits.
        use_drop(move || {
            let Some(dir) = drafts_dir.as_ref() else {
                return;
            };
            // Held drafts are copies whose write failed or has not happened
            // yet; the current editor content is newer than its held copy.
            for (site_id, kept) in held.peek().iter() {
                let _ = drafts::save(dir, site_id, kept);
            }
            if *saving_suppressed.peek() {
                return;
            }
            if let Some(site) = selected.peek().and_then(|index| sites.get(index)) {
                let _ = drafts::save(dir, &site.id, &draft.peek());
            }
        });
    }

    // Fields are read-only while a publish is in flight, so the receipt always
    // describes exactly what was sent; `disabled` would drop keyboard focus.
    let mut edited = move || {
        // Until the restore effect runs, the editor holds the outgoing site's
        // content, already saved by `select_site`, and the restore replaces
        // it: saving an edit to it would land under the incoming site's id.
        if *restoring.peek() {
            return;
        }
        // Any edit retires the last receipt or error: it described the
        // previous content.
        if result.peek().is_some() {
            result.set(None);
        }
        if discarded.peek().is_some() {
            discarded.set(None);
        }
        if kept_slug.peek().is_some() {
            kept_slug.set(None);
        }
        saving_suppressed.set(false);
        dirty.set(dirty() + 1);
    };

    let sites_for_select = sites.clone();
    let drafts_for_select = drafts_dir.clone();
    let select_site = EventHandler::new(move |index: usize| {
        let mut selected = selected;
        let current = *selected.peek();
        if current == Some(index) || in_flight() {
            return;
        }
        // Edits still inside the autosave debounce would be replaced by the
        // other site's draft: hold them in memory and write them out. A
        // failed write is not fatal here because the held copy survives.
        if !*saving_suppressed.peek()
            && let Some(site) = current.and_then(|current| sites_for_select.get(current))
        {
            let mut held = held;
            held.write().insert(site.id.clone(), draft.peek().clone());
            if let Some(dir) = drafts_for_select.as_ref() {
                let _ = drafts::save(dir, &site.id, &draft.peek());
            }
        }
        // Until the restore effect runs the editor still shows the outgoing
        // draft; a second switch in that window must not save it under the
        // site just selected.
        saving_suppressed.set(true);
        restoring.set(true);
        // The next thing to do on a site is write: skip its posts list.
        focus_title.set(true);
        selected.set(Some(index));
    });

    let sites_for_submit = sites.clone();
    let drafts_for_submit = drafts_dir.clone();
    let submit = EventHandler::new(move |_: ()| {
        // A shown receipt means this content is already on the site.
        // A shown receipt: already sent. A changed draft: sending again would
        // only repeat the refusal; the panel offers the ways forward.
        if in_flight()
            || *restoring.peek()
            || matches!(
                *result.peek(),
                Some(Ok(_)) | Some(Err(PublishError::DraftChanged))
            )
        {
            return;
        }
        let Some(site) = selected()
            .and_then(|index| sites_for_submit.get(index))
            .cloned()
        else {
            return;
        };
        attempted.set(true);
        attempt += 1;
        let candidate = draft().normalized();
        if let Err(errors) = candidate.validate() {
            // The body error lives in the write pane; never leave it hidden
            // behind the preview.
            if errors.iter().any(|error| error.field == Field::Body) {
                body_view.set(BodyView::Write);
            }
            let first = errors
                .first()
                .and_then(|error| mounts.peek().get(&error.field).cloned());
            result.set(Some(Err(PublishError::Invalid(errors))));
            if let Some(handle) = first {
                spawn(async move {
                    let _ = handle.set_focus(true).await;
                });
            }
            return;
        }

        in_flight.set(true);
        kept_slug.set(None);
        result.set(None);
        let drafts_dir = drafts_for_submit.clone();
        let revision = *dirty.peek();
        spawn(async move {
            let outcome = publish(&site, &candidate).await;
            // `in_flight` stays set until the outcome is shown, so a site
            // switch cannot slip in and receive this site's result.
            match outcome {
                Ok(receipt) => {
                    // The published content stays in the editor until
                    // "Write another"; suppressing saves keeps the deleted
                    // draft file from coming back. Edits made while the
                    // request was in flight are newer than what was
                    // published, so their autosave is kept.
                    if *dirty.peek() == revision {
                        saving_suppressed.set(true);
                        held.write().remove(&site.id);
                        if let Some(dir) = drafts_dir.as_ref()
                            && let Err(error) = drafts::delete(dir, &site.id)
                        {
                            // The sent draft is still on disk: say so, and
                            // hold an empty one so it is not restored.
                            held.write().insert(site.id.clone(), Draft::default());
                            save_error
                                .set(Some(format!("could not remove the saved text: {error}")));
                        }
                        saved_at.set(None);
                    } else {
                        held.write().remove(&site.id);
                    }
                    reload.set(reload() + 1);
                    result.set(Some(Ok(receipt)));
                }
                Err(error) => {
                    if error == PublishError::SlugTaken {
                        let handle = mounts.peek().get(&Field::Slug).cloned();
                        if let Some(handle) = handle {
                            let _ = handle.set_focus(true).await;
                        }
                    }
                    // A lost response does not prove nothing was stored:
                    // refresh the list so the operator can see.
                    // A lost response does not prove nothing was stored, and a
                    // changed draft must be reopened from a fresh list.
                    if matches!(
                        error,
                        PublishError::Unreachable(_) | PublishError::DraftChanged
                    ) {
                        reload.set(reload() + 1);
                    }
                    result.set(Some(Err(error)));
                }
            }
            in_flight.set(false);
        });
    });

    let mut write_another = move || {
        draft.set(Draft::default());
        body_view.set(BodyView::Write);
        epoch.set(epoch() + 1);
        focus_title.set(true);
        restored_draft.set(false);
        kept_slug.set(None);
        tags_input.set(String::new());
        slug_touched.set(false);
        attempted.set(false);
        touched.set(HashSet::new());
        result.set(None);
        saved_at.set(None);
        save_error.set(None);
        saving_suppressed.set(true);
    };

    let sites_for_discard = sites.clone();
    let drafts_for_discard = drafts_dir.clone();
    let discard = EventHandler::new(move |_: ()| {
        // A queued click can arrive before the render that hides the button.
        if in_flight() || *restoring.peek() {
            return;
        }
        let previous = draft.peek().clone();
        if let Some(site) = selected
            .peek()
            .and_then(|index| sites_for_discard.get(index))
        {
            // A file that cannot be removed would come back on the next
            // restore: keep the draft and say why.
            if let Some(dir) = drafts_for_discard.as_ref()
                && let Err(error) = drafts::delete(dir, &site.id)
            {
                save_error.set(Some(format!("could not clear the saved text: {error}")));
                return;
            }
            held.write().remove(&site.id);
        }
        write_another();
        discarded.set(Some(previous));
    });
    let undo_discard = EventHandler::new(move |_: ()| {
        // During a switch the editor belongs to no site yet: an undo there
        // would be saved under the next site's id.
        if in_flight() || *restoring.peek() {
            return;
        }
        let Some(previous) = discarded.peek().clone() else {
            return;
        };
        let slug_is_derived = slugify(&previous.title) == previous.slug;
        slug_touched.set(!previous.slug.is_empty() && !slug_is_derived);
        tags_input.set(previous.tags.join(", "));
        draft.set(previous);
        epoch.set(epoch() + 1);
        focus_title.set(true);
        // `edited` clears the undo and schedules the autosave again.
        edited();
    });

    // Opening a site draft replaces the editor; what was there stays one Undo
    // away, as after "Clear editor".
    let open_draft = EventHandler::new(move |opened: Draft| {
        // Refused while locked (sending or a receipt shown); the button only
        // looks disabled so it keeps keyboard focus.
        if in_flight() || *restoring.peek() || matches!(*result.peek(), Some(Ok(_))) {
            return;
        }
        let previous = draft.peek().clone();
        // The slug names the post being replaced: keep it unless edited.
        slug_touched.set(true);
        tags_input.set(opened.tags.join(", "));
        draft.set(opened);
        body_view.set(BodyView::Write);
        epoch.set(epoch() + 1);
        focus_title.set(true);
        restored_draft.set(false);
        kept_slug.set(None);
        attempted.set(false);
        touched.set(HashSet::new());
        edited();
        if previous != Draft::default() {
            discarded.set(Some(previous));
        }
    });

    let sites_for_dir = sites.clone();
    let posts_dir = use_memo(move || {
        match selected()
            .and_then(|index| sites_for_dir.get(index))
            .map(|site| &site.target)
        {
            Some(Target::Project { root, .. }) => crate::publish::project_posts_dir(root).ok(),
            _ => None,
        }
    });
    let current_site = selected().and_then(|index| sites.get(index)).cloned();
    let draft_value = draft();
    let tags_value = tags_input();
    let outcome = result();
    let posts_state = posts();
    let published_ok = matches!(outcome, Some(Ok(_)));
    // Sent content is not editable: in flight so the receipt matches what was
    // sent, afterwards because a second publish of the same slug cannot work.
    let locked = in_flight() || published_ok;
    let errors: Vec<FieldError> = draft_value.validate().err().unwrap_or_default();
    let attempted_now = attempted();
    let touched_now = touched();
    let show_error = |field: Field| field_error(&errors, attempted_now, &touched_now, field);

    let title_error = show_error(Field::Title);
    let slug_taken = matches!(outcome.as_ref(), Some(Err(PublishError::SlugTaken)));
    // An empty slug under an empty title is the title's problem, said once.
    let slug_follows_empty_title =
        draft_value.title.trim().is_empty() && draft_value.slug.trim().is_empty();
    let slug_error = show_error(Field::Slug)
        .filter(|_| !slug_follows_empty_title)
        .or_else(|| {
            let site_name = current_site.as_ref().map(|site| site.name.as_str())?;
            slug_taken.then(|| publish_error_message(&PublishError::SlugTaken, site_name))
        });
    // The only description error is the length limit, which the counter
    // already shows live; the message stays with it.
    let description_error = errors
        .iter()
        .find(|error| error.field == Field::Description)
        .map(|error| error.message.clone());
    let body_error = show_error(Field::Body);

    let preview_html = markdown::render(&draft_value.body);
    let description_len = draft_value.description.trim().chars().count();
    let description_over = description_len > crate::MAX_DESCRIPTION_LEN;

    let has_content = draft_value != Draft::default();
    // Order: a failed save, then a send in progress, then one-off notices.
    let status_text = if let Some(error) = save_error() {
        // A failed save outranks every notice: the draft may not survive.
        format!("Autosave failed: {error}")
    } else if in_flight() {
        // Polite status: a keyboard user in the body hears that it is going.
        // Same verb as the primary button.
        current_site
            .as_ref()
            .map(
                |site| match (site.target.is_personal(), draft_value.published) {
                    (_, false) => format!("Saving draft on {}…", site.name),
                    (true, true) => format!("Publishing to {}…", site.name),
                    (false, true) => format!("Writing to {}…", site.name),
                },
            )
            .unwrap_or_default()
    } else if discarded().is_some() {
        // Clear empties the editor; Open fills it with a site draft. Both keep
        // the previous text one Undo away.
        if has_content {
            "Replaced by the opened draft".to_string()
        } else {
            "Editor cleared".to_string()
        }
    } else if let Some(slug) = kept_slug() {
        format!("Kept as a new post: {slug}")
    } else if !saving_suppressed() && dirty() > 0 {
        "Unsaved changes".to_string()
    } else if let Some(saved) = saved_at() {
        format!("Autosaved {saved}")
    } else if restored_draft() && !published_ok {
        "Unsent text restored".to_string()
    } else {
        String::new()
    };

    let slug_or_dots = match draft_value.slug.trim() {
        "" => "…",
        slug => slug,
    };
    let slug_hint = match current_site.as_ref().map(|site| &site.target) {
        Some(target @ Target::Personal { api_url, .. }) => {
            let public = target.public_url().unwrap_or(api_url);
            format!("{}/posts/{slug_or_dots}", strip_scheme(public))
        }
        Some(Target::Project { root, .. }) => {
            // Relative to the site root, like the receipt.
            let dir = posts_dir().unwrap_or_else(|| root.join("posts"));
            let relative = dir.strip_prefix(root).unwrap_or(&dir).to_path_buf();
            format!("{}/{slug_or_dots}.md", relative.display())
        }
        None => String::new(),
    };

    let personal_sites: Vec<(usize, Site)> = sites
        .iter()
        .cloned()
        .enumerate()
        .filter(|(_, site)| site.target.is_personal())
        .collect();
    let project_sites: Vec<(usize, Site)> = sites
        .iter()
        .cloned()
        .enumerate()
        .filter(|(_, site)| !site.target.is_personal())
        .collect();
    let posts_heading = match (current_site.as_ref(), &posts_state) {
        (Some(site), PostsState::Loaded(posts)) => {
            format!("Posts on {} · {}", site.name, posts.len())
        }
        (Some(site), _) => format!("Posts on {}", site.name),
        (None, _) => String::new(),
    };
    // Personal posts link to the live page; project posts show their file slug.
    let post_link_base = match current_site.as_ref().map(|site| &site.target) {
        Some(target @ Target::Personal { .. }) => target.public_url().map(str::to_string),
        _ => None,
    };

    let editing = draft_value.replaces.clone();
    // Once a receipt says the post went public it is no longer a draft.
    // Says what the editor holds: a new post, an opened site draft, or (after
    // a receipt) what became of that draft.
    let draft_changed = matches!(outcome, Some(Err(PublishError::DraftChanged)));
    let editor_title = match (draft_value.replaces.is_some(), published_ok) {
        (true, true) if draft_value.published => "Draft published",
        (true, true) => "Draft saved",
        // The link to the site draft is broken until Keep or a reopen.
        (true, false) if draft_changed => "Draft changed",
        (true, false) => "Edit draft",
        (false, _) => "New post",
    };

    rsx! {
        div {
            class: "plinth-ws",
            // Ctrl+Enter publishes from anywhere in the workspace.
            onkeydown: move |event| {
                if event.key() == Key::Named(NamedKey::Enter)
                    && (event.modifiers().ctrl() || event.modifiers().meta())
                {
                    event.prevent_default();
                    submit.call(());
                }
            },
            if sites.is_empty() {
                div { class: "plinth-empty",
                    h2 { class: "plinth-empty-title", "No sites configured" }
                    p {
                        "The hub passes Plinth sites to this workspace from sites.toml. Add a personal site (admin API) or a project site (repository checkout) there, then reopen Sites."
                    }
                }
            }
            div { class: if sites.is_empty() { "plinth-layout plinth-is-hidden" } else { "plinth-layout" },
                nav { class: "plinth-context", aria_label: "Sites and posts",
                if !sites.is_empty() {
                    div { class: "plinth-site-groups",
                        if !personal_sites.is_empty() {
                            section { class: "plinth-site-group",
                                // Shown in capitals by CSS; the name stays mixed case
                                // so screen readers do not spell it out.
                                h2 { class: "plinth-label", aria_label: "Personal sites", "Personal sites" }
                                for (index, site) in personal_sites {
                                    {site_button(
                                        index,
                                        selected() == Some(index),
                                        in_flight(),
                                        held.read().contains_key(&site.id)
                                            || drafts_dir.as_ref().is_some_and(|dir| drafts::path(dir, &site.id).exists()),
                                        site,
                                        select_site,
                                    )}
                                }
                            }
                        }
                        if !project_sites.is_empty() {
                            section { class: "plinth-site-group",
                                h2 { class: "plinth-label", aria_label: "Project sites", "Project sites" }
                                for (index, site) in project_sites {
                                    {site_button(
                                        index,
                                        selected() == Some(index),
                                        in_flight(),
                                        held.read().contains_key(&site.id)
                                            || drafts_dir.as_ref().is_some_and(|dir| drafts::path(dir, &site.id).exists()),
                                        site,
                                        select_site,
                                    )}
                                }
                            }
                        }
                    }

                    if current_site.is_some() {
                        section { class: "plinth-posts plinth-posts-wide",
                            h2 { class: "plinth-label", aria_label: "{posts_heading}", "{posts_heading}" }
                            {posts_body(&posts_state, post_link_base.as_deref(), reload, open_draft, locked, editing.as_ref())}
                        }
                        details {
                            class: "plinth-posts-details",
                            // A failure must be visible without opening the list.
                            open: matches!(posts_state, PostsState::Failed(_)),
                            summary { "{posts_heading}" }
                            {posts_body(&posts_state, post_link_base.as_deref(), reload, open_draft, locked, editing.as_ref())}
                        }
                    }
                }
            }

            if let Some(site) = current_site.as_ref() {
                form {
                    class: "plinth-editor",
                    // The target is part of the name: after a site switch, focus
                    // lands in Title and the form is what names the site.
                    aria_label: "{editor_title}, {site.name}",
                    // Implicit submission (Enter in a single-line field) must
                    // not publish; only the button and Ctrl+Enter do.
                    onsubmit: move |event| event.prevent_default(),
                    aria_busy: if in_flight() { "true" } else { "false" },
                    header {
                        class: "plinth-editor-head",
                        onmounted: move |event| head_mount.set(Some(event.data())),
                        div { class: "plinth-editor-heading",
                            h2 { class: "plinth-editor-title", "{editor_title}" }
                            span { class: "plinth-chip plinth-chip-target", "→ {site.name}" }
                        }
                        if save_error().is_some() {
                            span { class: "plinth-save-state plinth-is-failed", role: "alert",
                                span { aria_hidden: "true", "! " }
                                "{status_text}"
                            }
                        } else {
                            span { class: "plinth-save-state", role: "status", "{status_text}" }
                        }
                        if discarded().is_some() {
                            button {
                                r#type: "button",
                                class: "plinth-btn plinth-btn-quiet plinth-btn-small",
                                onclick: move |_| undo_discard.call(()),
                                "Undo"
                            }
                        } else if has_content && !locked {
                            button {
                                r#type: "button",
                                class: "plinth-btn plinth-btn-quiet plinth-btn-small",
                                onclick: move |_| discard.call(()),
                                // The local copy is "unsent text"; "draft" means
                                // the site's unpublished post.
                                "Clear editor"
                            }
                        }
                    }

                    for field_epoch in [epoch()] {
                    div { key: "{field_epoch}", class: if locked { "plinth-fields plinth-is-locked" } else { "plinth-fields" },
                        div { class: "plinth-field",
                            label { class: "plinth-field-label", r#for: "plinth-title", "Title" }
                            // A textarea so a long title wraps instead of
                            // clipping; it stays a single line of text.
                            textarea {
                                id: "plinth-title",
                                // WebKit drops "editable" but reports no read-only state
                                // for locked textareas: say it explicitly.
                                aria_readonly: if locked { "true" } else { "false" },
                                class: "plinth-input plinth-input-title",
                                rows: "1",
                                initial_value: "{draft_value.title}",
                                readonly: locked,
                                autocomplete: "off",
                                onkeydown: move |event| {
                                    if event.key() == Key::Named(NamedKey::Enter)
                                        && !(event.modifiers().ctrl() || event.modifiers().meta())
                                    {
                                        event.prevent_default();
                                    }
                                },
                                aria_invalid: if title_error.is_some() { "true" } else { "false" },
                                aria_describedby: title_error.as_ref().map(|_| "plinth-title-error"),
                                onmounted: move |event| async move {
                                    mounts.write().insert(Field::Title, event.data());
                                    if focus_title() {
                                        focus_title.set(false);
                                        let _ = event.data().set_focus(true).await;
                                    }
                                },
                                oninput: move |event| {
                                    let value = event.value().replace(['\r', '\n'], " ");
                                    let auto_slug = !slug_touched();
                                    {
                                        let mut draft = draft;
                                        let mut current = draft.write();
                                        current.title = value.clone();
                                        if auto_slug {
                                            current.slug = slugify(&value);
                                        }
                                    }
                                    edited();
                                },
                                onblur: move |_| {
                                    touched.write().insert(Field::Title);
                                },
                            }
                            if let Some(message) = &title_error {
                                p { id: "plinth-title-error", class: "plinth-field-error", "{message}" }
                            }
                        }

                        div { class: "plinth-field-row",
                        div { class: "plinth-field",
                            label { class: "plinth-field-label", r#for: "plinth-slug", "Slug" }
                            input {
                                id: "plinth-slug",
                                class: "plinth-input plinth-input-mono",
                                r#type: "text",
                                initial_value: "{draft_value.slug}",
                                readonly: locked,
                                autocomplete: "off",
                                spellcheck: "false",
                                aria_invalid: if slug_error.is_some() { "true" } else { "false" },
                                aria_describedby: if slug_error.is_some() { "plinth-slug-hint plinth-slug-error" } else { "plinth-slug-hint" },
                                onmounted: move |event| {
                                    mounts.write().insert(Field::Slug, event.data());
                                },
                                oninput: move |event| {
                                    slug_touched.set(true);
                                    draft.write().slug = event.value();
                                    edited();
                                },
                                onblur: move |_| {
                                    touched.write().insert(Field::Slug);
                                },
                            }
                            p { id: "plinth-slug-hint", class: "plinth-hint plinth-hint-mono", "{slug_hint}" }
                            if let Some(message) = &slug_error {
                                p { id: "plinth-slug-error", class: "plinth-field-error", "{message}" }
                            }
                        }
                        div { class: "plinth-field",
                            label { class: "plinth-field-label", r#for: "plinth-tags", "Tags" }
                            input {
                                id: "plinth-tags",
                                class: "plinth-input",
                                r#type: "text",
                                initial_value: "{tags_value}",
                                readonly: locked,
                                autocomplete: "off",
                                spellcheck: "false",
                                placeholder: "rust, release",
                                aria_describedby: "plinth-tags-hint",
                                oninput: move |event| {
                                    let value = event.value();
                                    tags_input.set(value.clone());
                                    draft.write().tags = parse_tags(&value);
                                    edited();
                                },
                                onblur: move |_| {
                                    touched.write().insert(Field::Tags);
                                },
                            }
                            p { id: "plinth-tags-hint", class: "plinth-hint", "Comma-separated" }
                            if !draft_value.tags.is_empty() {
                                ul { class: "plinth-tag-chips",
                                    for tag in draft_value.tags.iter() {
                                        li { class: "plinth-chip", key: "{tag}", "{tag}" }
                                    }
                                }
                            }
                        }
                        }

                        div { class: "plinth-field",
                            div { class: "plinth-label-row",
                                label { class: "plinth-field-label", r#for: "plinth-description", "Description" }
                                span {
                                    id: "plinth-description-count",
                                    class: if description_over { "plinth-counter plinth-is-over" } else { "plinth-counter" },
                                    // Read as a description of the field: units, not a bare fraction.
                                    aria_label: "{description_len} of {crate::MAX_DESCRIPTION_LEN} characters",
                                    "{description_len}/{crate::MAX_DESCRIPTION_LEN}"
                                }
                            }
                            textarea {
                                id: "plinth-description",
                                // WebKit drops "editable" but reports no read-only state
                                // for locked textareas: say it explicitly.
                                aria_readonly: if locked { "true" } else { "false" },
                                class: "plinth-input plinth-textarea plinth-textarea-description",
                                rows: "2",
                                initial_value: "{draft_value.description}",
                                readonly: locked,
                                aria_invalid: if description_error.is_some() { "true" } else { "false" },
                                aria_describedby: if description_error.is_some() { "plinth-description-count plinth-description-error" } else { "plinth-description-count" },
                                onmounted: move |event| {
                                    mounts.write().insert(Field::Description, event.data());
                                },
                                oninput: move |event| {
                                    draft.write().description = event.value();
                                    edited();
                                },
                                onblur: move |_| {
                                    touched.write().insert(Field::Description);
                                },
                            }
                            if let Some(message) = &description_error {
                                p { id: "plinth-description-error", class: "plinth-field-error", "{message}" }
                            }
                        }

                        div { class: "plinth-field plinth-field-body",
                            div { class: "plinth-label-row",
                                label { class: "plinth-field-label", r#for: "plinth-body", "Body · Markdown" }
                                div { class: "plinth-segmented plinth-view-toggle", role: "group", aria_label: "Body view",
                                    button {
                                        r#type: "button",
                                        aria_pressed: if body_view() == BodyView::Write { "true" } else { "false" },
                                        onclick: move |_| body_view.set(BodyView::Write),
                                        "Write"
                                    }
                                    button {
                                        r#type: "button",
                                        aria_pressed: if body_view() == BodyView::Preview { "true" } else { "false" },
                                        onclick: move |_| {
                                            body_view.set(BodyView::Preview);
                                            // The pane has no size until it is shown.
                                            spawn(async move {
                                                tokio::time::sleep(Duration::from_millis(50)).await;
                                                sync_preview().await;
                                            });
                                        },
                                        "Preview"
                                    }
                                }
                            }
                            span { class: "plinth-field-label plinth-preview-label", aria_hidden: "true", "Preview" }
                            div {
                                class: if body_view() == BodyView::Write { "plinth-body-grid plinth-view-write" } else { "plinth-body-grid plinth-view-preview" },
                                div { class: "plinth-body-write",
                                    if let Some(message) = &body_error {
                                        p { id: "plinth-body-error", class: "plinth-field-error", "{message}" }
                                    }
                                    textarea {
                                        id: "plinth-body",
                                // WebKit drops "editable" but reports no read-only state
                                // for locked textareas: say it explicitly.
                                aria_readonly: if locked { "true" } else { "false" },
                                        class: "plinth-input plinth-textarea plinth-body",
                                        initial_value: "{draft_value.body}",
                                readonly: locked,
                                        spellcheck: "false",
                                        aria_invalid: if body_error.is_some() { "true" } else { "false" },
                                        aria_describedby: body_error.as_ref().map(|_| "plinth-body-error"),
                                        onmounted: move |event| {
                                            mounts.write().insert(Field::Body, event.data());
                                        },
                                        oninput: move |event| {
                                            draft.write().body = event.value();
                                            edited();
                                        },
                                        onscroll: move |event| {
                                            let data = event.data();
                                            let range = f64::from(data.scroll_height() - data.client_height());
                                            body_scroll.set(if range > 0.0 {
                                                (data.scroll_top() / range).clamp(0.0, 1.0)
                                            } else {
                                                0.0
                                            });
                                            spawn(sync_preview());
                                        },
                                        onblur: move |_| {
                                            touched.write().insert(Field::Body);
                                        },
                                    }
                                }
                                div { class: "plinth-body-preview",
                                    if draft_value.body.trim().is_empty() {
                                        div { class: "plinth-preview plinth-preview-empty", role: "region", aria_label: "Preview",
                                            "The rendered post appears here as you write."
                                        }
                                    } else {
                                        div {
                                            class: "plinth-preview plinth-prose",
                                            // Focusable so a long preview scrolls by keyboard.
                                            tabindex: "0",
                                            role: "region",
                                            aria_label: "Preview",
                                            onmounted: move |event| preview_mount.set(Some(event.data())),
                                            dangerous_inner_html: "{preview_html}",
                                        }
                                    }
                                }
                            }
                        }
                    }
                    }

                    // A sibling of the fields, not part of the sticky foot: below
                    // 860 px it moves under the heading (CSS order) and scrolls with
                    // the page instead of covering the body.
                    if let Some(outcome) = outcome.as_ref() {
                        match outcome {
                            Ok(receipt) => rsx! {
                                div { id: "plinth-receipt", class: "plinth-result plinth-is-ok", role: "status",
                                    // The focused button reads this as its description:
                                    // Orca does not announce nodes added to a live region.
                                    div { id: "plinth-receipt-text",
                                    {match (receipt, draft_value.published) {
                                        (Receipt::Personal { url, .. }, true) => rsx! {
                                            p { class: "plinth-result-title",
                                                span { aria_hidden: "true", "✓ " }
                                                "Published to {site.name}"
                                            }
                                            p {
                                                a { class: "plinth-mono", href: url.clone(), target: "_blank", rel: "noopener", "{url}" }
                                            }
                                        },
                                        (Receipt::Personal { slug, .. }, false) => rsx! {
                                            p { class: "plinth-result-title",
                                                span { aria_hidden: "true", "✓ " }
                                                "Saved as draft on {site.name}"
                                            }
                                            p {
                                                code { class: "plinth-mono", "{slug}" }
                                            }
                                            p { "Not public. It is listed with the site's posts; open it there to keep editing." }
                                        },
                                        (Receipt::Project { path, slug }, published) => rsx! {
                                            p { class: "plinth-result-title",
                                                span { aria_hidden: "true", "✓ " }
                                                if published { "Written to {site.name}" } else { "Draft written to {site.name}" }
                                            }
                                            p {
                                                // Relative to the site root: short enough to read at 760.
                                                code { class: "plinth-mono",
                                                    if let Target::Project { root, .. } = &site.target {
                                                        "{path.strip_prefix(root).unwrap_or(path).display()}"
                                                    } else {
                                                        "{path.display()}"
                                                    }
                                                }
                                            }
                                            p {
                                                if published {
                                                    "Not live yet: commit and push the repository to deploy it."
                                                    if let Target::Project { base_url: Some(base), .. } = &site.target {
                                                        " It will be at "
                                                        code { class: "plinth-mono", "{base.trim_end_matches('/')}/blog/{slug}/" }
                                                        "."
                                                    }
                                                } else {
                                                    "Marked unpublished: the site will not render it until the file says published: true."
                                                }
                                            }
                                        },
                                    }}
                                    }
                                    button {
                                        r#type: "button",
                                        class: "plinth-btn plinth-btn-quiet",
                                        aria_describedby: "plinth-receipt-text",
                                        // Publishing is done; the next keyboard action is here.
                                        onmounted: move |event| async move {
                                            let _ = event.data().set_focus(true).await;
                                        },
                                        onclick: move |_| write_another(),
                                        "Write another"
                                    }
                                }
                            },
                            Err(error) => {
                                // Field errors move focus to the field; scrolling to the
                                // heading would pull it away.
                                let invalid = matches!(error, PublishError::Invalid(_) | PublishError::SlugTaken);
                                rsx! {
                                div {
                                    key: "{attempt}",
                                    class: "plinth-result plinth-is-error",
                                    role: "alert",
                                    // Below 860 px the panel sits under the heading, maybe
                                    // above the fold; "nearest" leaves a visible panel alone.
                                    // Validation moves focus to the first bad field;
                                    // scrolling to the panel would pull it off-screen.
                                    onmounted: move |event| async move {
                                        if invalid {
                                            return;
                                        }
                                        // The heading, not the panel: the panel follows it
                                        // below 860 px, and the context stays in view.
                                        // ponytail: dioxus-desktop 0.8.0-alpha.3 sends these
                                        // options as vertical/horizontal, which scrollIntoView
                                        // ignores, so this aligns to the start. That is the
                                        // intent below 860 px; above it the heading is already
                                        // at the top. Revisit if a later Dioxus maps them to
                                        // block/inline.
                                        let target = head_mount.peek().clone().unwrap_or(event.data());
                                        let _ = target
                                            .scroll_to_with_options(ScrollToOptions {
                                                behavior: ScrollBehavior::Instant,
                                                vertical: ScrollLogicalPosition::Nearest,
                                                horizontal: ScrollLogicalPosition::Nearest,
                                            })
                                            .await;
                                    },
                                    // Other errors focus the message itself: Orca drops alerts
                                    // added next to other DOM changes but always reads focus,
                                    // and a stray key here presses nothing. Shift+Tab is back
                                    // in the editor, Tab reaches the actions.
                                    // Other errors focus this heading, described by the rest:
                                    // Orca drops alerts added next to other DOM changes but
                                    // reads focus, and a stray key here presses nothing. Tab
                                    // reaches the actions, Shift+Tab is back in the editor.
                                    p {
                                        class: "plinth-result-title plinth-result-alert",
                                        role: "heading",
                                        aria_level: "3",
                                        tabindex: if invalid { None } else { Some("-1") },
                                        aria_describedby: "plinth-error-why plinth-error-detail",
                                        onmounted: move |event| async move {
                                            if !invalid {
                                                let _ = event.data().set_focus(true).await;
                                            }
                                        },
                                        if matches!(error, PublishError::SlugTaken) {
                                            // The full message sits on the Slug field.
                                            "Not sent: {site.name} already has a post with this slug."
                                        } else {
                                            "{publish_error_message(error, &site.name)}"
                                        }
                                    }
                                    if matches!(error, PublishError::Unreachable(_)) {
                                        p { id: "plinth-error-why", "No confirmation came back, so the post may or may not have arrived. Reload the posts list to check before sending again; your text is still here." }
                                    } else if matches!(error, PublishError::DraftChanged) {
                                        p { id: "plinth-error-why", "It was changed, published or removed there after you opened it. Nothing was written. Your text is still here: open the current draft from the list, or keep yours as a new post." }
                                    } else if !matches!(error, PublishError::Invalid(_)) {
                                        p { id: "plinth-error-why",
                                            if site.target.is_personal() { "The post was not published." } else { "Nothing was written." }
                                            " Your text is still here."
                                        }
                                    }
                                    if let PublishError::Unreachable(detail) | PublishError::Io(detail) = error {
                                        // Paths relative to a project site's root, as in the receipt.
                                        p { id: "plinth-error-detail", class: "plinth-hint plinth-hint-mono",
                                            if let Target::Project { root, .. } = &site.target {
                                                {detail.replace(&format!("{}/", root.display()), "")}
                                            } else {
                                                "{detail}"
                                            }
                                        }
                                    }
                                    if matches!(error, PublishError::DraftChanged) {
                                        button {
                                            r#type: "button",
                                            class: "plinth-btn plinth-btn-quiet",
                                            // Sending again would create a post; a taken
                                            // slug is then reported on the slug field.
                                            onclick: move |_| {
                                                // A queued click must not act mid-send or
                                                // mid-switch, or after the error is gone.
                                                let still_shown = matches!(
                                                    *result.peek(),
                                                    Some(Err(PublishError::DraftChanged))
                                                );
                                                if in_flight() || *restoring.peek() || !still_shown {
                                                    return;
                                                }
                                                // The draft's slug is taken (by the newer draft):
                                                // pick the first free one the fresh list shows.
                                                let taken: HashSet<String> = match &*posts.peek() {
                                                    PostsState::Loaded(list) => {
                                                        list.iter().map(|post| post.slug.clone()).collect()
                                                    }
                                                    _ => HashSet::new(),
                                                };
                                                {
                                                    let mut current = draft.write();
                                                    current.replaces = None;
                                                    current.slug = free_slug(&current.slug, &taken);
                                                }
                                                slug_touched.set(true);
                                                // Fields take their value on mount: remount them
                                                // to show the new slug, then focus it once mounted.
                                                epoch.set(epoch() + 1);
                                                edited();
                                                kept_slug.set(Some(draft.peek().slug.clone()));
                                                let converted = (*epoch.peek(), *selected.peek());
                                                spawn(async move {
                                                    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                                                    // Only the editor just converted: not one a site
                                                    // switch, discard or open put there meanwhile.
                                                    if (*epoch.peek(), *selected.peek()) != converted {
                                                        return;
                                                    }
                                                    let slug_field = mounts.peek().get(&Field::Slug).cloned();
                                                    if let Some(handle) = slug_field {
                                                        let _ = handle.set_focus(true).await;
                                                    }
                                                });
                                            },
                                            "Keep as a new post"
                                        }
                                    }
                                    if matches!(error, PublishError::Unreachable(_)) {
                                        // The check the copy asks for, inside the alert, so the
                                        // remedy travels with the message.
                                        button {
                                            r#type: "button",
                                            class: "plinth-btn plinth-btn-quiet plinth-panel-reload",
                                            onclick: move |_| reload.set(reload() + 1),
                                            "Reload posts"
                                        }
                                    }
                                    // The primary button is the retry; Dismiss puts the
                                    // message away without editing.
                                    button {
                                        r#type: "button",
                                        class: "plinth-btn plinth-btn-quiet",
                                        onclick: move |_| {
                                            // The button goes away with the panel: put
                                            // keyboard focus back in the editor first. WebKit
                                            // reports no focus change to screen readers when
                                            // focus moves while the focused button is removed.
                                            let title = mounts.peek().get(&Field::Title).cloned();
                                            let head = head_mount.peek().clone();
                                            let shown = *attempt.peek();
                                            spawn(async move {
                                                // Only the error this button was on: not one
                                                // from a send started meanwhile.
                                                let still_shown = move || {
                                                    *attempt.peek() == shown
                                                        && !*in_flight.peek()
                                                        && matches!(*result.peek(), Some(Err(_)))
                                                };
                                                if !still_shown() {
                                                    return;
                                                }
                                                if let Some(handle) = title {
                                                    let _ = handle.set_focus(true).await;
                                                }
                                                if still_shown() {
                                                    result.set(None);
                                                }
                                                if let Some(head) = head {
                                                    let _ = head
                                                        .scroll_to_with_options(ScrollToOptions {
                                                            behavior: ScrollBehavior::Instant,
                                                            vertical: ScrollLogicalPosition::Nearest,
                                                            horizontal: ScrollLogicalPosition::Nearest,
                                                        })
                                                        .await;
                                                }
                                            });
                                        },
                                        "Dismiss"
                                    }
                                }
                                }
                            }
                        }
                    }

                    div { class: "plinth-foot",
                    div { class: "plinth-actions",
                        div { class: "plinth-visibility",
                            span { class: "plinth-field-label", "Visibility" }
                            div { class: "plinth-segmented", role: "group", aria_label: "Visibility",
                                button {
                                    r#type: "button",
                                    aria_disabled: if locked { "true" } else { "false" },
                                    aria_pressed: if draft_value.published { "true" } else { "false" },
                                    onclick: move |_| {
                                        if locked {
                                            return;
                                        }
                                        draft.write().published = true;
                                        edited();
                                    },
                                    "Public"
                                }
                                button {
                                    r#type: "button",
                                    aria_disabled: if locked { "true" } else { "false" },
                                    aria_pressed: if !draft_value.published { "true" } else { "false" },
                                    onclick: move |_| {
                                        if locked {
                                            return;
                                        }
                                        draft.write().published = false;
                                        edited();
                                    },
                                    "Draft"
                                }
                            }
                        }
                        div { class: "plinth-spacer" }
                        // Only beside a primary that the shortcut can press.
                        if !locked && !draft_changed {
                            span { class: "plinth-kbd", aria_hidden: "true", "Ctrl+Enter" }
                        }
                        button {
                            r#type: "button",
                            class: "plinth-btn plinth-primary",
                            // Only offered while the shortcut can act.
                            aria_keyshortcuts: if in_flight() || published_ok || draft_changed { "" } else { "Control+Enter" },
                            // While sending the button stays focusable (aria-disabled;
                            // submit ignores it), so a keyboard user who pressed it
                            // keeps their place. Done and changed-draft states disable it.
                            disabled: published_ok || draft_changed,
                            aria_disabled: if in_flight() { "true" } else { "false" },
                            onclick: move |_| submit.call(()),
                            if in_flight() {
                                // Same verb as the idle label and the status line.
                                if !draft_value.published { "Saving…" } else if site.target.is_personal() { "Publishing…" } else { "Writing…" }
                            } else if published_ok {
                                // Project sites always write a file ("Write to …",
                                // "Draft written to …"); personal drafts are saved.
                                if !site.target.is_personal() { "Written" } else if !draft_value.published { "Saved" } else { "Published" }
                            } else if draft_value.published {
                                // The target drops out on very narrow modules; the
                                // chip on the heading still names the site.
                                if site.target.is_personal() { "Publish" } else { "Write" }
                                span { class: "plinth-primary-target", " to {site.name}" }
                            } else {
                                "Save as draft"
                                span { class: "plinth-primary-target", " on {site.name}" }
                            }
                        }
                    }
                    }
                }
            }
            }
        }
    }
}

fn site_button(
    index: usize,
    is_selected: bool,
    busy: bool,
    has_draft: bool,
    site: Site,
    on_select: EventHandler<usize>,
) -> Element {
    let meta = match &site.target {
        Target::Personal { api_url, .. } => {
            strip_scheme(site.target.public_url().unwrap_or(api_url)).to_string()
        }
        Target::Project { root, .. } => shorten_path(root),
    };
    let kind = if site.target.is_personal() {
        "personal"
    } else {
        "project"
    };
    let draft_note = if has_draft { ", unsent text kept" } else { "" };
    rsx! {
        button {
            key: "{site.id}",
            r#type: "button",
            class: if is_selected { "plinth-site plinth-is-selected" } else { "plinth-site" },
            // The current site, not a toggle: Orca answers a pressed-state change
            // by focusing the button again, undoing the move to Title.
            aria_current: if is_selected { "true" } else { "false" },
            // One clean name for screen readers, not the chip and path run together.
            aria_label: "{site.name}, {kind} site{draft_note}",
            // Switching mid-publish would attach the receipt to another site:
            // select_site refuses; aria-disabled keeps keyboard focus here.
            aria_disabled: if busy { "true" } else { "false" },
            onclick: move |_| on_select.call(index),
            span { class: "plinth-site-row",
                span { class: "plinth-site-name", "{site.name}" }
                span { class: "plinth-chip", "{site.target.kind_label()}" }
            }
            span { class: "plinth-site-meta", "{meta}" }
            if has_draft {
                span { class: "plinth-site-draft", "Unsent text kept" }
            }
        }
    }
}

fn posts_body(
    state: &PostsState,
    link_base: Option<&str>,
    mut reload: Signal<u32>,
    open: EventHandler<Draft>,
    locked: bool,
    editing: Option<&Replaces>,
) -> Element {
    match state {
        PostsState::Loading => rsx! {
            p { class: "plinth-hint", "Loading posts…" }
        },
        PostsState::Failed(message) => rsx! {
            div { class: "plinth-posts-error",
                // Polite: a list that failed to load must not interrupt typing
                // or talk over a publish error.
                p { role: "status", span { aria_hidden: "true", "! " } "Could not load posts: {message}" }
                button {
                    r#type: "button",
                    class: "plinth-btn plinth-btn-quiet",
                    onclick: move |_| reload.set(reload() + 1),
                    "Reload posts"
                }
            }
        },
        PostsState::Loaded(posts) if posts.is_empty() => rsx! {
            p { class: "plinth-hint", "No posts yet" }
        },
        PostsState::Loaded(posts) => rsx! {
            ul { class: "plinth-post-list",
                for post in posts.iter() {
                    li { class: "plinth-post", key: "{post.slug}",
                        div { class: "plinth-post-row",
                            if let Some(base) = link_base.filter(|_| post.published) {
                                a {
                                    class: "plinth-post-title",
                                    href: "{base}/posts/{post.slug}",
                                    target: "_blank",
                                    rel: "noopener",
                                    "{post.title}"
                                }
                            } else {
                                span { class: "plinth-post-title", "{post.title}" }
                            }
                            if !post.published {
                                span { class: "plinth-chip plinth-chip-draft", "Draft" }
                            }
                            // The exact version in the editor: a newer save of the same
                            // draft gets Open again, so it can be reopened.
                            if editing.is_some()
                                && post.draft.as_ref().and_then(|listed| listed.replaces.as_ref()) == editing
                            {
                                span { class: "plinth-chip plinth-chip-editing", aria_current: "true", "Editing" }
                            } else if let Some(opened) = post.draft.clone() {
                                // Disabled, not hidden, while locked: the row keeps its shape.
                                button {
                                    r#type: "button",
                                    class: "plinth-btn plinth-btn-quiet plinth-btn-small",
                                    aria_label: "Open draft {post.title}",
                                    aria_disabled: if locked { "true" } else { "false" },
                                    onclick: move |_| open.call(opened.clone()),
                                    "Open"
                                }
                            } else if !post.published {
                                span {
                                    class: "plinth-chip plinth-chip-draft",
                                    title: "Typst drafts are edited with plinth-cli",
                                    "Typst"
                                }
                            }
                        }
                        span { class: "plinth-post-date",
                            if let Some(date) = post.date.as_ref() {
                                "{date} · "
                            }
                            "{post.slug}"
                        }
                    }
                }
            }
        },
    }
}

/// `base`, or the first `base-2`, `base-3`, … not in `taken`, cut so the
/// result stays within the slug limit in characters.
fn free_slug(base: &str, taken: &HashSet<String>) -> String {
    // Restored drafts and file names are not validated: cut by characters.
    let base = base.trim();
    if base.is_empty() || !taken.contains(base) {
        return base.to_string();
    }
    (2..)
        .map(|n| {
            let suffix = format!("-{n}");
            let room = MAX_SLUG_LEN.saturating_sub(suffix.len());
            let head: String = base.chars().take(room).collect();
            format!("{}{suffix}", head.trim_end_matches('-'))
        })
        .find(|slug| !taken.contains(slug))
        .unwrap_or_else(|| base.to_string())
}

fn field_error(
    errors: &[FieldError],
    attempted: bool,
    touched: &HashSet<Field>,
    field: Field,
) -> Option<String> {
    if !attempted && !touched.contains(&field) {
        return None;
    }
    errors
        .iter()
        .find(|error| error.field == field)
        .map(|error| error.message.clone())
}

fn strip_scheme(url: &str) -> &str {
    url.trim_start_matches("https://")
        .trim_start_matches("http://")
}

/// The last three path components behind an ellipsis: enough to recognise a
/// checkout without wrapping the context column.
fn shorten_path(path: &std::path::Path) -> String {
    let parts: Vec<_> = path.iter().map(|part| part.to_string_lossy()).collect();
    if parts.len() <= 4 {
        return path.display().to_string();
    }
    format!("…/{}", parts[parts.len() - 3..].join("/"))
}

fn format_error(error: &PublishError) -> String {
    match error {
        PublishError::Invalid(errors) => errors
            .first()
            .map(|problem| problem.message.clone())
            .unwrap_or_else(|| "the draft is invalid".to_string()),
        PublishError::SlugTaken => "that slug already exists".to_string(),
        PublishError::DraftChanged => "the opened draft can no longer be replaced".to_string(),
        PublishError::Unauthorized => "the API key was rejected".to_string(),
        PublishError::Unreachable(_) => "the site is unreachable".to_string(),
        PublishError::Rejected { status, message } => format!("HTTP {status}: {message}"),
        PublishError::Io(message) => message.clone(),
    }
}

fn publish_error_message(error: &PublishError, site_name: &str) -> String {
    match error {
        PublishError::Invalid(_) => "Fix the highlighted fields to publish.".to_string(),
        PublishError::SlugTaken => {
            format!("A post with this slug already exists on {site_name}. Choose another slug.")
        }
        PublishError::Unauthorized => {
            format!("The API key for {site_name} was rejected. Check the key file and try again.")
        }
        PublishError::Unreachable(_) => format!("Could not reach {site_name}."),
        PublishError::DraftChanged => {
            format!("The opened draft can no longer be replaced on {site_name}.")
        }
        PublishError::Rejected { status, message } => {
            format!("{site_name} rejected the post (HTTP {status}): {message}")
        }
        PublishError::Io(_) => format!("Could not save the post to {site_name}."),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn motion_is_limited_to_short_transitions_that_reduced_motion_removes() {
        let css = crate::STYLE;
        assert!(!css.contains("@keyframes"), "no keyframe animation");
        let reduced = css
            .split("@media (prefers-reduced-motion: reduce)")
            .nth(1)
            .expect("a reduced-motion block");
        assert!(reduced.contains("transition: none !important"));
        assert!(reduced.contains("animation: none !important"));
        // Every duration in every transition, in seconds or milliseconds.
        for line in css
            .lines()
            .map(str::trim)
            .filter(|line| line.starts_with("transition:") && !line.contains("!important"))
        {
            for word in line.split([' ', ',', ';']) {
                let seconds = if let Some(ms) = word.strip_suffix("ms") {
                    ms.parse::<f32>().ok().map(|ms| ms / 1000.0)
                } else {
                    word.strip_suffix('s').and_then(|s| s.parse::<f32>().ok())
                };
                if let Some(seconds) = seconds {
                    assert!(seconds <= 0.2, "{line}");
                }
            }
        }
    }

    #[test]
    fn free_slug_skips_taken_and_stays_within_the_limit() {
        let taken: HashSet<String> = ["idea", "idea-2"].map(String::from).into();
        assert_eq!(free_slug("fresh", &taken), "fresh");
        assert_eq!(free_slug("idea", &taken), "idea-3");
        let long = format!("{}-word", "a".repeat(MAX_SLUG_LEN - 5));
        let taken: HashSet<String> = [long.clone()].into();
        let free = free_slug(&long, &taken);
        assert!(free.len() <= MAX_SLUG_LEN && free.ends_with("-2"), "{free}");
        assert!(!free.contains("--"), "{free}");
        // A multi-byte character at the cut must not panic.
        let odd = format!("{}\u{a0}b", "a".repeat(MAX_SLUG_LEN - 3));
        let taken: HashSet<String> = [odd.clone()].into();
        let free = free_slug(&odd, &taken);
        assert!(
            free.ends_with("-2") && free.chars().count() <= MAX_SLUG_LEN,
            "{free}"
        );
        assert_eq!(free_slug("", &HashSet::new()), "");
    }
}
