"""Kafka POC - Producer and Consumer Package"""

__version__ = "0.1.0"
__author__ = "Your Name"
__email__ = "your.email@example.com"

from .producer import create_producer, send_message
from .consumer import create_consumer, consume_messages

__all__ = [
    "create_producer",
    "send_message",
    "create_consumer",
    "consume_messages",
]
