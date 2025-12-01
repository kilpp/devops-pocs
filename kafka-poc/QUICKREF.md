# Kafka POC - Quick Reference

## 🚀 Getting Started (3 Steps)

```bash
# 1. Activate virtual environment
source activate.sh

# 2. Use the interactive menu
./start.sh

# 3. Follow the prompts!
```

## 📝 Common Commands

### Virtual Environment
```bash
source activate.sh        # Activate
deactivate               # Deactivate
```

### Run Application
```bash
./start.sh                        # Interactive menu (EASIEST)
python3 src/producer.py           # Run producer
python3 src/consumer.py           # Run consumer
```

### Kafka Services
```bash
cd scripts
./setup-kafka.sh                  # Download Kafka (first time)
./start-zookeeper.sh             # Start ZooKeeper (terminal 1)
./start-kafka.sh                 # Start Kafka (terminal 2)
./create-topic.sh my-topic       # Create topic
```

### Development
```bash
pytest                           # Run tests
pytest --cov=src                # With coverage
black src/ tests/               # Format code
```

### Configuration
```bash
source config/dev.env           # Load dev settings
source config/prod.env          # Load prod settings
```

## 🔧 Environment Variables

```bash
export KAFKA_BOOTSTRAP_SERVERS="localhost:9092"
export KAFKA_TOPIC="test-topic"
export KAFKA_CONSUMER_GROUP="test-consumer-group"
```

## 📂 Important Files

- `start.sh` - Interactive menu (USE THIS!)
- `activate.sh` - Activate virtual environment
- `src/producer.py` - Producer script
- `src/consumer.py` - Consumer script
- `README.md` - Full documentation
- `DEVELOPMENT.md` - Developer guide
- `PROJECT_SUMMARY.md` - Project overview

## ⚡ Quick Setup (First Time)

```bash
# Terminal 1 - Setup & ZooKeeper
cd kafka-poc
source activate.sh
./start.sh
# Select: 1 (Setup Kafka)
# Then: 2 (Start ZooKeeper)

# Terminal 2 - Kafka Broker
cd kafka-poc
source activate.sh
cd scripts && ./start-kafka.sh

# Terminal 3 - Create Topic & Producer
cd kafka-poc
source activate.sh
./scripts/create-topic.sh test-topic
python3 src/producer.py

# Terminal 4 - Consumer
cd kafka-poc
source activate.sh
python3 src/consumer.py
```

## 🐛 Troubleshooting

### Virtual environment not activated?
```bash
source activate.sh
```

### Module not found?
```bash
source activate.sh
pip install -r requirements.txt
```

### Can't connect to Kafka?
1. Check ZooKeeper is running (terminal 1)
2. Check Kafka is running (terminal 2)
3. Check topic exists: `./scripts/create-topic.sh test-topic`

### Port already in use?
```bash
# Kill ZooKeeper
lsof -ti:2181 | xargs kill -9

# Kill Kafka
lsof -ti:9092 | xargs kill -9
```

## 📖 Documentation

- **README.md** - User guide and setup instructions
- **DEVELOPMENT.md** - Development workflow and best practices
- **PROJECT_SUMMARY.md** - Project structure and features
- **QUICKREF.md** - This file!

## 💡 Tips

1. **Always activate venv first**: `source activate.sh`
2. **Use start.sh for everything**: `./start.sh`
3. **Keep 4 terminals open**: ZooKeeper, Kafka, Producer, Consumer
4. **Load config when needed**: `source config/dev.env`
5. **Test your changes**: `pytest`

## 🎯 Next Steps

1. ✅ Setup complete - virtual environment created
2. ⏳ Download Kafka - run `./start.sh` → option 1
3. ⏳ Start services - ZooKeeper → Kafka → Create topic
4. ⏳ Run producer and consumer
5. 🎉 Send and receive messages!

---

**Need help?** Check README.md or DEVELOPMENT.md for detailed information.
