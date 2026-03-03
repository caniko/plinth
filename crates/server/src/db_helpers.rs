use serde::de::DeserializeOwned;
use surrealdb::IndexedResults;
use surrealdb::types::Value;

/// Deserialize SurrealDB query results via Value::into_json_value().
///
/// SurrealDB 3.0's `SurrealValue` impl for `serde_json::Value` can't convert
/// native Datetime or RecordId types. We instead take the raw `Value`,
/// convert via `into_json_value()` (which handles all types), then deserialize.
pub fn take_as<T: DeserializeOwned>(
    result: &mut IndexedResults,
    idx: usize,
) -> Result<Vec<T>, String> {
    let value: Value = result.take(idx).map_err(|e| e.to_string())?;
    let values = match value {
        Value::Array(arr) => arr.into_vec(),
        Value::None => return Ok(vec![]),
        other => vec![other],
    };
    values
        .into_iter()
        .map(|v| {
            let json = v.into_json_value();
            serde_json::from_value(json).map_err(|e| e.to_string())
        })
        .collect()
}

/// Like `take_as` but returns `Option<T>` (first result or None).
pub fn take_as_opt<T: DeserializeOwned>(
    result: &mut IndexedResults,
    idx: usize,
) -> Result<Option<T>, String> {
    let items: Vec<T> = take_as(result, idx)?;
    Ok(items.into_iter().next())
}
