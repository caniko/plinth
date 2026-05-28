use serde::de;

/// Deserializer that accepts a string value or gracefully returns None for any
/// other type (e.g. an opaque database record ID type).
/// This allows the same struct to work with JSON APIs and database query results.
pub(crate) fn deserialize_flexible_id<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: de::Deserializer<'de>,
{
    deserializer.deserialize_option(FlexibleIdVisitor)
}

struct FlexibleIdVisitor;

impl<'de> de::Visitor<'de> for FlexibleIdVisitor {
    type Value = Option<String>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a string, null, or any type")
    }

    fn visit_none<E: de::Error>(self) -> Result<Self::Value, E> {
        Ok(None)
    }

    fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
        Ok(None)
    }

    fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
        Ok(Some(v.to_string()))
    }

    fn visit_string<E: de::Error>(self, v: String) -> Result<Self::Value, E> {
        Ok(Some(v))
    }

    fn visit_some<D2: de::Deserializer<'de>>(self, d: D2) -> Result<Self::Value, D2::Error> {
        d.deserialize_any(AnyToStringVisitor)
    }
}

/// Inner visitor that converts any non-null value to a string or drops it.
struct AnyToStringVisitor;

impl<'de> de::Visitor<'de> for AnyToStringVisitor {
    type Value = Option<String>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("any value")
    }

    fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
        Ok(Some(v.to_string()))
    }

    fn visit_string<E: de::Error>(self, v: String) -> Result<Self::Value, E> {
        Ok(Some(v))
    }

    // For non-string types, return None.
    fn visit_bool<E: de::Error>(self, _: bool) -> Result<Self::Value, E> {
        Ok(None)
    }
    fn visit_i64<E: de::Error>(self, _: i64) -> Result<Self::Value, E> {
        Ok(None)
    }
    fn visit_u64<E: de::Error>(self, _: u64) -> Result<Self::Value, E> {
        Ok(None)
    }
    fn visit_f64<E: de::Error>(self, _: f64) -> Result<Self::Value, E> {
        Ok(None)
    }
    fn visit_bytes<E: de::Error>(self, _: &[u8]) -> Result<Self::Value, E> {
        Ok(None)
    }
    fn visit_none<E: de::Error>(self) -> Result<Self::Value, E> {
        Ok(None)
    }
    fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
        Ok(None)
    }
    fn visit_map<A: de::MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
        while map
            .next_entry::<de::IgnoredAny, de::IgnoredAny>()?
            .is_some()
        {}
        Ok(None)
    }
    fn visit_seq<A: de::SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        while seq.next_element::<de::IgnoredAny>()?.is_some() {}
        Ok(None)
    }
    fn visit_newtype_struct<D2: de::Deserializer<'de>>(
        self,
        d: D2,
    ) -> Result<Self::Value, D2::Error> {
        d.deserialize_any(self)
    }
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct Wrapper {
        #[serde(default, deserialize_with = "super::deserialize_flexible_id")]
        id: Option<String>,
    }

    fn parse(json: &str) -> Option<String> {
        serde_json::from_str::<Wrapper>(json).unwrap().id
    }

    #[test]
    fn string_id_is_kept() {
        assert_eq!(parse(r#"{"id": "abc"}"#), Some("abc".to_string()));
    }

    #[test]
    fn null_id_is_none() {
        assert_eq!(parse(r#"{"id": null}"#), None);
    }

    #[test]
    fn missing_id_is_none() {
        assert_eq!(parse(r#"{}"#), None);
    }

    #[test]
    fn numeric_id_is_none() {
        // A bare DB integer/record id maps to None rather than failing.
        assert_eq!(parse(r#"{"id": 42}"#), None);
        assert_eq!(parse(r#"{"id": 1.5}"#), None);
    }

    #[test]
    fn bool_id_is_none() {
        assert_eq!(parse(r#"{"id": true}"#), None);
    }

    #[test]
    fn object_id_is_none() {
        // e.g. an opaque {"tb": ..., "id": ...} record id object.
        assert_eq!(parse(r#"{"id": {"tb": "x", "id": "y"}}"#), None);
    }

    #[test]
    fn array_id_is_none() {
        assert_eq!(parse(r#"{"id": [1, 2, 3]}"#), None);
    }
}
