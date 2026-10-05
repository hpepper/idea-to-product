use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{
    Resource, logs::SdkLoggerProvider, metrics::SdkMeterProvider, trace::SdkTracerProvider,
};
use opentelemetry_semantic_conventions::resource::{
    K8S_NAMESPACE_NAME, SERVICE_NAME, SERVICE_VERSION,
};

pub fn init_tracer(
    service_name: &str,
    service_version: &str,
) -> Result<SdkTracerProvider, Box<dyn std::error::Error + Send + Sync + 'static>> {
    let endpoint = std::env::var("OTLP_TRACE_BACKEND_URL")
        .unwrap_or_else(|_| "http://localhost:4317".to_string());

    let otlp_exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(endpoint)
        .with_timeout(std::time::Duration::from_secs(3))
        .build()?;

    let resource = Resource::builder_empty()
        .with_attribute(KeyValue::new("service.name", service_name.to_string()))
        .with_attribute(KeyValue::new("service.version", service_version.to_string()))
        .with_attribute(KeyValue::new("service.namespace", "aws2xml"))
        .build();

    // No stdout exporter: one multi-line dump per span floods the console.
    let tracer_provider = SdkTracerProvider::builder()
        .with_batch_exporter(otlp_exporter)
        .with_resource(resource)
        .build();

    Ok(tracer_provider)
}

pub fn init_logger(service_name: &str, service_version: &str) -> SdkLoggerProvider {
    let endpoint_url = std::env::var("OTLP_LOGGING_BACKEND_URL")
        .unwrap_or_else(|_| "http://localhost:4317".to_string());

    let resource = Resource::builder_empty()
        .with_attribute(KeyValue::new(SERVICE_NAME, service_name.to_string()))
        .with_attribute(KeyValue::new(SERVICE_VERSION, service_version.to_string()))
        .with_attribute(KeyValue::new(K8S_NAMESPACE_NAME, "aws2xml"))
        .build();

    let otlp_exporter = opentelemetry_otlp::LogExporter::builder()
        .with_tonic()
        .with_endpoint(endpoint_url)
        .build()
        .expect("Failed to create log exporter");

    // No stdout exporter: main.rs prints a one-line console message per log record instead.
    let log_provider = SdkLoggerProvider::builder()
        .with_batch_exporter(otlp_exporter)
        .with_resource(resource)
        .build();

    log_provider
}

/// Emit a log record directly through the given provider's OTel native API.
/// Because every service holds its own `SdkLoggerProvider` with its own
/// `service.name` resource attribute, logs in Loki will carry the correct
/// service label regardless of how many services run in the same process.
pub fn service_log(
    provider: &SdkLoggerProvider,
    severity: opentelemetry::logs::Severity,
    message: String,
) {
    use opentelemetry::logs::{AnyValue, Logger, LogRecord, LoggerProvider};
    let logger = provider.logger("service-logger");
    let mut record = logger.create_log_record();
    record.set_severity_number(severity);
    record.set_body(AnyValue::from(message));
    logger.emit(record);
}

pub fn init_metrics(
    service_name: &str,
    service_version: &str,
) -> Result<SdkMeterProvider, Box<dyn std::error::Error + Send + Sync + 'static>> {
    let endpoint = std::env::var("OTLP_METRICS_BACKEND_URL")
        .unwrap_or_else(|_| "http://localhost:4317".to_string());

    let otlp_exporter = opentelemetry_otlp::MetricExporter::builder()
        .with_tonic()
        .with_endpoint(endpoint)
        .with_timeout(std::time::Duration::from_secs(3))
        .build()?;

    let resource = Resource::builder_empty()
        .with_attribute(KeyValue::new("service.name", service_name.to_string()))
        .with_attribute(KeyValue::new("service.version", service_version.to_string()))
        .with_attribute(KeyValue::new("service.namespace", "aws2xml"))
        .build();

    let metrics_provider = SdkMeterProvider::builder()
        .with_periodic_exporter(otlp_exporter)
        .with_resource(resource)
        .build();

    Ok(metrics_provider)
}

pub fn emit_log(
    service_name: &str,
    level: &str,
    operation: &str,
    message: &str,
    attributes: Vec<KeyValue>,
) {
    let endpoint = std::env::var("OTLP_LOGGING_BACKEND_URL")
        .unwrap_or_else(|_| "http://localhost:4317".to_string());

    let timestamp = chrono::Utc::now().to_rfc3339();
    let attrs_str = attributes
        .iter()
        .map(|kv| format!("\"{}\":\"{}\"", kv.key, kv.value.as_str()))
        .collect::<Vec<_>>()
        .join(",");

    let log_entry = if attrs_str.is_empty() {
        format!(
            "{{\"timestamp\":\"{}\",\"level\":\"{}\",\"service\":\"{}\",\"operation\":\"{}\",\"message\":\"{}\",\"otlp_endpoint\":\"{}\"}}",
            timestamp, level, service_name, operation, message, endpoint
        )
    } else {
        format!(
            "{{\"timestamp\":\"{}\",\"level\":\"{}\",\"service\":\"{}\",\"operation\":\"{}\",\"message\":\"{}\",{},\"otlp_endpoint\":\"{}\"}}",
            timestamp, level, service_name, operation, message, attrs_str, endpoint
        )
    };

    println!("{}", log_entry);
    log::info!(target: "emit-log", "{}", log_entry);
}
