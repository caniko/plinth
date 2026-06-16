use std::error::Error;
use std::sync::OnceLock;
use std::time::Duration;

use opentelemetry::trace::TracerProvider as _;
use opentelemetry::{KeyValue, global};
use opentelemetry_otlp::{WithExportConfig, WithTonicConfig};
use opentelemetry_sdk::trace as sdktrace;
use tracing::{info, warn};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

/// Global tracer provider stored for shutdown
static TRACER_PROVIDER: OnceLock<sdktrace::SdkTracerProvider> = OnceLock::new();

/// Configuration for observability initialization
#[derive(Debug, Clone)]
pub struct ObservabilityConfig {
    /// OTLP endpoint URL (e.g., `<https://openobserve.example.com:5081>`)
    /// If None or empty, OTLP export is disabled and logging goes to stdout only
    pub otlp_endpoint: Option<String>,

    /// OTLP headers for authentication (comma-separated key=value pairs)
    /// Example: "Authorization=Basic xxx,organization=default,stream=default"
    pub otlp_headers: Option<String>,

    /// Service name for telemetry spans
    pub service_name: String,

    /// Log level (RUST_LOG format)
    pub log_level: String,
}

impl Default for ObservabilityConfig {
    fn default() -> Self {
        Self {
            otlp_endpoint: std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").ok(),
            otlp_headers: std::env::var("OTEL_EXPORTER_OTLP_HEADERS").ok(),
            service_name: std::env::var("OTEL_SERVICE_NAME")
                .unwrap_or_else(|_| "plinth".to_string()),
            log_level: std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
        }
    }
}

impl ObservabilityConfig {
    /// Build from the unified TOML config (env overrides already applied)
    pub fn from_config(toml: &crate::config::ObservabilityTomlConfig) -> Self {
        Self {
            otlp_endpoint: if toml.otlp_endpoint.is_empty() {
                None
            } else {
                Some(toml.otlp_endpoint.clone())
            },
            otlp_headers: if toml.otlp_headers.is_empty() {
                None
            } else {
                Some(toml.otlp_headers.clone())
            },
            service_name: toml.service_name.clone(),
            log_level: toml.log_level.clone(),
        }
    }
}

/// Initialize observability with optional OTLP export
pub fn init_observability(config: ObservabilityConfig) -> Result<(), Box<dyn Error>> {
    // Create env filter for log level
    let env_filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(&config.log_level))
        .unwrap_or_else(|_| EnvFilter::new("info"));

    // Check if OTLP endpoint is configured
    if let Some(endpoint) = &config.otlp_endpoint {
        if !endpoint.is_empty() {
            info!(
                "Initializing observability with OTLP endpoint: {}",
                endpoint
            );

            // Initialize OTLP tracer provider
            match init_otlp_tracer_provider(config.clone()) {
                Ok(tracer_provider) => {
                    // Store for later shutdown
                    let _ = TRACER_PROVIDER.set(tracer_provider.clone());

                    // Set global tracer provider
                    global::set_tracer_provider(tracer_provider.clone());

                    // Create telemetry layer
                    let telemetry_layer = tracing_opentelemetry::layer()
                        .with_tracer(tracer_provider.tracer("server"));

                    // Create subscriber with both stdout and OTLP
                    tracing_subscriber::registry()
                        .with(env_filter)
                        .with(tracing_subscriber::fmt::layer().json())
                        .with(telemetry_layer)
                        .init();

                    info!("Observability initialized with OTLP export");
                    return Ok(());
                }
                Err(e) => {
                    warn!(
                        "Failed to initialize OTLP exporter: {}. Falling back to local logging.",
                        e
                    );
                }
            }
        } else {
            info!("OTEL_EXPORTER_OTLP_ENDPOINT is empty, using local logging only");
        }
    } else {
        info!("OTLP endpoint not configured, using local logging only");
    }

    // Fallback: Initialize with stdout logging only (no OTLP)
    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    info!("Observability initialized with local logging (OTLP disabled)");
    Ok(())
}

/// Initialize OTLP exporter and tracer provider
fn init_otlp_tracer_provider(
    config: ObservabilityConfig,
) -> Result<sdktrace::SdkTracerProvider, Box<dyn Error>> {
    let endpoint = config.otlp_endpoint.ok_or("OTLP endpoint not configured")?;

    // Parse headers if provided
    let headers = if let Some(headers_str) = config.otlp_headers {
        parse_otlp_headers(&headers_str)
    } else {
        std::collections::HashMap::new()
    };

    // Build metadata map from headers
    let mut metadata = tonic::metadata::MetadataMap::new();
    for (key, value) in headers {
        if let (Ok(key_name), Ok(value_ascii)) = (
            key.parse::<tonic::metadata::MetadataKey<tonic::metadata::Ascii>>(),
            value.parse::<tonic::metadata::MetadataValue<tonic::metadata::Ascii>>(),
        ) {
            metadata.insert(key_name, value_ascii);
        } else {
            // Never log the value: OTLP header values are credentials
            // (e.g. `Authorization=Basic …`).
            warn!("Failed to parse OTLP header: {}", key);
        }
    }

    // Create OTLP span exporter
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(&endpoint)
        .with_timeout(Duration::from_secs(10))
        .with_metadata(metadata)
        .build()?;

    // Build tracer provider with batch exporter
    let tracer_provider = sdktrace::SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(
            opentelemetry_sdk::Resource::builder_empty()
                .with_attributes(vec![KeyValue::new("service.name", config.service_name)])
                .build(),
        )
        .build();

    Ok(tracer_provider)
}

/// Parse OTLP headers from comma-separated key=value pairs
fn parse_otlp_headers(headers: &str) -> std::collections::HashMap<String, String> {
    headers
        .split(',')
        .filter_map(|pair| {
            let mut parts = pair.splitn(2, '=');
            match (parts.next(), parts.next()) {
                (Some(key), Some(value)) => {
                    Some((key.trim().to_string(), value.trim().to_string()))
                }
                _ => None,
            }
        })
        .collect()
}

/// Shutdown observability and flush telemetry
pub fn shutdown_observability() {
    info!("Shutting down observability and flushing telemetry...");
    if let Some(provider) = TRACER_PROVIDER.get()
        && let Err(e) = provider.shutdown()
    {
        warn!("Error shutting down tracer provider: {:?}", e);
    }
    info!("Observability shutdown complete");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_otlp_headers() {
        let input = "Authorization=Basic xxx,organization=default,stream=default";
        let headers = parse_otlp_headers(input);

        assert_eq!(headers.get("Authorization"), Some(&"Basic xxx".to_string()));
        assert_eq!(headers.get("organization"), Some(&"default".to_string()));
        assert_eq!(headers.get("stream"), Some(&"default".to_string()));
    }

    #[test]
    fn test_parse_otlp_headers_with_spaces() {
        let input = " key1 = value1 , key2 = value2 ";
        let headers = parse_otlp_headers(input);

        assert_eq!(headers.get("key1"), Some(&"value1".to_string()));
        assert_eq!(headers.get("key2"), Some(&"value2".to_string()));
    }

    #[test]
    fn test_config_from_env() {
        // SAFETY: This test runs serially and no other threads depend on these env vars
        unsafe {
            std::env::set_var("OTEL_EXPORTER_OTLP_ENDPOINT", "http://localhost:5081");
            std::env::set_var("OTEL_SERVICE_NAME", "test-service");
            std::env::set_var("RUST_LOG", "debug");
        }

        let config = ObservabilityConfig::default();

        assert_eq!(
            config.otlp_endpoint,
            Some("http://localhost:5081".to_string())
        );
        assert_eq!(config.service_name, "test-service");
        assert_eq!(config.log_level, "debug");

        // Cleanup
        // SAFETY: This test runs serially and no other threads depend on these env vars
        unsafe {
            std::env::remove_var("OTEL_EXPORTER_OTLP_ENDPOINT");
            std::env::remove_var("OTEL_SERVICE_NAME");
            std::env::remove_var("RUST_LOG");
        }
    }

    #[test]
    fn test_parse_otlp_headers_empty() {
        let headers = parse_otlp_headers("");
        assert!(headers.is_empty());
    }

    #[test]
    fn test_parse_otlp_headers_malformed() {
        let headers = parse_otlp_headers("no_equals_here,also_bad");
        assert!(headers.is_empty());
    }

    #[test]
    fn test_parse_otlp_headers_single() {
        let headers = parse_otlp_headers("key=value");
        assert_eq!(headers.len(), 1);
        assert_eq!(headers.get("key"), Some(&"value".to_string()));
    }
}
