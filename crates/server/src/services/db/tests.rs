use super::redact_db_url;

#[test]
fn redacts_userinfo() {
    assert_eq!(
        redact_db_url("postgres://plinth:plinth@localhost:5432/plinth"),
        "postgres://localhost:5432/plinth"
    );
}

#[test]
fn keeps_url_without_credentials() {
    assert_eq!(
        redact_db_url("postgres://localhost/plinth"),
        "postgres://localhost/plinth"
    );
}

#[test]
fn strips_query_string() {
    assert_eq!(
        redact_db_url("postgres://localhost/plinth?host=/run/sock&password=secret"),
        "postgres://localhost/plinth"
    );
}

#[test]
fn handles_unparseable_url() {
    assert_eq!(redact_db_url("not-a-url"), "<redacted>");
}
