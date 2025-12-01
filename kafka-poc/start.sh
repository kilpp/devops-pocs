#!/bin/bash

# Quick start script for Kafka POC
# This script provides a menu-driven interface to run common tasks

set -e

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$PROJECT_ROOT"

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BLUE}=================================${NC}"
echo -e "${BLUE}Kafka POC Quick Start${NC}"
echo -e "${BLUE}=================================${NC}"
echo ""

# Check if venv exists
if [ ! -d "venv" ]; then
    echo -e "${YELLOW}→ Virtual environment not found. Creating...${NC}"
    python3 -m venv venv
    echo -e "${GREEN}✓ Virtual environment created${NC}"
fi

# Activate venv
source venv/bin/activate
echo -e "${GREEN}✓ Virtual environment activated${NC}"

# Check if dependencies are installed
if ! python3 -c "import kafka" 2>/dev/null; then
    echo -e "${YELLOW}→ Installing dependencies...${NC}"
    pip install -q --upgrade pip
    pip install -q -r requirements.txt
    echo -e "${GREEN}✓ Dependencies installed${NC}"
fi

echo ""
echo "Select an option:"
echo ""
echo "  1) Setup Kafka (download and extract)"
echo "  2) Start ZooKeeper"
echo "  3) Start Kafka Broker"
echo "  4) Create a topic"
echo "  5) Run Producer"
echo "  6) Run Consumer"
echo "  7) Run Tests"
echo "  8) Load Development Environment"
echo "  9) Show Project Info"
echo "  0) Exit"
echo ""

read -p "Enter your choice [0-9]: " choice

case $choice in
    1)
        echo -e "${BLUE}→ Setting up Kafka...${NC}"
        cd scripts
        ./setup-kafka.sh
        ;;
    2)
        echo -e "${BLUE}→ Starting ZooKeeper...${NC}"
        echo -e "${YELLOW}Press Ctrl+C to stop${NC}"
        cd scripts
        ./start-zookeeper.sh
        ;;
    3)
        echo -e "${BLUE}→ Starting Kafka Broker...${NC}"
        echo -e "${YELLOW}Press Ctrl+C to stop${NC}"
        cd scripts
        ./start-kafka.sh
        ;;
    4)
        read -p "Enter topic name [test-topic]: " topic_name
        topic_name=${topic_name:-test-topic}
        echo -e "${BLUE}→ Creating topic: ${topic_name}${NC}"
        cd scripts
        ./create-topic.sh "$topic_name"
        ;;
    5)
        echo -e "${BLUE}→ Running Producer...${NC}"
        source config/dev.env
        python3 src/producer.py
        ;;
    6)
        echo -e "${BLUE}→ Running Consumer...${NC}"
        source config/dev.env
        python3 src/consumer.py
        ;;
    7)
        echo -e "${BLUE}→ Running Tests...${NC}"
        if ! command -v pytest &> /dev/null; then
            echo -e "${YELLOW}→ Installing pytest...${NC}"
            pip install -q pytest pytest-cov
        fi
        pytest -v
        ;;
    8)
        echo -e "${BLUE}→ Loading development environment...${NC}"
        source config/dev.env
        echo -e "${GREEN}✓ Environment loaded${NC}"
        echo ""
        echo "Environment variables set:"
        echo "  KAFKA_BOOTSTRAP_SERVERS=$KAFKA_BOOTSTRAP_SERVERS"
        echo "  KAFKA_TOPIC=$KAFKA_TOPIC"
        echo "  KAFKA_CONSUMER_GROUP=$KAFKA_CONSUMER_GROUP"
        echo ""
        echo -e "${YELLOW}Virtual environment is still active.${NC}"
        echo "Run 'deactivate' to exit the venv."
        ;;
    9)
        echo ""
        echo -e "${BLUE}Project Information:${NC}"
        echo ""
        echo "Python: $(python3 --version)"
        echo "Pip: $(pip --version | cut -d' ' -f1-2)"
        echo "Virtual Environment: $(which python3)"
        echo ""
        echo "Installed packages:"
        pip list | grep -i kafka
        echo ""
        echo "Project structure:"
        echo "  src/          - Python source code"
        echo "  scripts/      - Kafka setup scripts"
        echo "  tests/        - Unit tests"
        echo "  config/       - Environment configs"
        echo "  venv/         - Virtual environment"
        echo ""
        ;;
    0)
        echo -e "${GREEN}Goodbye!${NC}"
        exit 0
        ;;
    *)
        echo -e "${RED}✗ Invalid choice${NC}"
        exit 1
        ;;
esac
