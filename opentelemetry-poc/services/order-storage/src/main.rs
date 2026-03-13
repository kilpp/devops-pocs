use axum::{
    extract::{Json, Path, State},
    http::HeaderMap,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use common::Order;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, info_span, Instrument};
use tracing_opentelemetry::OpenTelemetrySpanExt;

type OrderStore = Arc<RwLock<HashMap<String, Order>>>;

#[tokio::main]
async fn main() {
    common::init_tracer("order-storage");

    let store: OrderStore = Arc::new(RwLock::new(HashMap::new()));

    let app = Router::new()
        .route("/store", post(store_order))
        .route("/orders/:id", get(get_order))
        .with_state(store);

    let addr = "0.0.0.0:8083";
    info!("order-storage listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();

    common::shutdown_tracer();
}

async fn store_order(
    State(store): State<OrderStore>,
    headers: HeaderMap,
    Json(mut order): Json<Order>,
) -> impl IntoResponse {
    let parent_cx = common::extract_context(&headers);

    let span = info_span!(
        "store_order",
        order.id = %order.order_id,
        db.system = "in_memory",
    );
    span.set_parent(parent_cx);

    async move {
        info!(order_id = %order.order_id, "Storing order");

        // Prepare order data (sync, no await)
        let now = chrono::Utc::now().to_rfc3339();
        order.status = "stored".to_string();
        order
            .lineage
            .push(format!("stored by order-storage at {}", now));

        // Persist
        {
            let mut db = store.write().await;
            let persist_span = info_span!("persist_to_db");
            let _guard = persist_span.enter();
            db.insert(order.order_id.clone(), order.clone());
            info!(
                order_id = %order.order_id,
                total_orders = db.len(),
                "Order persisted"
            );
        }

        (StatusCode::OK, Json(order)).into_response()
    }
    .instrument(span)
    .await
}

async fn get_order(
    State(store): State<OrderStore>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let db = store.read().await;
    match db.get(&id) {
        Some(order) => (StatusCode::OK, Json(order.clone())).into_response(),
        None => (StatusCode::NOT_FOUND, "Order not found").into_response(),
    }
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install CTRL+C signal handler");
    info!("Shutting down...");
}
