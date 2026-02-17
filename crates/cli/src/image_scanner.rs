use std::collections::HashMap;

use regex::Regex;

/// A local image reference found in a Typst source file.
#[derive(Debug)]
pub struct ImageReference {
    /// The path as written in the .typ file (e.g., "photos/sunset.jpg")
    pub src: String,
}

/// Scan Typst content for image references that point to local files.
///
/// Matches `blog-image("path", ...)` and `hero-image("path", ...)` calls,
/// filtering out URLs and already-resolved proxy paths.
pub fn scan_image_references(content: &str) -> Vec<ImageReference> {
    let re = Regex::new(r#"(?:blog-image|hero-image)\(\s*"([^"]+)""#).unwrap();

    re.captures_iter(content)
        .filter_map(|cap| {
            let src = cap.get(1)?.as_str();
            // Skip absolute URLs and already-resolved proxy paths
            if src.starts_with("http://") || src.starts_with("https://") || src.starts_with("/api/")
            {
                return None;
            }
            Some(ImageReference {
                src: src.to_string(),
            })
        })
        .collect()
}

/// Replace local image paths with Immich proxy URLs.
///
/// `mapping` maps local paths to Immich asset IDs.
pub fn replace_image_references(content: &str, mapping: &HashMap<String, String>) -> String {
    let mut result = content.to_string();
    for (local_path, asset_id) in mapping {
        let proxy_url = format!("/api/images/{}", asset_id);
        result = result.replace(
            &format!("\"{}\"", local_path),
            &format!("\"{}\"", proxy_url),
        );
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_finds_local_images() {
        let content = r#"
#blog-image("photos/sunset.jpg", placement: "hero", alt: "Sunset")
Some text here.
#hero-image("diagram.png", caption: "Architecture")
"#;
        let refs = scan_image_references(content);
        assert_eq!(refs.len(), 2);
        assert_eq!(refs[0].src, "photos/sunset.jpg");
        assert_eq!(refs[1].src, "diagram.png");
    }

    #[test]
    fn test_scan_skips_urls() {
        let content = r#"
#blog-image("https://example.com/image.png", alt: "Remote")
#blog-image("/api/images/abc-123", alt: "Proxy")
#hero-image("http://cdn.example.com/pic.jpg")
"#;
        let refs = scan_image_references(content);
        assert!(refs.is_empty());
    }

    #[test]
    fn test_scan_mixed() {
        let content = r#"
#blog-image("local.jpg", alt: "Local")
#blog-image("https://remote.com/img.png", alt: "Remote")
#hero-image("another-local.png")
"#;
        let refs = scan_image_references(content);
        assert_eq!(refs.len(), 2);
        assert_eq!(refs[0].src, "local.jpg");
        assert_eq!(refs[1].src, "another-local.png");
    }

    #[test]
    fn test_replace_image_references() {
        let content = r#"#blog-image("photo.jpg", alt: "Test")"#;
        let mut mapping = HashMap::new();
        mapping.insert(
            "photo.jpg".to_string(),
            "550e8400-e29b-41d4-a716-446655440000".to_string(),
        );

        let result = replace_image_references(content, &mapping);
        assert_eq!(
            result,
            r#"#blog-image("/api/images/550e8400-e29b-41d4-a716-446655440000", alt: "Test")"#
        );
    }

    #[test]
    fn test_replace_multiple_references() {
        let content = r#"
#blog-image("a.jpg")
#hero-image("b.png")
"#;
        let mut mapping = HashMap::new();
        mapping.insert("a.jpg".to_string(), "id-aaa".to_string());
        mapping.insert("b.png".to_string(), "id-bbb".to_string());

        let result = replace_image_references(content, &mapping);
        assert!(result.contains("\"/api/images/id-aaa\""));
        assert!(result.contains("\"/api/images/id-bbb\""));
    }

    #[test]
    fn test_no_references() {
        let content = "Just some text without any image references.";
        let refs = scan_image_references(content);
        assert!(refs.is_empty());
    }
}
