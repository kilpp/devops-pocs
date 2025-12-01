# Kafka POC - Complete Setup Guide

A complete Kafka proof-of-concept with scripts to set up Kafka, produce messages, and consume messages.

## Prerequisites

- Linux or macOS
- Java 8+ installed (required for Kafka)
- Python 3.6+ installed
- curl and tar utilities

## Quick Start

Follow these steps in order:

### Step 0: Setup Python Virtual Environment

Create and activate a virtual environment:

```bash
# Create venv (if not already created)
python3 -m venv venv

# Activate it
source activate.sh
# or
source venv/bin/activate

# Install dependencies
pip install -r requirements.txt
```

### Step 1: Setup Kafka

Download and extract Kafka:

```bash
cd scripts
chmod +x *.sh
./setup-kafka.sh
```

This will download Apache Kafka and extract it to your current directory.

### Step 2: Start ZooKeeper

In a **new terminal window**, start ZooKeeper:

```bash
cd scripts
./start-zookeeper.sh
```

Keep this terminal running. You should see logs indicating ZooKeeper is running on port 2181.

### Step 3: Start Kafka Broker

In **another new terminal window**, start the Kafka broker:

```bash
cd scripts
./start-kafka.sh
```

Keep this terminal running as well. You should see logs indicating Kafka is running on port 9092.

### Step 4: Create a Topic

In your **original terminal**, create a test topic:

```bash
cd scripts
./create-topic.sh test-topic
```

This creates a topic named "test-topic" with 3 partitions.

### Step 5: Run the Consumer

In a **new terminal window**, activate venv and start the consumer:

```bash
source activate.sh
python3 src/consumer.py
```

You'll be prompted to choose:
1. Read from latest (new messages only)
2. Read from beginning (all messages)
3. Show topic info only

Choose option 1 for now. The consumer will wait for new messages.

### Step 7: Run the Producer

In **another terminal window**, start the producer to send messages:

```bash
python3 producer.py
```

You'll be prompted to choose:
1. Interactive mode (type messages manually)
2. Demo mode (send sample messages)

Choose option 2 to send sample messages, or option 1 to type your own messages.
### Step 6: Run the Producer
## Project Structure

```
kafka-poc/
├── src/                    # Python source code
│   ├── producer.py        # Kafka producer
│   ├── consumer.py        # Kafka consumer
│   └── config.py          # Configuration
├── scripts/               # Shell scripts
│   ├── setup-kafka.sh     # Downloads Kafka
│   ├── start-zookeeper.sh # Starts ZooKeeper
│   ├── start-kafka.sh     # Starts Kafka broker
│   └── create-topic.sh    # Creates topics
├── tests/                 # Unit tests
├── config/               # Environment configs
├── venv/                 # Virtual environment
├── activate.sh           # Venv activation helper
- **`consumer.py`** - Reads messages from a Kafka topic
  - Can read from latest or from beginning
  - Shows topic information (partitions, offsets)
  - Pretty-prints received messages
  - Graceful shutdown with Ctrl+C
- **`config.py`** - Centralized configuration with environment variable support
### Shell Scripts (in `scripts/`)

- **`setup-kafka.sh`** - Downloads and extracts Apache Kafka
- **`start-zookeeper.sh`** - Starts ZooKeeper (required for Kafka)
- **`start-kafka.sh`** - Starts the Kafka broker
- **`create-topic.sh`** - Creates a new Kafka topic (usage: `./create-topic.sh <topic-name>`)

### Python Scripts (in `src/`)

- **`producer.py`** - Sends messages to a Kafka topic
- **`producer.py`** - Sends messages to a Kafka topic
  - Interactive mode: Manually type messages
  - Demo mode: Automatically sends sample messages
  - Messages include timestamp and message ID
  - Supports message keys for partitioning

- **`consumer.py`** - Reads messages from a Kafka topic
  - Can read from latest or from beginning
  - Shows topic information (partitions, offsets)
  - Pretty-prints received messages
  - Graceful shutdown with Ctrl+C

## Architecture

```
┌─────────────┐
│  ZooKeeper  │ (port 2181)
│  Cluster    │ - Manages Kafka metadata
└──────┬──────┘ - Coordinates brokers
       │
┌──────▼──────┐
│    Kafka    │ (port 9092)
│   Broker    │ - Stores messages
└──────┬──────┘ - Manages topics
       │
## Configuration

### Default Settings

- **Kafka Bootstrap Server**: `localhost:9092`
- **ZooKeeper Port**: `2181`
- **Default Topic**: `test-topic`
- **Consumer Group**: `test-consumer-group`
- **Partitions**: 3
- **Replication Factor**: 1

### Customizing

Configuration is managed through environment variables. Load a configuration:

```bash
# Development environment
source config/dev.env

# Production environment
source config/prod.env
```

Or set individual variables:
```bash
export KAFKA_TOPIC="my-custom-topic"
export KAFKA_BOOTSTRAP_SERVERS="kafka1:9092,kafka2:9092"
```

Create a new topic:
```bash
cd scripts
./create-topic.sh my-custom-topic
```
## Configuration

### Default Settings

- **Kafka Bootstrap Server**: `localhost:9092`
- **ZooKeeper Port**: `2181`
- **Default Topic**: `test-topic`
- **Consumer Group**: `test-consumer-group`
- **Partitions**: 3
- **Replication Factor**: 1

### Customizing

To use a different topic name, edit the `TOPIC_NAME` variable in:
- `producer.py`
- `consumer.py`

Or create a new topic:
```bash
./create-topic.sh my-custom-topic
```

## Troubleshooting

### Port Already in Use

If you see "Address already in use" errors:

**ZooKeeper (port 2181):**
```bash
lsof -ti:2181 | xargs kill -9
```

**Kafka (port 9092):**
```bash
lsof -ti:9092 | xargs kill -9
```

### Consumer Not Receiving Messages
### List all topics:
```bash
cd scripts/kafka_2.13-3.6.1
bin/kafka-topics.sh --list --bootstrap-server localhost:9092
```

### Describe a topic:
```bash
cd scripts/kafka_2.13-3.6.1
bin/kafka-topics.sh --describe --topic test-topic --bootstrap-server localhost:9092
```

### Delete a topic:
```bash
cd scripts/kafka_2.13-3.6.1
bin/kafka-topics.sh --delete --topic test-topic --bootstrap-server localhost:9092
```

### View consumer groups:
```bash
cd scripts/kafka_2.13-3.6.1
bin/kafka-consumer-groups.sh --list --bootstrap-server localhost:9092
```

### Check consumer group lag:
```bash
cd scripts/kafka_2.13-3.6.1
bin/kafka-consumer-groups.sh --describe --group test-consumer-group --bootstrap-server localhost:9092
```bash
brew install openjdk@11
```

## Useful Commands

### List all topics:
```bash
cd kafka_2.13-3.6.1
bin/kafka-topics.sh --list --bootstrap-server localhost:9092
```

### Describe a topic:
```bash
cd kafka_2.13-3.6.1
bin/kafka-topics.sh --describe --topic test-topic --bootstrap-server localhost:9092
```

### Delete a topic:
```bash
cd kafka_2.13-3.6.1
bin/kafka-topics.sh --delete --topic test-topic --bootstrap-server localhost:9092
```

### View consumer groups:
```bash
cd kafka_2.13-3.6.1
bin/kafka-consumer-groups.sh --list --bootstrap-server localhost:9092
## Testing

The project includes a full test suite:

```bash
# Activate virtual environment
source activate.sh

# Run all tests
pytest

# Run with coverage
pytest --cov=src --cov-report=html

# View coverage report
open htmlcov/index.html
```

## Development

For detailed development information, see [DEVELOPMENT.md](DEVELOPMENT.md).

Quick commands:
```bash
# Format code
black src/ tests/

# Lint code
flake8 src/ tests/

# Type check
mypy src/
```

## Clean Up

To remove all data and start fresh:

```bash
rm -rf scripts/kafka_2.13-3.6.1
rm -rf /tmp/kafka-logs
rm -rf /tmp/zookeeper
```

Then run `./scripts/setup-kafka.sh` again.Ctrl+C`
2. Stop Kafka broker: Press `Ctrl+C` in the Kafka terminal
3. Stop ZooKeeper: Press `Ctrl+C` in the ZooKeeper terminal

## Clean Up

To remove all data and start fresh:

```bash
rm -rf kafka_2.13-3.6.1
rm -rf /tmp/kafka-logs
rm -rf /tmp/zookeeper
```

Then run `./setup-kafka.sh` again.

## Next Steps

- Experiment with multiple consumers in the same consumer group (load balancing)
- Try different consumer groups to see independent consumption
- Modify partition count and observe message distribution
- Add error handling and retry logic to your applications
- Explore Kafka Streams for real-time processing

## Resources

- [Apache Kafka Documentation](https://kafka.apache.org/documentation/)
- [kafka-python Documentation](https://kafka-python.readthedocs.io/)
- [Kafka Quickstart Guide](https://kafka.apache.org/quickstart)

## License

This POC is for educational and testing purposes.
