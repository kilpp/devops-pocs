#!/usr/bin/env python3

"""
Kafka Consumer Script
This script reads messages from a Kafka topic.
Install kafka-python first: pip install kafka-python
"""

from kafka import KafkaConsumer
from kafka.errors import KafkaError
import json
import signal
import sys

# Configuration
BOOTSTRAP_SERVERS = ['localhost:9092']
TOPIC_NAME = 'test-topic'
GROUP_ID = 'test-consumer-group'

# Global flag for graceful shutdown
running = True

def signal_handler(sig, frame):
    """Handle Ctrl+C for graceful shutdown."""
    global running
    print("\n\nShutting down consumer...")
    running = False

def create_consumer(auto_offset_reset='latest'):
    """
    Create and return a Kafka consumer instance.
    
    Args:
        auto_offset_reset: 'earliest' to read from beginning, 'latest' to read new messages only
    """
    try:
        consumer = KafkaConsumer(
            TOPIC_NAME,
            bootstrap_servers=BOOTSTRAP_SERVERS,
            group_id=GROUP_ID,
            auto_offset_reset=auto_offset_reset,  # 'earliest' or 'latest'
            enable_auto_commit=True,
            auto_commit_interval_ms=1000,
            value_deserializer=lambda m: json.loads(m.decode('utf-8')),
            key_deserializer=lambda k: k.decode('utf-8') if k else None,
            consumer_timeout_ms=1000  # Return from poll after 1 second if no messages
        )
        print(f"✓ Connected to Kafka broker")
        print(f"✓ Subscribed to topic: {TOPIC_NAME}")
        print(f"✓ Consumer group: {GROUP_ID}")
        print(f"✓ Reading from: {auto_offset_reset}")
        return consumer
    except Exception as e:
        print(f"✗ Failed to connect to Kafka: {e}")
        print("\nMake sure:")
        print("1. ZooKeeper is running (./start-zookeeper.sh)")
        print("2. Kafka broker is running (./start-kafka.sh)")
        print("3. Topic exists (./create-topic.sh test-topic)")
        exit(1)

def consume_messages(consumer):
    """Consume and display messages from Kafka."""
    global running
    
    print("\n" + "="*50)
    print("Listening for messages...")
    print("Press Ctrl+C to stop")
    print("="*50 + "\n")
    
    message_count = 0
    
    try:
        while running:
            # Poll for messages
            message_batch = consumer.poll(timeout_ms=1000)
            
            for topic_partition, messages in message_batch.items():
                for message in messages:
                    message_count += 1
                    
                    print(f"\n{'='*50}")
                    print(f"Message #{message_count}")
                    print(f"{'='*50}")
                    print(f"Topic: {message.topic}")
                    print(f"Partition: {message.partition}")
                    print(f"Offset: {message.offset}")
                    print(f"Key: {message.key}")
                    print(f"Timestamp: {message.timestamp}")
                    print(f"\nValue:")
                    print(json.dumps(message.value, indent=2))
                    print(f"{'='*50}")
            
            # Small delay to prevent tight loop when no messages
            if not message_batch:
                sys.stdout.write('.')
                sys.stdout.flush()
    
    except KeyboardInterrupt:
        pass
    
    finally:
        print(f"\n\n✓ Total messages consumed: {message_count}")
        consumer.close()
        print("✓ Consumer closed successfully")

def show_topic_info(consumer):
    """Display information about the topic."""
    print("\n" + "="*50)
    print("Topic Information")
    print("="*50)
    
    try:
        partitions = consumer.partitions_for_topic(TOPIC_NAME)
        if partitions:
            print(f"Topic: {TOPIC_NAME}")
            print(f"Number of partitions: {len(partitions)}")
            print(f"Partitions: {sorted(partitions)}")
            
            # Get partition assignments
            assignments = consumer.assignment()
            if assignments:
                print(f"\nAssigned partitions: {[tp.partition for tp in assignments]}")
                
                # Get current positions
                print("\nCurrent offsets:")
                for tp in assignments:
                    position = consumer.position(tp)
                    print(f"  Partition {tp.partition}: {position}")
        else:
            print(f"Topic '{TOPIC_NAME}' not found or has no partitions")
    
    except Exception as e:
        print(f"Error getting topic info: {e}")
    
    print("="*50 + "\n")

def main():
    """Main function."""
    # Register signal handler for graceful shutdown
    signal.signal(signal.SIGINT, signal_handler)
    
    print("\n" + "="*50)
    print("Kafka Consumer")
    print("="*50)
    
    print("\nSelect mode:")
    print("1. Read from latest (new messages only)")
    print("2. Read from beginning (all messages)")
    print("3. Show topic info only")
    
    try:
        choice = input("\nEnter choice (1, 2, or 3): ").strip()
        
        if choice == '3':
            consumer = create_consumer('latest')
            show_topic_info(consumer)
            consumer.close()
            return
        
        if choice == '2':
            print("\n→ Reading from beginning of topic...")
            consumer = create_consumer('earliest')
        else:
            print("\n→ Reading new messages only...")
            consumer = create_consumer('latest')
        
        show_topic_info(consumer)
        consume_messages(consumer)
    
    except KeyboardInterrupt:
        print("\n\nInterrupted by user")
    except Exception as e:
        print(f"\nError: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main()
