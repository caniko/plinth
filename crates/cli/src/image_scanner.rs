use std::collections::HashMap;

use regex::Regex;

/// A local image reference found in a source file.
#[derive(Debug)]
pub struct ImageReference {
    /// The path as written in the source file (e.g., "photos/sunset.jpg")
    pub src: String,
}

/// Resolved image mapping with proxy URL and optional dimensions.
#[derive(Debug, Clone)]
pub struct ImageMapping {
    pub asset_id: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
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

/// Replace local image paths in Typst content with Immich proxy URLs,
/// injecting width/height parameters when available.
///
/// `mapping` maps local paths to `ImageMapping` with asset ID and dimensions.
pub fn replace_image_references(content: &str, mapping: &HashMap<String, ImageMapping>) -> String {
    let re = Regex::new(r#"((?:blog-image|hero-image)\(\s*)"([^"]+)"(\s*(?:,|\)))"#).unwrap();

    let result = re.replace_all(content, |caps: &regex::Captures| {
        let prefix = &caps[1];
        let src = &caps[2];
        let suffix = &caps[3];

        if let Some(img) = mapping.get(src) {
            let proxy_url = format!("/api/images/{}", img.asset_id);
            let mut dim_params = String::new();
            if let (Some(w), Some(h)) = (img.width, img.height) {
                dim_params = format!(", width: {}, height: {}", w, h);
            }
            format!("{}\"{}\"{}{}", prefix, proxy_url, dim_params, suffix)
        } else {
            caps[0].to_string()
        }
    });

    result.to_string()
}

/// Scan markdown content for image references that point to local files.
///
/// Matches `![alt](path)` syntax, filtering out URLs and already-resolved proxy paths.
pub fn scan_markdown_image_references(content: &str) -> Vec<ImageReference> {
    let re = Regex::new(r"!\[([^\]]*)\]\(([^)\s]+)\)").unwrap();
    re.captures_iter(content)
        .filter_map(|cap| {
            let src = cap.get(2)?.as_str();
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

/// Replace local image paths in markdown with Immich proxy URLs.
///
/// Encodes dimensions as query params (`?w=X&h=Y`) so the server-side
/// markdown renderer can extract them for width/height attributes.
pub fn replace_markdown_image_references(
    content: &str,
    mapping: &HashMap<String, ImageMapping>,
) -> String {
    let mut result = content.to_string();
    for (local_path, img) in mapping {
        let mut proxy_url = format!("/api/images/{}", img.asset_id);
        if let (Some(w), Some(h)) = (img.width, img.height) {
            proxy_url = format!("{}?w={}&h={}", proxy_url, w, h);
        }
        result = result.replace(&format!("]({})", local_path), &format!("]({})", proxy_url));
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
            ImageMapping {
                asset_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
                width: None,
                height: None,
            },
        );

        let result = replace_image_references(content, &mapping);
        assert_eq!(
            result,
            r#"#blog-image("/api/images/550e8400-e29b-41d4-a716-446655440000", alt: "Test")"#
        );
    }

    #[test]
    fn test_replace_image_references_with_dimensions() {
        let content = r#"#blog-image("photo.jpg", alt: "Test")"#;
        let mut mapping = HashMap::new();
        mapping.insert(
            "photo.jpg".to_string(),
            ImageMapping {
                asset_id: "abc-123".to_string(),
                width: Some(1920),
                height: Some(1080),
            },
        );

        let result = replace_image_references(content, &mapping);
        assert!(result.contains("\"/api/images/abc-123\""));
        assert!(result.contains("width: 1920"));
        assert!(result.contains("height: 1080"));
    }

    #[test]
    fn test_replace_multiple_references() {
        let content = r#"
#blog-image("a.jpg")
#hero-image("b.png")
"#;
        let mut mapping = HashMap::new();
        mapping.insert(
            "a.jpg".to_string(),
            ImageMapping {
                asset_id: "id-aaa".to_string(),
                width: None,
                height: None,
            },
        );
        mapping.insert(
            "b.png".to_string(),
            ImageMapping {
                asset_id: "id-bbb".to_string(),
                width: None,
                height: None,
            },
        );

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

    #[test]
    fn test_scan_markdown_finds_local_images() {
        let content = "![sunset](photos/sunset.jpg)\nSome text.\n![diagram](diagram.png)";
        let refs = scan_markdown_image_references(content);
        assert_eq!(refs.len(), 2);
        assert_eq!(refs[0].src, "photos/sunset.jpg");
        assert_eq!(refs[1].src, "diagram.png");
    }

    #[test]
    fn test_scan_markdown_skips_urls() {
        let content = "![remote](https://example.com/img.png)\n![proxy](/api/images/abc)\n![http](http://cdn.com/pic.jpg)";
        let refs = scan_markdown_image_references(content);
        assert!(refs.is_empty());
    }

    #[test]
    fn test_scan_markdown_mixed() {
        let content =
            "![local](local.jpg)\n![remote](https://example.com/img.png)\n![another](dir/pic.png)";
        let refs = scan_markdown_image_references(content);
        assert_eq!(refs.len(), 2);
        assert_eq!(refs[0].src, "local.jpg");
        assert_eq!(refs[1].src, "dir/pic.png");
    }

    #[test]
    fn test_replace_markdown_image_references() {
        let content = "![sunset](photo.jpg)\nText\n![diag](diagram.png)";
        let mut mapping = HashMap::new();
        mapping.insert(
            "photo.jpg".to_string(),
            ImageMapping {
                asset_id: "id-aaa".to_string(),
                width: None,
                height: None,
            },
        );
        mapping.insert(
            "diagram.png".to_string(),
            ImageMapping {
                asset_id: "id-bbb".to_string(),
                width: None,
                height: None,
            },
        );
        let result = replace_markdown_image_references(content, &mapping);
        assert!(result.contains("](/api/images/id-aaa)"));
        assert!(result.contains("](/api/images/id-bbb)"));
    }

    #[test]
    fn test_replace_markdown_image_references_with_dimensions() {
        let content = "![photo](photo.jpg)";
        let mut mapping = HashMap::new();
        mapping.insert(
            "photo.jpg".to_string(),
            ImageMapping {
                asset_id: "id-aaa".to_string(),
                width: Some(1920),
                height: Some(1080),
            },
        );
        let result = replace_markdown_image_references(content, &mapping);
        assert!(result.contains("](/api/images/id-aaa?w=1920&h=1080)"));
    }

    #[test]
    fn test_scan_typst_extra_whitespace() {
        let content = r#"#blog-image(  "photo.jpg"  , alt: "test")"#;
        let refs = scan_image_references(content);
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].src, "photo.jpg");
    }

    #[test]
    fn test_replace_typst_partial_dimensions_width_only() {
        let content = r#"#blog-image("photo.jpg")"#;
        let mut mapping = HashMap::new();
        mapping.insert(
            "photo.jpg".to_string(),
            ImageMapping {
                asset_id: "abc".to_string(),
                width: Some(800),
                height: None,
            },
        );
        let result = replace_image_references(content, &mapping);
        // Only injects dimensions when both are present
        assert!(result.contains("\"/api/images/abc\""));
        assert!(!result.contains("width:"));
    }

    #[test]
    fn test_scan_markdown_empty_alt() {
        let content = "![](photo.jpg)";
        let refs = scan_markdown_image_references(content);
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].src, "photo.jpg");
    }

    #[test]
    fn test_replace_empty_mapping_unchanged() {
        let content = r#"#blog-image("photo.jpg", alt: "Test")"#;
        let mapping = HashMap::new();
        let result = replace_image_references(content, &mapping);
        assert_eq!(result, content);
    }

    #[test]
    fn test_replace_markdown_empty_mapping_unchanged() {
        let content = "![photo](photo.jpg)";
        let mapping = HashMap::new();
        let result = replace_markdown_image_references(content, &mapping);
        assert_eq!(result, content);
    }

    #[test]
    fn test_scan_gallery_not_matched() {
        let content = r#"#gallery((src: "a.jpg"), (src: "b.jpg"))"#;
        let refs = scan_image_references(content);
        assert!(refs.is_empty());
    }

    #[test]
    fn test_replace_typst_preserves_surrounding_content() {
        let content = "Some text before.\n#blog-image(\"photo.jpg\")\nSome text after.";
        let mut mapping = HashMap::new();
        mapping.insert(
            "photo.jpg".to_string(),
            ImageMapping {
                asset_id: "id-123".to_string(),
                width: None,
                height: None,
            },
        );
        let result = replace_image_references(content, &mapping);
        assert!(result.contains("Some text before."));
        assert!(result.contains("Some text after."));
        assert!(result.contains("\"/api/images/id-123\""));
    }

    #[test]
    fn test_replace_markdown_partial_dimensions_no_query_params() {
        let content = "![photo](photo.jpg)";
        let mut mapping = HashMap::new();
        mapping.insert(
            "photo.jpg".to_string(),
            ImageMapping {
                asset_id: "id-aaa".to_string(),
                width: Some(800),
                height: None,
            },
        );
        let result = replace_markdown_image_references(content, &mapping);
        // No dimensions injected when only one is present
        assert!(result.contains("](/api/images/id-aaa)"));
        assert!(!result.contains("?w="));
    }

    #[test]
    fn test_scan_markdown_no_match_for_links() {
        // Regular links should not be matched as images
        let content = "[click here](page.html)";
        let refs = scan_markdown_image_references(content);
        assert!(refs.is_empty());
    }
}
