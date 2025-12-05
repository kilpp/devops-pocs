#!/usr/bin/env python3

"""
Kafka Producer Script
This script sends messages to a Kafka topic.
Install kafka-python first: pip install kafka-python
"""

from kafka import KafkaProducer
from kafka.errors import KafkaError
import json
import time
from datetime import datetime

# Configuration
BOOTSTRAP_SERVERS = ['localhost:9092']
TOPIC_NAME = 'test-topic'

def create_producer():
    """Create and return a Kafka producer instance."""
    try:
        producer = KafkaProducer(
            bootstrap_servers=BOOTSTRAP_SERVERS,
            value_serializer=lambda v: json.dumps(v).encode('utf-8'),
            key_serializer=lambda k: k.encode('utf-8') if k else None,
            acks='all',  # Wait for all replicas to acknowledge
            retries=3,
            max_in_flight_requests_per_connection=1
        )
        print("✓ Connected to Kafka broker")
        return producer
    except Exception as e:
        print(f"✗ Failed to connect to Kafka: {e}")
        print("\nMake sure:")
        print("1. ZooKeeper is running (./start-zookeeper.sh)")
        print("2. Kafka broker is running (./start-kafka.sh)")
        print("3. Topic exists (./create-topic.sh test-topic)")
        exit(1)

def send_message(producer, key, message):
    """Send a message to the Kafka topic."""
    try:
        future = producer.send(
            TOPIC_NAME,
            key=key,
            value=message
        )
        # Block until a single message is sent (or timeout)
        record_metadata = future.get(timeout=10)
        print(f"✓ Message sent successfully!")
        print(f"  Topic: {record_metadata.topic}")
        print(f"  Partition: {record_metadata.partition}")
        print(f"  Offset: {record_metadata.offset}")
        return True
    except KafkaError as e:
        print(f"✗ Failed to send message: {e}")
        return False

def interactive_mode(producer):
    """Interactive mode to send messages."""
    print("\n" + "="*50)
    print("Interactive Message Producer")
    print("="*50)
    print("Type your messages (or 'quit' to exit)")
    print("Format: key:message (key is optional)")
    print("Example: user123:Hello World")
    print("="*50 + "\n")
    
    message_count = 0
    
    while True:
        try:
            user_input = input("Enter message: ").strip()
            
            if user_input.lower() in ['quit', 'exit', 'q']:
                print("\nExiting producer...")
                break
            
            if not user_input:
                continue
            
            # Parse key and message
            if ':' in user_input:
                key, message_text = user_input.split(':', 1)
                key = key.strip()
            else:
                key = None
                message_text = user_input
            
            # Create message object
            message = {
                'text': message_text,
                'timestamp': datetime.now().isoformat(),
                'message_id': message_count + 1
            }
            
            if send_message(producer, key, message):
                message_count += 1
                print(f"Total messages sent: {message_count}\n")
            
        except KeyboardInterrupt:
            print("\n\nExiting producer...")
            break
        except Exception as e:
            print(f"Error: {e}\n")

def demo_mode(producer):
    """Demo mode that sends sample messages."""
    print("\n" + "="*50)
    print("Demo Mode - Sending Sample Messages")
    print("="*50 + "\n")
    
    sample_messages = [
        ("user1", {"text": "Hello from Kafka!", "type": "greeting"}),
        ("user2", {"text": "This is a test message", "type": "info"}),
        ("user3", {"text": "Kafka is awesome!", "type": "opinion"}),
        ("user1", {"text": "Another message from user1", "type": "update"}),
        (None, {"text": "Message without a key", "type": "broadcast"}),
        (None, {})
    ]
    
    for i, (key, msg_data) in enumerate(sample_messages, 1):
        msg_data['timestamp'] = datetime.now().isoformat()
        msg_data['message_id'] = i
        
        print(f"\nSending message {i}/5...")
        print(f"Key: {key if key else 'None'}")
        print(f"Message: {msg_data}")
        
        send_message(producer, key, msg_data)
        time.sleep(1)
    
    print(f"\n✓ Demo complete! Sent {len(sample_messages)} messages")

def main():
    """Main function."""
    print("\n" + "="*50)
    print("Kafka Producer")
    print("="*50)
    
    producer = create_producer()
    
    print("\nSelect mode:")
    print("1. Interactive mode (type messages manually)")
    print("2. Demo mode (send sample messages)")
    
    try:
        choice = input("\nEnter choice (1 or 2): ").strip()
        
        if choice == '1':
            interactive_mode(producer)
        elif choice == '2':
            demo_mode(producer)
        else:
            print("Invalid choice. Running demo mode...")
            demo_mode(producer)
    
    except KeyboardInterrupt:
        print("\n\nInterrupted by user")
    
    finally:
        print("\nClosing producer connection...")
        producer.flush()
        producer.close()
        print("✓ Producer closed successfully")

if __name__ == "__main__":
    main()
