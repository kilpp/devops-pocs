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
echo "Open Jaeger UI at: http://localhost:16686"
echo "Search for service 'order-ingestion' to see the full trace."
