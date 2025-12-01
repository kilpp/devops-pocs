#!/bin/bash

# Activation script for Kafka POC virtual environment
# Usage: source activate.sh

if [ -d "venv" ]; then
    source venv/bin/activate
    echo "✓ Virtual environment activated (kafka-poc)"
    echo "→ Python: $(which python3)"
    echo "→ Pip: $(which pip)"
    echo ""
    echo "To deactivate, run: deactivate"
else
    echo "✗ Virtual environment not found"
    echo "→ Run: python3 -m venv venv"
    exit 1
fi
