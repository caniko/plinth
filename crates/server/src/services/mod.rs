// Business logic and services

pub mod db;
#[cfg(feature = "brick-blog")]
pub mod declarative_content;
pub mod markdown_processor;
pub mod migrations;
pub mod rows;

/// Truncate a string to at most `max_bytes`, snapping down to the nearest UTF-8
/// character boundary so the result is always a valid `&str`.
///
/// A naive `&s[..max_bytes]` panics when `max_bytes` falls in the middle of a
/// multi-byte codepoint (em-dashes, smart quotes, emoji, CJK). This is used to
/// bound text fed to the embedding model.
pub(crate) fn truncate_on_char_boundary(s: &str, max_bytes: usize) -> &str {
    if s.len() <= max_bytes {
        return s;
    }
    let mut end = max_bytes;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    &s[..end]
}

#[cfg(test)]
mod tests {
    use super::truncate_on_char_boundary;

    #[test]
    fn returns_whole_string_when_short() {
        assert_eq!(truncate_on_char_boundary("hello", 10), "hello");
        assert_eq!(truncate_on_char_boundary("hello", 5), "hello");
    }

    #[test]
    fn truncates_ascii_at_exact_byte() {
        assert_eq!(truncate_on_char_boundary("hello world", 5), "hello");
    }

    #[test]
    fn never_splits_a_codepoint() {
        // "a€b" — '€' is 3 bytes (indices 1..4). A cut at byte 2 or 3 must
        // snap back to byte 1, never panicking and never yielding invalid UTF-8.
        let s = "a€b";
        assert_eq!(truncate_on_char_boundary(s, 2), "a");
        assert_eq!(truncate_on_char_boundary(s, 3), "a");
        assert_eq!(truncate_on_char_boundary(s, 4), "a€");
    }

    #[test]
    fn handles_emoji() {
        let s = "👍👍👍"; // each is 4 bytes
        assert_eq!(truncate_on_char_boundary(s, 5), "👍");
        assert_eq!(truncate_on_char_boundary(s, 8), "👍👍");
    }
}
