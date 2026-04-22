#!/usr/bin/env bash
set -euo pipefail

echo "Submitting test order..."
echo ""

RESPONSE=$(curl -s -X POST http://localhost:8081/orders \
  -H "Content-Type: application/json" \
  -d '{
    "customer_name": "Alice",
    "items": [
      {"name": "Widget", "price": 29.99, "quantity": 2},
      {"name": "Gadget", "price": 49.99, "quantity": 1}
    ]
  }')

echo "Response:"
echo "$RESPONSE" | python3 -m json.tool 2>/dev/null || echo "$RESPONSE"

echo ""
echo "Submitting a small order (to see different priority baggage)..."
echo ""

RESPONSE2=$(curl -s -X POST http://localhost:8081/orders \
  -H "Content-Type: application/json" \
  -d '{
    "customer_name": "Bob",
    "items": [
      {"name": "Sticker", "price": 2.50, "quantity": 3}
    ]
  }')

echo "Response:"
echo "$RESPONSE2" | python3 -m json.tool 2>/dev/null || echo "$RESPONSE2"

echo ""
echo "=== Observability UIs ==="
echo "Jaeger (traces):     http://localhost:16686"
echo "Prometheus (metrics): http://localhost:9090"
echo "Grafana (dashboards): http://localhost:3000  (admin/admin)"
echo ""
echo "=== Try these Prometheus queries ==="
echo "  otel_poc_http_requests_total"
echo "  otel_poc_order_value_dollars_bucket"
echo "  otel_poc_orders_in_store"
echo "  rate(otel_poc_http_request_duration_seconds_sum[1m])"
