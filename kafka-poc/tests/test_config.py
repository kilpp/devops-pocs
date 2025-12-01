"""Unit tests for configuration"""

import os
import pytest
from src import config


class TestConfig:
    """Test suite for configuration settings"""

    def test_default_bootstrap_servers(self):
        """Test default bootstrap servers"""
        assert "localhost:9092" in config.BOOTSTRAP_SERVERS

    def test_default_topic(self):
        """Test default topic name"""
        assert config.DEFAULT_TOPIC == "test-topic"

    def test_default_consumer_group(self):
        """Test default consumer group"""
        assert config.CONSUMER_GROUP_ID == "test-consumer-group"

    def test_environment_variable_override(self, monkeypatch):
        """Test configuration override via environment variables"""
        monkeypatch.setenv("KAFKA_TOPIC", "custom-topic")
        # Need to reload the module to pick up new env vars
        import importlib
        importlib.reload(config)
        assert config.DEFAULT_TOPIC == "custom-topic"
