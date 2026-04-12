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
///   - width: Optional image width in pixels (for layout shift prevention)
///   - height: Optional image height in pixels (for layout shift prevention)
#let blog-image(src, placement: "inline", caption: none, alt: "", width: none, height: none) = {
  let classes = "blog-image placement-" + placement
  let is-proxy = src.starts-with("/api/images/")

  // Build img attributes
  let img-attrs = (
    alt: alt,
    loading: "lazy",
  )

  // Set src: use preview size as default for proxy images
  if is-proxy {
    img-attrs.insert("src", src + "?size=preview")
    // Build srcset for responsive loading
    let width-descriptor = if width != none { str(width) + "w" } else { "2560w" }
    img-attrs.insert("srcset",
      src + "?size=thumbnail 250w, " +
      src + "?size=preview 1440w, " +
      src + "?size=original " + width-descriptor
    )
    img-attrs.insert("sizes", "(max-width: 768px) 100vw, (max-width: 1200px) 80vw, 1200px")
  } else {
    img-attrs.insert("src", src)
  }

  // Add dimensions for layout shift prevention
  if width != none {
    img-attrs.insert("width", str(width))
  }
  if height != none {
    img-attrs.insert("height", str(height))
  }

  html.elem("figure", attrs: (class: classes), {
    html.elem("img", attrs: img-attrs)
    if caption != none {
      html.elem("figcaption", caption)
    }
  })
}

/// Display a hero image (full-width above the fold).
///
/// Convenience wrapper around blog-image with placement: "hero".
#let hero-image(src, caption: none, alt: "", width: none, height: none) = {
  blog-image(src, placement: "hero", caption: caption, alt: alt, width: width, height: height)
}

/// Display a gallery of images in a grid layout.
///
/// Parameters:
///   - images: Positional arguments, each a dictionary with:
///     - src: Image URL
///     - alt: Alt text (optional)
///     - caption: Caption (optional)
///     - width: Width in pixels (optional)
///     - height: Height in pixels (optional)
#let gallery(..images) = {
  html.elem("div", attrs: (class: "blog-gallery"), {
    for img in images.pos() {
      blog-image(
        img.at("src"),
        alt: img.at("alt", default: ""),
        caption: img.at("caption", default: none),
        width: img.at("width", default: none),
        height: img.at("height", default: none),
      )
    }
  })
}
