#!/bin/bash

# LeetCode Tracker Setup Script
set -e

echo "🚀 LeetCode Tracker Production Setup"
echo "======================================"
echo ""

# Check if .env exists
if [ ! -f .env ]; then
    echo "📝 Creating .env file from template..."
    cp .env.example .env
    echo "⚠️  Please edit .env and add your GitHub OAuth credentials!"
    echo "   1. Go to: https://github.com/settings/developers"
    echo "   2. Create a new OAuth App"
    echo "   3. Set callback URL to: http://localhost:3000/api/auth/callback"
    echo "   4. Copy Client ID and Client Secret to .env"
    echo ""
    read -p "Press Enter after you've updated .env..."
else
    echo "✓ .env file already exists"
fi

# Check if data directory exists
if [ ! -d data ]; then
    mkdir -p data
    echo "✓ Created data directory"
fi

# Ask if user wants to migrate old data
if [ -f data/02_state/leetcode.duckdb ]; then
    echo ""
    echo "📦 Found existing Streamlit database"
    read -p "Do you want to migrate data? (y/n) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo "🔄 Running migration..."
        cd backend
        cargo run --bin migrate
        cd ..
        echo "✓ Migration completed"
    fi
fi

# Check if CSV files exist
echo ""
echo "📂 Checking for CSV question lists..."
if [ ! -d data/01_raw ]; then
    echo "⚠️  data/01_raw/ not found. Please add your CSV files there."
else
    csv_count=$(ls -1 data/01_raw/*.csv 2>/dev/null | wc -l)
    if [ $csv_count -eq 0 ]; then
        echo "⚠️  No CSV files found in data/01_raw/"
        echo "   Please add your question list CSV files."
    else
        echo "✓ Found $csv_count CSV files"
    fi
fi

# Load CSV data if Python environment exists
if command -v uv &> /dev/null && [ -f load_data.py ]; then
    echo ""
    read -p "Load CSV data into database? (y/n) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo "📊 Loading data..."
        uv run python load_data.py
        echo "✓ Data loaded"
    fi
fi

echo ""
echo "🎉 Setup complete!"
echo ""
echo "Next steps:"
echo "  1. docker compose up -d     # Start the application"
echo "  2. Open http://localhost:3000"
echo "  3. Sign in with GitHub"
echo ""
echo "To view logs:"
echo "  docker compose logs -f"
echo ""
echo "To stop:"
echo "  docker compose down"
echo ""
