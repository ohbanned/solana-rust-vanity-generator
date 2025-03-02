#!/bin/bash

# Print banner
echo "====================================================================="
echo "      SOLANA VANITY ADDRESS GENERATOR - QUICK START LAUNCHER"
echo "====================================================================="
echo ""
echo "This script will launch the server in the background."
echo "You can then generate addresses using simple commands."
echo ""

# Check if server is already running
curl -s http://127.0.0.1:3001/health > /dev/null
if [ $? -eq 0 ]; then
    echo "✅ Server is already running at http://127.0.0.1:3001"
else
    echo "🚀 Starting server in the background..."
    nohup ./run_server.sh > server.log 2>&1 &
    SERVER_PID=$!
    echo "   Server started with PID: $SERVER_PID"
    echo "   Server logs: server.log"
    
    # Wait for server to start
    echo "   Waiting for server to initialize..."
    for i in {1..10}; do
        sleep 1
        curl -s http://127.0.0.1:3001/health > /dev/null
        if [ $? -eq 0 ]; then
            echo "   ✅ Server ready!"
            break
        fi
        if [ $i -eq 10 ]; then
            echo "   ❌ Server failed to start in time. Check server.log"
            exit 1
        fi
    done
fi

echo ""
echo "====================================================================="
echo "                      HOW TO USE THE GENERATOR"
echo "====================================================================="
echo ""
echo "To generate a vanity address, run one of these commands:"
echo ""
echo "  ./run_cli.sh abc prefix  # Generate address starting with 'abc'"
echo "  ./run_cli.sh xyz suffix  # Generate address ending with 'xyz'"
echo ""
echo "Replace 'abc' or 'xyz' with your desired 3-8 character pattern."
echo ""
echo "====================================================================="
