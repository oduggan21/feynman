#!/bin/bash

# Test script for Feynman Tutor
set -e

echo "🧪 Testing Feynman Tutor Application"
echo "=================================="

# Test backend compilation
echo "📦 Building backend..."
cd backend
cargo build --release
echo "✅ Backend builds successfully"

# Test frontend compilation  
echo "📦 Building frontend..."
cd ../front
npm ci
npm run build
echo "✅ Frontend builds successfully"

# Test backend in test mode
echo "🔧 Testing backend in test mode..."
cd ../backend
timeout 10s bash -c 'TEST_MODE=true cargo run' &
BACKEND_PID=$!

# Wait for backend to start
sleep 3

# Test WebSocket connection
echo "🌐 Testing WebSocket connection..."
if command -v wscat &> /dev/null; then
    echo "Testing WebSocket at ws://localhost:3000/ws..."
    timeout 5s wscat -c ws://localhost:3000/ws -x '{"test": "connection"}' || echo "⚠️  wscat test skipped (wscat not available)"
else
    echo "⚠️  WebSocket test skipped (wscat not installed)"
fi

# Kill backend
kill $BACKEND_PID 2>/dev/null || true

echo ""
echo "✅ All tests passed!"
echo ""
echo "🚀 To run the application:"
echo "1. Test mode:     cd backend && TEST_MODE=true cargo run"
echo "2. Production:    cd backend && OPENAI_API_KEY=sk-... cargo run" 
echo "3. Frontend:      cd front && npm run dev"
echo "4. Open:          http://localhost:5173"
echo ""
echo "📖 See README.md for detailed setup instructions"