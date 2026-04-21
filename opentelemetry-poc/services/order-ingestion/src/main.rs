use axum::{extract::Json, extract::State, http::StatusCode, response::IntoResponse, routing::post, Router};
use common::{CreateOrderRequest, Order};
use opentelemetry::{global, metrics, KeyValue};
use tracing::{info, info_span, Instrument};

#[derive(Clone)]
struct AppState {
    requests_counter: metrics::Counter<u64>,
    request_duration: metrics::Histogram<f64>,
    orders_created: metrics::Counter<u64>,
}

#[tokio::main]
async fn main() {
    common::init_telemetry("order-ingestion");

    let meter = global::meter("order-ingestion");
    let state = AppState {
        requests_counter: meter
            .u64_counter("http_requests_total")
            .with_description("Total HTTP requests received")
            .init(),
        request_duration: meter
            .f64_histogram("http_request_duration_seconds")
            .with_description("HTTP request duration in seconds")
            .init(),
        orders_created: meter
            .u64_counter("orders_created_total")
            .with_description("Total orders created")
            .init(),
    };

    let app = Router::new()
        .route("/orders", post(create_order))
        .with_state(state);

    let addr = "0.0.0.0:8081";
    info!("order-ingestion listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();

    common::shutdown_telemetry();
}

async fn create_order(
    State(state): State<AppState>,
    Json(req): Json<CreateOrderRequest>,
) -> impl IntoResponse {
    let span = info_span!(
        "ingest_order",
        order.customer = %req.customer_name,
        order.item_count = req.items.len(),
        otel.status_code = tracing::field::Empty,
    );

    async move {
        let start = std::time::Instant::now();
        let labels = [KeyValue::new("method", "POST"), KeyValue::new("path", "/orders")];
        state.requests_counter.add(1, &labels);

        let order_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        info!(order_id = %order_id, "Ingesting new order");

        // Span event: order received
        tracing::info!(
            event_type = "order.received",
            order_id = %order_id,
            customer = %req.customer_name,
            item_count = req.items.len(),
            "Order received at ingestion gateway"
        );

        let order = Order {
            order_id: order_id.clone(),
            customer_name: req.customer_name.clone(),
            items: req.items,
            total: None,
            tax: None,
            status: "ingested".to_string(),
            lineage: vec![format!("ingested by order-ingestion at {}", now)],
        };

        // Forward to order-processor with trace context + baggage
        let client = reqwest::Client::new();
        let processor_url = std::env::var("ORDER_PROCESSOR_URL")
            .unwrap_or_else(|_| "http://order-processor:8082".to_string());

        // Inject trace context and OTel baggage (order source + customer info)
        let mut headers = http::HeaderMap::new();
        common::inject_context_with_baggage(
            &mut headers,
            vec![
                opentelemetry::KeyValue::new("order.source", "api-gateway"),
                opentelemetry::KeyValue::new("customer.name", req.customer_name.clone()),
            ],
        );

        let response = match client
            .post(format!("{}/process", processor_url))
            .headers(common::to_reqwest_headers(&headers))
            .json(&order)
            .send()
            .await
        {
            Ok(resp) if resp.status().is_success() => {
                let result: Order = resp.json().await.unwrap();
                state.orders_created.add(1, &[
                    KeyValue::new("customer", result.customer_name.clone()),
                    KeyValue::new("status", "success"),
                ]);
                // Span event: order completed
                tracing::info!(
                    event_type = "order.completed",
                    order_id = %result.order_id,
                    status = %result.status,
                    "Order processed successfully"
                );
                tracing::Span::current().record("otel.status_code", "OK");
                (StatusCode::OK, Json(result)).into_response()
            }
            Ok(resp) => {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                state.orders_created.add(1, &[
                    KeyValue::new("customer", req.customer_name.clone()),
                    KeyValue::new("status", "error"),
                ]);
                tracing::error!(
                    event_type = "order.failed",
                    %status, %body,
                    "Processor returned error"
                );
                tracing::Span::current().record("otel.status_code", "ERROR");
                (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
            }
            Err(e) => {
                state.orders_created.add(1, &[
                    KeyValue::new("customer", req.customer_name.clone()),
                    KeyValue::new("status", "error"),
                ]);
                tracing::error!(
                    event_type = "order.failed",
                    error = %e,
                    "Failed to reach order-processor"
                );
                tracing::Span::current().record("otel.status_code", "ERROR");
                (StatusCode::SERVICE_UNAVAILABLE, e.to_string()).into_response()
            }
        };

        let duration = start.elapsed().as_secs_f64();
        state.request_duration.record(duration, &labels);

        response
    }
    .instrument(span)
    .await
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install CTRL+C signal handler");
    info!("Shutting down...");
}
