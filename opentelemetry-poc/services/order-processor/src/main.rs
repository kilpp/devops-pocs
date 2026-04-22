use axum::{
    extract::{Json, State},
    http::HeaderMap,
    http::StatusCode,
    response::IntoResponse,
    routing::post,
    Router,
};
use common::Order;
use opentelemetry::{global, metrics, KeyValue};
use tracing::{info, info_span, Instrument};
use tracing_opentelemetry::OpenTelemetrySpanExt;

const TAX_RATE: f64 = 0.085;

#[derive(Clone)]
struct AppState {
    requests_counter: metrics::Counter<u64>,
    request_duration: metrics::Histogram<f64>,
    orders_processed: metrics::Counter<u64>,
    order_value: metrics::Histogram<f64>,
}

#[tokio::main]
async fn main() {
    common::init_telemetry("order-processor");

    let meter = global::meter("order-processor");
    let state = AppState {
        requests_counter: meter
            .u64_counter("http_requests_total")
            .with_description("Total HTTP requests received")
            .init(),
        request_duration: meter
            .f64_histogram("http_request_duration_seconds")
            .with_description("HTTP request duration in seconds")
            .init(),
        orders_processed: meter
            .u64_counter("orders_processed_total")
            .with_description("Total orders processed")
            .init(),
        order_value: meter
            .f64_histogram("order_value_dollars")
            .with_description("Distribution of order total values in dollars")
            .init(),
    };

    let app = Router::new()
        .route("/process", post(process_order))
        .with_state(state);

    let addr = "0.0.0.0:8082";
    info!("order-processor listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();

    common::shutdown_telemetry();
}

async fn process_order(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(mut order): Json<Order>,
) -> impl IntoResponse {
    let parent_cx = common::extract_context(&headers);

    // Read baggage propagated from upstream
    let order_source = common::get_baggage(&parent_cx, "order.source")
        .unwrap_or_else(|| "unknown".to_string());
    let customer_from_baggage = common::get_baggage(&parent_cx, "customer.name")
        .unwrap_or_else(|| "unknown".to_string());

    let span = info_span!(
        "process_order",
        order.id = %order.order_id,
        order.source = %order_source,
        customer.from_baggage = %customer_from_baggage,
        otel.status_code = tracing::field::Empty,
    );
    span.set_parent(parent_cx);

    async move {
        let start = std::time::Instant::now();
        let labels = [KeyValue::new("method", "POST"), KeyValue::new("path", "/process")];
        state.requests_counter.add(1, &labels);

        info!(
            order_id = %order.order_id,
            order_source = %order_source,
            "Processing order (source from baggage: {})",
            order_source
        );

        // Validate
        {
            let _validate_span = info_span!("validate_order").entered();
            if order.items.is_empty() {
                tracing::error!(
                    event_type = "validation.failed",
                    reason = "no_items",
                    "Order has no items"
                );
                state.orders_processed.add(1, &[KeyValue::new("status", "validation_failed")]);
                tracing::Span::current().record("otel.status_code", "ERROR");
                let duration = start.elapsed().as_secs_f64();
                state.request_duration.record(duration, &labels);
                return (StatusCode::BAD_REQUEST, "Order has no items").into_response();
            }
            if order.customer_name.is_empty() {
                tracing::error!(
                    event_type = "validation.failed",
                    reason = "no_customer",
                    "Order has no customer name"
                );
                state.orders_processed.add(1, &[KeyValue::new("status", "validation_failed")]);
                tracing::Span::current().record("otel.status_code", "ERROR");
                let duration = start.elapsed().as_secs_f64();
                state.request_duration.record(duration, &labels);
                return (StatusCode::BAD_REQUEST, "Missing customer name").into_response();
            }
            // Span event: validation passed
            tracing::info!(
                event_type = "validation.passed",
                item_count = order.items.len(),
                customer = %order.customer_name,
                "Order validation passed"
            );
        }

        // Calculate totals
        let total;
        {
            let _calc_span = info_span!("calculate_totals").entered();
            let subtotal: f64 = order
                .items
                .iter()
                .map(|item| item.price * item.quantity as f64)
                .sum();
            let tax = (subtotal * TAX_RATE * 100.0).round() / 100.0;
            total = ((subtotal + tax) * 100.0).round() / 100.0;

            order.total = Some(total);
            order.tax = Some(tax);

            // Span event: calculation complete
            tracing::info!(
                event_type = "calculation.complete",
                subtotal = subtotal,
                tax = tax,
                total = total,
                tax_rate = TAX_RATE,
                "Totals calculated"
            );
        }

        // Record order value metric
        state.order_value.record(total, &[
            KeyValue::new("source", order_source.clone()),
        ]);

        let now = chrono::Utc::now().to_rfc3339();
        order.status = "processed".to_string();
        order.lineage.push(format!(
            "processed by order-processor at {} (tax={:.2}, total={:.2})",
            now,
            order.tax.unwrap_or(0.0),
            order.total.unwrap_or(0.0),
        ));

        // Determine customer priority based on order value (for baggage propagation)
        let priority = if total > 100.0 { "high" } else { "standard" };

        // Forward to order-storage with trace context + enriched baggage
        let client = reqwest::Client::new();
        let storage_url = std::env::var("ORDER_STORAGE_URL")
            .unwrap_or_else(|_| "http://order-storage:8083".to_string());

        let mut headers = http::HeaderMap::new();
        common::inject_context_with_baggage(
            &mut headers,
            vec![
                opentelemetry::KeyValue::new("order.source", order_source.clone()),
                opentelemetry::KeyValue::new("customer.name", customer_from_baggage.clone()),
                opentelemetry::KeyValue::new("customer.priority", priority),
                opentelemetry::KeyValue::new("order.total", total.to_string()),
            ],
        );

        let response = match client
            .post(format!("{}/store", storage_url))
            .headers(common::to_reqwest_headers(&headers))
            .json(&order)
            .send()
            .await
        {
            Ok(resp) if resp.status().is_success() => {
                let result: Order = resp.json().await.unwrap();
                state.orders_processed.add(1, &[KeyValue::new("status", "success")]);
                tracing::info!(
                    event_type = "processing.complete",
                    order_id = %result.order_id,
                    "Order stored successfully"
                );
                tracing::Span::current().record("otel.status_code", "OK");
                (StatusCode::OK, Json(result)).into_response()
            }
            Ok(resp) => {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                state.orders_processed.add(1, &[KeyValue::new("status", "storage_error")]);
                tracing::error!(
                    event_type = "storage.failed",
                    %status, %body,
                    "Storage returned error"
                );
                tracing::Span::current().record("otel.status_code", "ERROR");
                (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
            }
            Err(e) => {
                state.orders_processed.add(1, &[KeyValue::new("status", "storage_unreachable")]);
                tracing::error!(
                    event_type = "storage.unreachable",
                    error = %e,
                    "Failed to reach order-storage"
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
