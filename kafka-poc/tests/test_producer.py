"""Unit tests for Kafka producer"""

import pytest
from unittest.mock import Mock, patch, MagicMock
from src.producer import create_producer, send_message


class TestProducer:
    """Test suite for Kafka producer functionality"""

    @patch("src.producer.KafkaProducer")
    def test_create_producer_success(self, mock_kafka_producer):
        """Test successful producer creation"""
        mock_producer_instance = Mock()
        mock_kafka_producer.return_value = mock_producer_instance

        producer = create_producer()

        assert producer is not None
        mock_kafka_producer.assert_called_once()

    @patch("src.producer.KafkaProducer")
    def test_create_producer_failure(self, mock_kafka_producer):
        """Test producer creation failure handling"""
        mock_kafka_producer.side_effect = Exception("Connection failed")

        with pytest.raises(SystemExit):
            create_producer()

    @patch("src.producer.KafkaProducer")
    def test_send_message_success(self, mock_kafka_producer):
        """Test successful message sending"""
        mock_producer = MagicMock()
        mock_future = Mock()
        mock_metadata = Mock()
        mock_metadata.topic = "test-topic"
        mock_metadata.partition = 0
        mock_metadata.offset = 123
        mock_future.get.return_value = mock_metadata
        mock_producer.send.return_value = mock_future

        message = {"text": "test message"}
        result = send_message(mock_producer, "test-key", message)

        assert result is True
        mock_producer.send.assert_called_once()

    @patch("src.producer.KafkaProducer")
    def test_send_message_failure(self, mock_kafka_producer):
        """Test message sending failure handling"""
        mock_producer = MagicMock()
        mock_future = Mock()
        mock_future.get.side_effect = Exception("Send failed")
        mock_producer.send.return_value = mock_future

        message = {"text": "test message"}
        result = send_message(mock_producer, "test-key", message)

        assert result is False
