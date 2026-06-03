use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd, html};
use std::fmt::Write;

/// Convert markdown string to HTML with enhanced image handling.
///
/// Images are rendered with `loading="lazy"`, and for Immich proxy images
/// (`/api/images/{id}?w=X&h=Y`), generates `width`/`height` attributes
/// and `srcset` for responsive loading.
pub fn markdown_to_html(markdown: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_HEADING_ATTRIBUTES);

    let parser = Parser::new_ext(markdown, options);

    let mut html_output = String::new();
    let mut in_image = false;
    let mut image_url = String::new();
    let mut image_title = String::new();
    let mut image_alt = String::new();

    // Collect events, handling images specially
    let mut events: Vec<Event> = Vec::new();
    for event in parser {
        match &event {
            Event::Start(Tag::Image {
                dest_url, title, ..
            }) => {
                in_image = true;
                image_url = dest_url.to_string();
                image_title = title.to_string();
                image_alt.clear();
                continue;
            }
            Event::End(TagEnd::Image) => {
                in_image = false;
                let img_html = render_image_tag(&image_url, &image_alt, &image_title);
                events.push(Event::Html(img_html.into()));
                continue;
            }
            Event::Text(text) if in_image => {
                image_alt.push_str(text);
                continue;
            }
            _ => {}
        }
        events.push(event);
    }

    html::push_html(&mut html_output, events.into_iter());
    html_output
}

/// Parse dimension query params (`?w=X&h=Y`) from a URL.
pub(crate) fn parse_image_dimensions(url: &str) -> (String, Option<u32>, Option<u32>) {
    if let Some(idx) = url.find('?') {
        let base = &url[..idx];
        let query = &url[idx + 1..];
        let mut width = None;
        let mut height = None;
        for pair in query.split('&') {
            if let Some(val) = pair.strip_prefix("w=") {
                width = val.parse().ok();
            } else if let Some(val) = pair.strip_prefix("h=") {
                height = val.parse().ok();
            }
        }
        (base.to_string(), width, height)
    } else {
        (url.to_string(), None, None)
    }
}

/// Render an `<img>` tag with lazy loading, optional dimensions, and srcset for proxy images.
pub(crate) fn render_image_tag(url: &str, alt: &str, title: &str) -> String {
    let (base_url, width, height) = parse_image_dimensions(url);
    let is_proxy = base_url.starts_with("/api/images/");
    let escaped_alt = html_escape(alt);

    let mut tag = String::new();

    if is_proxy {
        let width_descriptor = width.map_or("2560w".to_string(), |w| format!("{}w", w));
        let _ = write!(
            tag,
            "<img src=\"{}?size=preview\" \
             srcset=\"{}?size=thumbnail 250w, {}?size=preview 1440w, {}?size=original {}\" \
             sizes=\"(max-width: 768px) 100vw, (max-width: 1200px) 80vw, 1200px\" \
             loading=\"lazy\" alt=\"{}\"",
            base_url, base_url, base_url, base_url, width_descriptor, escaped_alt
        );
    } else {
        let _ = write!(
            tag,
            "<img src=\"{}\" loading=\"lazy\" alt=\"{}\"",
            html_escape(url),
            escaped_alt
        );
    }

    if let Some(w) = width {
        let _ = write!(tag, " width=\"{}\"", w);
    }
    if let Some(h) = height {
        let _ = write!(tag, " height=\"{}\"", h);
    }
    if !title.is_empty() {
        let _ = write!(tag, " title=\"{}\"", html_escape(title));
    }

    tag.push_str(" />");
    tag
}

/// Minimal HTML escaping for attribute values.
pub(crate) fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
