use axum::{
    extract::{Json, Path, State},
    http::HeaderMap,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use common::Order;
use opentelemetry::{global, metrics, KeyValue};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, info_span, Instrument};
use tracing_opentelemetry::OpenTelemetrySpanExt;

type OrderStore = Arc<RwLock<HashMap<String, Order>>>;

#[derive(Clone)]
struct AppState {
    store: OrderStore,
    requests_counter: metrics::Counter<u64>,
    request_duration: metrics::Histogram<f64>,
    orders_stored: metrics::Counter<u64>,
    store_size: metrics::UpDownCounter<i64>,
}

#[tokio::main]
async fn main() {
    common::init_telemetry("order-storage");

    let meter = global::meter("order-storage");
    let state = AppState {
        store: Arc::new(RwLock::new(HashMap::new())),
        requests_counter: meter
            .u64_counter("http_requests_total")
            .with_description("Total HTTP requests received")
            .init(),
        request_duration: meter
            .f64_histogram("http_request_duration_seconds")
            .with_description("HTTP request duration in seconds")
            .init(),
        orders_stored: meter
            .u64_counter("orders_stored_total")
            .with_description("Total orders stored")
            .init(),
        store_size: meter
            .i64_up_down_counter("orders_in_store")
            .with_description("Current number of orders in the store")
            .init(),
    };

    let app = Router::new()
        .route("/store", post(store_order))
        .route("/orders/:id", get(get_order))
        .with_state(state);

    let addr = "0.0.0.0:8083";
    info!("order-storage listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();

    common::shutdown_telemetry();
}

async fn store_order(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(mut order): Json<Order>,
) -> impl IntoResponse {
    let parent_cx = common::extract_context(&headers);

    // Read baggage propagated through the entire chain
    let order_source = common::get_baggage(&parent_cx, "order.source")
        .unwrap_or_else(|| "unknown".to_string());
    let customer_priority = common::get_baggage(&parent_cx, "customer.priority")
        .unwrap_or_else(|| "unknown".to_string());
    let order_total_baggage = common::get_baggage(&parent_cx, "order.total")
        .unwrap_or_else(|| "0".to_string());

    let span = info_span!(
        "store_order",
        order.id = %order.order_id,
        db.system = "in_memory",
        order.source = %order_source,
        customer.priority = %customer_priority,
        order.total_from_baggage = %order_total_baggage,
        otel.status_code = tracing::field::Empty,
    );
    span.set_parent(parent_cx);

    async move {
        let start = std::time::Instant::now();
        let labels = [KeyValue::new("method", "POST"), KeyValue::new("path", "/store")];
        state.requests_counter.add(1, &labels);

        info!(
            order_id = %order.order_id,
            source = %order_source,
            priority = %customer_priority,
            "Storing order (baggage: source={}, priority={})",
            order_source,
            customer_priority
        );

        // Prepare order data
        let now = chrono::Utc::now().to_rfc3339();
        order.status = "stored".to_string();
        order
            .lineage
            .push(format!("stored by order-storage at {}", now));

        // Persist
        {
            let persist_span = info_span!(
                "persist_to_db",
                db.system = "in_memory",
                db.operation = "INSERT",
            );
            let _guard = persist_span.enter();

            let mut db = state.store.write().await;
            db.insert(order.order_id.clone(), order.clone());

            // Span event: persistence complete
            tracing::info!(
                event_type = "db.write.complete",
                order_id = %order.order_id,
                total_orders = db.len(),
                priority = %customer_priority,
                "Order persisted to in-memory store"
            );
        }

        state.orders_stored.add(1, &[
            KeyValue::new("priority", customer_priority.clone()),
            KeyValue::new("source", order_source.clone()),
        ]);
        state.store_size.add(1, &[]);

        tracing::Span::current().record("otel.status_code", "OK");

        let duration = start.elapsed().as_secs_f64();
        state.request_duration.record(duration, &labels);

        (StatusCode::OK, Json(order)).into_response()
    }
    .instrument(span)
    .await
}

async fn get_order(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let span = info_span!(
        "get_order",
        order.id = %id,
        db.system = "in_memory",
        db.operation = "SELECT",
        otel.status_code = tracing::field::Empty,
    );

    async move {
        let labels = [KeyValue::new("method", "GET"), KeyValue::new("path", "/orders/:id")];
        state.requests_counter.add(1, &labels);

        let db = state.store.read().await;
        match db.get(&id) {
            Some(order) => {
                tracing::info!(
                    event_type = "db.read.hit",
                    order_id = %id,
                    "Order found in store"
                );
                tracing::Span::current().record("otel.status_code", "OK");
                (StatusCode::OK, Json(order.clone())).into_response()
            }
            None => {
                tracing::warn!(
                    event_type = "db.read.miss",
                    order_id = %id,
                    "Order not found in store"
                );
                tracing::Span::current().record("otel.status_code", "ERROR");
                (StatusCode::NOT_FOUND, "Order not found").into_response()
            }
        }
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
