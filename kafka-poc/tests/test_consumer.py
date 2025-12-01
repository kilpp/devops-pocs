"""Unit tests for Kafka consumer"""

import pytest
from unittest.mock import Mock, patch, MagicMock
from src.consumer import create_consumer


class TestConsumer:
    """Test suite for Kafka consumer functionality"""

    @patch("src.consumer.KafkaConsumer")
    def test_create_consumer_success(self, mock_kafka_consumer):
        """Test successful consumer creation"""
        mock_consumer_instance = Mock()
        mock_kafka_consumer.return_value = mock_consumer_instance

        consumer = create_consumer()

        assert consumer is not None
        mock_kafka_consumer.assert_called_once()

    @patch("src.consumer.KafkaConsumer")
    def test_create_consumer_with_offset_reset(self, mock_kafka_consumer):
        """Test consumer creation with custom offset reset"""
        mock_consumer_instance = Mock()
        mock_kafka_consumer.return_value = mock_consumer_instance

        consumer = create_consumer(auto_offset_reset="earliest")

        assert consumer is not None
        call_kwargs = mock_kafka_consumer.call_args[1]
        assert call_kwargs["auto_offset_reset"] == "earliest"

    @patch("src.consumer.KafkaConsumer")
    def test_create_consumer_failure(self, mock_kafka_consumer):
        """Test consumer creation failure handling"""
        mock_kafka_consumer.side_effect = Exception("Connection failed")

        with pytest.raises(SystemExit):
            create_consumer()
