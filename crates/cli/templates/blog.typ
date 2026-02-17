// Plinth blog template — provides image placement functions for HTML output.
//
// These functions generate semantic HTML elements with CSS classes
// that are styled by Plinth's CSS (input.css).

/// Display a blog image with configurable placement.
///
/// Parameters:
///   - src: Image URL (resolved to /api/images/{id} by the CLI)
///   - placement: One of "inline", "hero", "float-left", "float-right",
///                "full-width" (default: "inline")
///   - caption: Optional figure caption
///   - alt: Alt text for accessibility
#let blog-image(src, placement: "inline", caption: none, alt: "") = {
  let classes = "blog-image placement-" + placement
  html.elem("figure", attrs: (class: classes), {
    html.elem("img", attrs: (
      src: src,
      alt: alt,
      loading: "lazy",
    ))
    if caption != none {
      html.elem("figcaption", caption)
    }
  })
}

/// Display a hero image (full-width above the fold).
///
/// Convenience wrapper around blog-image with placement: "hero".
#let hero-image(src, caption: none, alt: "") = {
  blog-image(src, placement: "hero", caption: caption, alt: alt)
}

/// Display a gallery of images in a grid layout.
///
/// Parameters:
///   - images: Positional arguments, each a dictionary with:
///     - src: Image URL
///     - alt: Alt text (optional)
///     - caption: Caption (optional)
#let gallery(..images) = {
  html.elem("div", attrs: (class: "blog-gallery"), {
    for img in images.pos() {
      blog-image(
        img.at("src"),
        alt: img.at("alt", default: ""),
        caption: img.at("caption", default: none),
      )
    }
  })
}
