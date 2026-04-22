# OpenTelemetry POC

Demonstrates **distributed tracing**, **metrics**, **baggage propagation**, and **span events** using OpenTelemetry across three Rust microservices. A single order flows through ingestion, processing, and storage — with full observability via Jaeger, Prometheus, and Grafana.

## Architecture

```
POST /orders
     │
     ▼
┌─────────────────┐     HTTP      ┌──────────────────┐     HTTP      ┌────────────────┐
│ order-ingestion │──────────────►│ order-processor  │──────────────►│ order-storage  │
│   (port 8081)   │              │   (port 8082)    │              │   (port 8083)  │
└────────┬────────┘              └────────┬─────────┘              └────────┬───────┘
         │                                │                                 │
         └───────────── OTLP ────────────►│◄── OTLP ────────────────────────┘
                                          ▼
                                 ┌────────────────┐
                                 │ OTel Collector  │
                                 │  (port 4317)    │
                                 └───┬────────┬───┘
                                     │        │
                            OTLP     │        │  Prometheus
                                     ▼        ▼
                              ┌──────────┐ ┌────────────┐
                              │  Jaeger  │ │ Prometheus  │
                              │  (16686) │ │   (9090)    │
                              └──────────┘ └──────┬─────┘
                                                  │
                                           ┌──────▼─────┐
                                           │  Grafana   │
                                           │   (3000)   │
                                           └────────────┘
```

## OpenTelemetry Features Demonstrated

### 1. Distributed Tracing
Traces propagated across all three services using W3C Trace Context. Jaeger shows the full trace with parent/child spans:

- `ingest_order` (order-ingestion)
  - `process_order` (order-processor)
    - `validate_order`
    - `calculate_totals`
    - `store_order` (order-storage)
      - `persist_to_db`

### 2. Metrics
Each service exports metrics via OTLP to the collector, which exposes them for Prometheus scraping:

| Metric | Type | Service | Description |
|--------|------|---------|-------------|
| `http_requests_total` | Counter | All | Total HTTP requests by method/path |
| `http_request_duration_seconds` | Histogram | All | Request latency distribution |
| `orders_created_total` | Counter | Ingestion | Orders created by customer/status |
| `orders_processed_total` | Counter | Processor | Orders processed by status |
| `order_value_dollars` | Histogram | Processor | Distribution of order values |
| `orders_stored_total` | Counter | Storage | Orders stored by priority/source |
| `orders_in_store` | UpDownCounter | Storage | Current store size |

### 3. Baggage Propagation
W3C Baggage headers carry metadata across service boundaries without modifying the payload:

```
order-ingestion sets:     order.source=api-gateway, customer.name=Alice
order-processor adds:     customer.priority=high, order.total=119.31
order-storage reads:      all baggage values → used as span attributes
```

### 4. Span Events
Structured events within spans track key moments:
- `order.received` — order arrives at ingestion
- `validation.passed` / `validation.failed` — validation outcome
- `calculation.complete` — totals computed with subtotal, tax, total
- `db.write.complete` — persistence confirmed
- `order.completed` / `order.failed` — final outcome

### 5. Span Status
Explicit `otel.status_code` set on every span:
- `OK` on success
- `ERROR` on validation failures, downstream errors, or unreachable services

### 6. Resource Attributes
Each service reports richer resource attributes:
- `service.name` — service identity
- `service.version` — `0.2.0`
- `deployment.environment` — `poc`

### 7. Data Lineage (Payload-level)
The `lineage` field in the order JSON accumulates a human-readable audit trail alongside the OTel traces.

## Quick Start

```bash
# Build and start everything
docker compose up --build

# Submit test orders (in another terminal)
./test-order.sh

# Open observability UIs
open http://localhost:16686   # Jaeger (traces)
open http://localhost:9090    # Prometheus (metrics)
open http://localhost:3000    # Grafana (dashboards, admin/admin)
```

## Example Prometheus Queries

```promql
# Request rate by service
rate(otel_poc_http_requests_total[1m])

# P95 request latency
histogram_quantile(0.95, rate(otel_poc_http_request_duration_seconds_bucket[5m]))

# Order value distribution
otel_poc_order_value_dollars_bucket

# Current orders in store
otel_poc_orders_in_store
```

## Example Response

```json
{
  "order_id": "a1b2c3d4-...",
  "customer_name": "Alice",
  "items": [
    {"name": "Widget", "price": 29.99, "quantity": 2},
    {"name": "Gadget", "price": 49.99, "quantity": 1}
  ],
  "total": 119.31,
  "tax": 9.35,
  "status": "stored",
  "lineage": [
    "ingested by order-ingestion at 2026-03-12T10:00:00Z",
    "processed by order-processor at 2026-03-12T10:00:00Z (tax=9.35, total=119.31)",
    "stored by order-storage at 2026-03-12T10:00:00Z"
  ]
}
```

## Cleanup

```bash
docker compose down
```
