use opentelemetry::baggage::BaggageExt;
use opentelemetry::propagation::{Extractor, Injector, TextMapCompositePropagator};
use opentelemetry::{global, KeyValue};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::propagation::{BaggagePropagator, TraceContextPropagator};
use opentelemetry_sdk::{trace as sdktrace, Resource};
use serde::{Deserialize, Serialize};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

// --- Models ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderItem {
    pub name: String,
    pub price: f64,
    pub quantity: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub order_id: String,
    pub customer_name: String,
    pub items: Vec<OrderItem>,
    pub total: Option<f64>,
    pub tax: Option<f64>,
    pub status: String,
    /// Data lineage trail: records every system that touched this order and when.
    pub lineage: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateOrderRequest {
    pub customer_name: String,
    pub items: Vec<OrderItem>,
}

// --- Telemetry initialization (traces + metrics + baggage propagation) ---

pub fn init_telemetry(service_name: &str) {
    // Composite propagator: W3C Trace Context + W3C Baggage
    let composite = TextMapCompositePropagator::new(vec![
        Box::new(TraceContextPropagator::new()),
        Box::new(BaggagePropagator::new()),
    ]);
    global::set_text_map_propagator(composite);

    let resource = Resource::new(vec![
        KeyValue::new("service.name", service_name.to_string()),
        KeyValue::new("service.version", "0.2.0"),
        KeyValue::new("deployment.environment", "poc"),
    ]);

    let otlp_endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
        .unwrap_or_else(|_| "http://otel-collector:4318".to_string());

    // --- Traces ---
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .http()
                .with_endpoint(&otlp_endpoint),
        )
        .with_trace_config(
            sdktrace::Config::default().with_resource(resource.clone()),
        )
        .install_batch(opentelemetry_sdk::runtime::Tokio)
        .expect("Failed to initialize tracer");

    // --- Metrics ---
    let meter_provider = opentelemetry_otlp::new_pipeline()
        .metrics(opentelemetry_sdk::runtime::Tokio)
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .http()
                .with_endpoint(&otlp_endpoint),
        )
        .with_resource(resource)
        .build()
        .expect("Failed to initialize meter provider");
    global::set_meter_provider(meter_provider);

    // --- Tracing subscriber (bridges tracing crate → OpenTelemetry) ---
    let telemetry_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(telemetry_layer)
        .init();
}

pub fn shutdown_telemetry() {
    global::shutdown_tracer_provider();
}

// --- Context propagation helpers ---

struct HeaderExtractor<'a>(&'a http::HeaderMap);

impl<'a> Extractor for HeaderExtractor<'a> {
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).and_then(|v| v.to_str().ok())
    }

    fn keys(&self) -> Vec<&str> {
        self.0.keys().map(|k| k.as_str()).collect()
    }
}

struct HeaderInjector<'a>(&'a mut http::HeaderMap);

impl<'a> Injector for HeaderInjector<'a> {
    fn set(&mut self, key: &str, value: String) {
        if let Ok(name) = http::header::HeaderName::from_bytes(key.as_bytes()) {
            if let Ok(val) = http::header::HeaderValue::from_str(&value) {
                self.0.insert(name, val);
            }
        }
    }
}

/// Extract OpenTelemetry context (trace context + baggage) from incoming HTTP headers.
pub fn extract_context(headers: &http::HeaderMap) -> opentelemetry::Context {
    global::get_text_map_propagator(|propagator| propagator.extract(&HeaderExtractor(headers)))
}

/// Inject current span's OpenTelemetry context into outgoing HTTP headers (trace only).
pub fn inject_context(headers: &mut http::HeaderMap) {
    let cx = tracing::Span::current().context();
    global::get_text_map_propagator(|propagator| {
        propagator.inject_context(&cx, &mut HeaderInjector(headers));
    });
}

/// Inject trace context AND baggage into outgoing HTTP headers.
pub fn inject_context_with_baggage(headers: &mut http::HeaderMap, baggage: Vec<KeyValue>) {
    let span_cx = tracing::Span::current().context();
    let cx_with_baggage = span_cx.with_baggage(baggage);
    global::get_text_map_propagator(|propagator| {
        propagator.inject_context(&cx_with_baggage, &mut HeaderInjector(headers));
    });
}

/// Read a baggage value from an extracted OpenTelemetry context.
pub fn get_baggage(cx: &opentelemetry::Context, key: &str) -> Option<String> {
    let value = cx.baggage().get(key)?;
    Some(value.to_string())
}

/// Convert http::HeaderMap to reqwest::header::HeaderMap for outgoing requests.
pub fn to_reqwest_headers(headers: &http::HeaderMap) -> reqwest::header::HeaderMap {
    let mut req_headers = reqwest::header::HeaderMap::new();
    for (k, v) in headers.iter() {
        if let Ok(name) = reqwest::header::HeaderName::from_bytes(k.as_str().as_bytes()) {
            if let Ok(val) = reqwest::header::HeaderValue::from_bytes(v.as_bytes()) {
                req_headers.insert(name, val);
            }
        }
    }
    req_headers
}
