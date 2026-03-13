# OpenTelemetry Data Lineage POC

Demonstrates **data lineage** using OpenTelemetry distributed tracing across three Rust microservices. A single order flows through ingestion, processing, and storage — and the full trace is visible in Jaeger.

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
                                 └────────┬───────┘
                                          │ OTLP
                                          ▼
                                 ┌────────────────┐
                                 │    Jaeger UI    │
                                 │ (port 16686)    │
                                 └────────────────┘
```

## Data Lineage — Two Levels

1. **Trace-level**: Jaeger shows a single distributed trace spanning all three services with parent/child spans, timing, and attributes.
2. **Payload-level**: The `lineage` field in the order JSON accumulates a human-readable audit trail of every system that touched the data.

## Quick Start

```bash
# Build and start everything
docker compose up --build

# Submit a test order (in another terminal)
./test-order.sh

# Open Jaeger UI
open http://localhost:16686
```

In Jaeger, select service **order-ingestion** and click **Find Traces**. You'll see a trace with spans across all three services:

- `ingest_order` (order-ingestion)
  - `process_order` (order-processor)
    - `validate_order`
    - `calculate_totals`
    - `store_order` (order-storage)
      - `persist_to_db`

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
