"""Configuration settings for Kafka POC"""

import os
from typing import List

# Kafka Connection Settings
BOOTSTRAP_SERVERS: List[str] = os.getenv(
    "KAFKA_BOOTSTRAP_SERVERS", "localhost:9092"
).split(",")

# Topic Settings
DEFAULT_TOPIC: str = os.getenv("KAFKA_TOPIC", "test-topic")
PARTITIONS: int = int(os.getenv("KAFKA_PARTITIONS", "3"))
REPLICATION_FACTOR: int = int(os.getenv("KAFKA_REPLICATION_FACTOR", "1"))

# Consumer Settings
CONSUMER_GROUP_ID: str = os.getenv("KAFKA_CONSUMER_GROUP", "test-consumer-group")
AUTO_OFFSET_RESET: str = os.getenv("KAFKA_AUTO_OFFSET_RESET", "latest")

# Producer Settings
PRODUCER_ACKS: str = os.getenv("KAFKA_PRODUCER_ACKS", "all")
PRODUCER_RETRIES: int = int(os.getenv("KAFKA_PRODUCER_RETRIES", "3"))

# Logging
LOG_LEVEL: str = os.getenv("LOG_LEVEL", "INFO")
