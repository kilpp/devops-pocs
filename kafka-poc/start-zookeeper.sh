#!/bin/bash

# Start ZooKeeper
# ZooKeeper is required for Kafka to manage cluster coordination

KAFKA_VERSION="3.6.1"
SCALA_VERSION="2.13"
KAFKA_DIR="kafka_${SCALA_VERSION}-${KAFKA_VERSION}"

if [ ! -d "$KAFKA_DIR" ]; then
    echo "Error: Kafka not found. Please run ./setup-kafka.sh first"
    exit 1
fi

echo "Starting ZooKeeper..."
echo "Press Ctrl+C to stop ZooKeeper"
echo ""

cd "$KAFKA_DIR"
bin/zookeeper-server-start.sh config/zookeeper.properties
