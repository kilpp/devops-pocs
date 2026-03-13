use axum::{extract::Json, http::StatusCode, response::IntoResponse, routing::post, Router};
use common::{CreateOrderRequest, Order};
use tracing::{info, info_span, Instrument};

#[tokio::main]
async fn main() {
    common::init_tracer("order-ingestion");

    let app = Router::new().route("/orders", post(create_order));

    let addr = "0.0.0.0:8081";
    info!("order-ingestion listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();

    common::shutdown_tracer();
}

async fn create_order(Json(req): Json<CreateOrderRequest>) -> impl IntoResponse {
    let span = info_span!(
        "ingest_order",
        order.customer = %req.customer_name,
        order.item_count = req.items.len(),
    );

    async move {
        let order_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        info!(order_id = %order_id, "Ingesting new order");

        let order = Order {
            order_id: order_id.clone(),
            customer_name: req.customer_name.clone(),
            items: req.items,
            total: None,
            tax: None,
            status: "ingested".to_string(),
            lineage: vec![format!("ingested by order-ingestion at {}", now)],
        };

        // Forward to order-processor with trace context
        let client = reqwest::Client::new();
        let processor_url = std::env::var("ORDER_PROCESSOR_URL")
            .unwrap_or_else(|_| "http://order-processor:8082".to_string());

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
            .post(format!("{}/process", processor_url))
            .headers(req_headers)
            .json(&order)
            .send()
            .await
        {
            Ok(resp) if resp.status().is_success() => {
                let result: Order = resp.json().await.unwrap();
                info!(order_id = %result.order_id, status = %result.status, "Order processed successfully");
                (StatusCode::OK, Json(result)).into_response()
            }
            Ok(resp) => {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                tracing::error!(%status, %body, "Processor returned error");
                (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
            }
            Err(e) => {
                tracing::error!(error = %e, "Failed to reach order-processor");
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
