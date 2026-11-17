//! JSON Schema and runtime validator agreement.

use std::path::{Path, PathBuf};

use serde_json::Value;

fn crate_file(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(relative)
}

fn compile_schema() -> (boon::Schemas, boon::SchemaIndex) {
    let path = crate_file("progress.schema.json");
    let location = path.to_str().expect("schema path is UTF-8");
    let text = std::fs::read_to_string(&path).expect("schema is readable");
    let schema: Value = serde_json::from_str(&text).expect("schema is JSON");

    let mut schemas = boon::Schemas::new();
    let mut compiler = boon::Compiler::new();
    compiler
        .add_resource(location, schema)
        .expect("schema resource location is absolute");
    let index = compiler
        .compile(location, &mut schemas)
        .expect("schema compiles");
    (schemas, index)
}

fn instance(source: &str) -> Value {
    let value: toml::Value = toml::from_str(source).expect("fixture is valid TOML");
    serde_json::to_value(value).expect("TOML value converts to JSON")
}

fn validates(schemas: &boon::Schemas, index: boon::SchemaIndex, source: &str) -> bool {
    schemas.validate(&instance(source), index).is_ok()
}

fn fixture(name: &str) -> String {
    std::fs::read_to_string(crate_file(&format!("tests/fixtures/{name}")))
        .expect("fixture is readable")
}

#[test]
fn schema_valid_example() {
    let (schemas, index) = compile_schema();
    let path = crate_file("examples/progress.toml");
    let source = std::fs::read_to_string(&path).expect("example is readable");

    assert!(
        validates(&schemas, index, &source),
        "the example must satisfy the schema"
    );
    plinth_progress::parse(&source, &path).expect("the example must pass the runtime validator");
}

#[test]
fn schema_rejects_invalid_fixtures() {
    let (schemas, index) = compile_schema();

    let schema_invalid = [
        "schema/unknown_key.toml",
        "schema/missing_field.toml",
        "schema/wrong_type.toml",
        "schema/empty_string.toml",
        "schema/bad_status.toml",
        "schema/bad_version.toml",
        "schema/bad_date.toml",
        "schema/empty_entries.toml",
    ];
    for name in schema_invalid {
        let source = fixture(name);
        assert!(
            !validates(&schemas, index, &source),
            "{name} must fail the schema"
        );
        assert!(
            plinth_progress::parse(&source, Path::new(name)).is_err(),
            "{name} must fail the runtime validator"
        );
    }

    // Semantic-only rules pass the schema and fail the runtime validator.
    let semantic_only = [
        "semantic/bad_calendar_date.toml",
        "semantic/unordered_versions.toml",
        "semantic/equal_precedence_build.toml",
        "semantic/duplicate_versions.toml",
        "semantic/increasing_dates.toml",
    ];
    for name in semantic_only {
        let source = fixture(name);
        assert!(
            validates(&schemas, index, &source),
            "{name} is structurally valid and must pass the schema"
        );
        assert!(
            plinth_progress::parse(&source, Path::new(name)).is_err(),
            "{name} must fail the runtime validator"
        );
    }
}
