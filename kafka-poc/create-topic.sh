#!/bin/bash

# Create a Kafka Topic
# Usage: ./create-topic.sh <topic-name>

KAFKA_VERSION="3.6.1"
SCALA_VERSION="2.13"
KAFKA_DIR="kafka_${SCALA_VERSION}-${KAFKA_VERSION}"

if [ ! -d "$KAFKA_DIR" ]; then
    echo "Error: Kafka not found. Please run ./setup-kafka.sh first"
    exit 1
fi

if [ -z "$1" ]; then
    TOPIC_NAME="test-topic"
    echo "No topic name provided. Using default: $TOPIC_NAME"
else
    TOPIC_NAME="$1"
fi

echo "Creating topic: $TOPIC_NAME"
echo ""

cd "$KAFKA_DIR"
bin/kafka-topics.sh --create \
    --topic "$TOPIC_NAME" \
    --bootstrap-server localhost:9092 \
    --partitions 3 \
    --replication-factor 1

echo ""
echo "✓ Topic '$TOPIC_NAME' created successfully!"
echo ""
echo "To list all topics, run:"
echo "  cd $KAFKA_DIR && bin/kafka-topics.sh --list --bootstrap-server localhost:9092"
