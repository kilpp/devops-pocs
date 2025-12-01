#!/bin/bash

# Kafka Setup Script
# This script downloads and sets up Kafka with ZooKeeper

set -e

KAFKA_VERSION="3.6.1"
SCALA_VERSION="2.13"
KAFKA_DIR="kafka_${SCALA_VERSION}-${KAFKA_VERSION}"
# Try multiple mirror URLs
KAFKA_DOWNLOAD_URLS=(
    "https://archive.apache.org/dist/kafka/${KAFKA_VERSION}/${KAFKA_DIR}.tgz"
    "https://dlcdn.apache.org/kafka/${KAFKA_VERSION}/${KAFKA_DIR}.tgz"
    "https://downloads.apache.org/kafka/${KAFKA_VERSION}/${KAFKA_DIR}.tgz"
)

echo "==================================="
echo "Kafka Setup Script"
echo "==================================="

# Check if Kafka is already downloaded
if [ -d "$KAFKA_DIR" ]; then
    echo "✓ Kafka directory already exists: $KAFKA_DIR"
else
    echo "→ Downloading Kafka ${KAFKA_VERSION}..."
    
    DOWNLOAD_SUCCESS=false
    
    # Try each mirror URL
    for KAFKA_DOWNLOAD_URL in "${KAFKA_DOWNLOAD_URLS[@]}"; do
        echo "→ Trying: $KAFKA_DOWNLOAD_URL"
        
        # Download with progress and follow redirects
        if curl -L -o "${KAFKA_DIR}.tgz" "$KAFKA_DOWNLOAD_URL"; then
            # Verify download size (should be > 10MB)
            FILE_SIZE=$(stat -c%s "${KAFKA_DIR}.tgz" 2>/dev/null || stat -f%z "${KAFKA_DIR}.tgz" 2>/dev/null || echo "0")
            
            if [ "$FILE_SIZE" -gt 10000000 ]; then
                echo "✓ Download successful (${FILE_SIZE} bytes)"
                DOWNLOAD_SUCCESS=true
                break
            else
                echo "✗ Download failed - file too small (${FILE_SIZE} bytes)"
                rm -f "${KAFKA_DIR}.tgz"
            fi
        else
            echo "✗ Download failed from this mirror"
            rm -f "${KAFKA_DIR}.tgz"
        fi
    done
    
    if [ "$DOWNLOAD_SUCCESS" = false ]; then
        echo ""
        echo "✗ All download attempts failed"
        echo "→ Please download manually from one of:"
        for url in "${KAFKA_DOWNLOAD_URLS[@]}"; do
            echo "   $url"
        done
        exit 1
    fi
    
    echo "→ Extracting Kafka..."
    if ! tar -xzf "${KAFKA_DIR}.tgz"; then
        echo "✗ Failed to extract Kafka archive"
        echo "→ The downloaded file may be corrupted. Removing it..."
        rm -f "${KAFKA_DIR}.tgz"
        exit 1
    fi
    
    echo "→ Cleaning up download..."
    rm "${KAFKA_DIR}.tgz"
    
    echo "✓ Kafka downloaded and extracted successfully!"
fi

echo ""
echo "==================================="
echo "Setup Complete!"
echo "==================================="
echo ""
echo "Next steps:"
echo "1. Start ZooKeeper: ./start-zookeeper.sh"
echo "2. Start Kafka: ./start-kafka.sh"
echo "3. Create a topic: ./create-topic.sh <topic-name>"
echo "4. Send messages: python3 producer.py"
echo "5. Read messages: python3 consumer.py"
echo ""
