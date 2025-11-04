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

# Check if database exists
echo ""
if [ -f data/leetcode.duckdb ]; then
    echo "✓ Database found at data/leetcode.duckdb"
else
    echo "⚠️  Database not found. Make sure to either:"
    echo "   1. Run the migration (if you have old data)"
    echo "   2. Or ensure data/leetcode.duckdb exists"
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
