#!/bin/bash

# Start Kafka Broker
# Make sure ZooKeeper is running before starting Kafka

KAFKA_VERSION="3.6.1"
SCALA_VERSION="2.13"
KAFKA_DIR="kafka_${SCALA_VERSION}-${KAFKA_VERSION}"

if [ ! -d "$KAFKA_DIR" ]; then
    echo "Error: Kafka not found. Please run ./setup-kafka.sh first"
    exit 1
fi

echo "Starting Kafka Broker..."
echo "Make sure ZooKeeper is running in another terminal!"
echo "Press Ctrl+C to stop Kafka"
echo ""

cd "$KAFKA_DIR"
bin/kafka-server-start.sh config/server.properties
