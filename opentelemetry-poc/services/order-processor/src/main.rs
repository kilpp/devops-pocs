use axum::{extract::Json, http::HeaderMap, http::StatusCode, response::IntoResponse, routing::post, Router};
use common::Order;
use tracing::{info, info_span, Instrument};
use tracing_opentelemetry::OpenTelemetrySpanExt;

const TAX_RATE: f64 = 0.085;

#[tokio::main]
async fn main() {
    common::init_tracer("order-processor");

    let app = Router::new().route("/process", post(process_order));

    let addr = "0.0.0.0:8082";
    info!("order-processor listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();

    common::shutdown_tracer();
}

async fn process_order(headers: HeaderMap, Json(mut order): Json<Order>) -> impl IntoResponse {
    let parent_cx = common::extract_context(&headers);

    let span = info_span!(
        "process_order",
        order.id = %order.order_id,
    );
    span.set_parent(parent_cx);

    async move {
        info!(order_id = %order.order_id, "Processing order");

        // Validate
        {
            let _validate_span = info_span!("validate_order").entered();
            if order.items.is_empty() {
                tracing::error!("Order has no items");
                return (StatusCode::BAD_REQUEST, "Order has no items").into_response();
            }
            if order.customer_name.is_empty() {
                tracing::error!("Order has no customer name");
                return (StatusCode::BAD_REQUEST, "Missing customer name").into_response();
            }
            info!("Validation passed");
        }

        // Calculate totals
        {
            let _calc_span = info_span!("calculate_totals").entered();
            let subtotal: f64 = order
                .items
                .iter()
                .map(|item| item.price * item.quantity as f64)
                .sum();
            let tax = (subtotal * TAX_RATE * 100.0).round() / 100.0;
            let total = ((subtotal + tax) * 100.0).round() / 100.0;

            order.total = Some(total);
            order.tax = Some(tax);
            info!(subtotal = subtotal, tax = tax, total = total, "Totals calculated");
        }

        let now = chrono::Utc::now().to_rfc3339();
        order.status = "processed".to_string();
        order.lineage.push(format!(
            "processed by order-processor at {} (tax={:.2}, total={:.2})",
            now,
            order.tax.unwrap_or(0.0),
            order.total.unwrap_or(0.0),
        ));

        // Forward to order-storage
        let client = reqwest::Client::new();
        let storage_url = std::env::var("ORDER_STORAGE_URL")
            .unwrap_or_else(|_| "http://order-storage:8083".to_string());

        let mut headers = http::HeaderMap::new();
        common::inject_context(&mut headers);

        let mut req_headers = reqwest::header::HeaderMap::new();
        for (k, v) in headers.iter() {
            if let Ok(name) = reqwest::header::HeaderName::from_bytes(k.as_str().as_bytes()) {
                if let Ok(val) = reqwest::header::HeaderValue::from_bytes(v.as_bytes()) {
                    req_headers.insert(name, val);
                }
            }
        }

        match client
            .post(format!("{}/store", storage_url))
            .headers(req_headers)
            .json(&order)
            .send()
            .await
        {
            Ok(resp) if resp.status().is_success() => {
                let result: Order = resp.json().await.unwrap();
                info!(order_id = %result.order_id, "Order stored successfully");
                (StatusCode::OK, Json(result)).into_response()
            }
            Ok(resp) => {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                tracing::error!(%status, %body, "Storage returned error");
                (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
            }
            Err(e) => {
                tracing::error!(error = %e, "Failed to reach order-storage");
                (StatusCode::SERVICE_UNAVAILABLE, e.to_string()).into_response()
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
