"""Test fixtures and configuration for pytest"""

import pytest


@pytest.fixture
def mock_kafka_config():
    """Fixture providing mock Kafka configuration"""
    return {
        "bootstrap_servers": ["localhost:9092"],
        "topic": "test-topic",
        "consumer_group": "test-group",
    }


@pytest.fixture
def sample_message():
    """Fixture providing a sample message"""
    return {
        "text": "Test message",
        "timestamp": "2025-12-01T10:00:00",
        "message_id": 1,
    }
