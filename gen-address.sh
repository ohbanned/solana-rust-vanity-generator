#!/bin/bash

# Simple script to generate Solana vanity addresses
# Usage: ./gen-address.sh <pattern> [prefix|suffix]

if [ "$#" -lt 1 ]; then
    echo -e "\033[1;31mError: Missing pattern argument\033[0m"
    echo -e "\033[1;33mUsage: ./gen-address.sh <pattern> [prefix|suffix]\033[0m"
    echo -e "Example: ./gen-address.sh abc prefix"
    echo -e "Example: ./gen-address.sh 123 suffix"
    exit 1
fi

PATTERN=$1
POSITION=${2:-prefix}  # Default to prefix if not specified

if [[ "$POSITION" != "prefix" && "$POSITION" != "suffix" ]]; then
    echo -e "\033[1;31mError: Position must be either 'prefix' or 'suffix'\033[0m"
    exit 1
fi

echo -e "\033[1;34m🔍 Generating Solana address with $POSITION '$PATTERN'...\033[0m"

# Start job
RESPONSE=$(curl -s -X POST http://127.0.0.1:3001/generate \
    -H "Content-Type: application/json" \
    -d "{\"pattern\":\"$PATTERN\",\"position\":\"$POSITION\"}")

JOB_ID=$(echo $RESPONSE | grep -o '"job_id":"[^"]*' | sed 's/"job_id":"//')

if [ -z "$JOB_ID" ]; then
    echo -e "\033[1;31m❌ Error: Failed to start job. Server response: $RESPONSE\033[0m"
    exit 1
fi

echo -e "\033[1;32m✅ Job started with ID: $JOB_ID\033[0m"
echo -e "\033[1;33m⏳ Checking for results...\033[0m"

# Poll for results with a spinning animation
SYMBOLS=("⠋" "⠙" "⠹" "⠸" "⠼" "⠴" "⠦" "⠧" "⠇" "⠏")
COUNTER=0
START_TIME=$(date +%s)

while true; do
    SYMBOL=${SYMBOLS[$COUNTER]}
    COUNTER=$(( (COUNTER + 1) % 10 ))
    
    # Get current time elapsed
    CURRENT_TIME=$(date +%s)
    ELAPSED=$((CURRENT_TIME - START_TIME))
    
    # Check status
    STATUS_RESPONSE=$(curl -s http://127.0.0.1:3001/status/$JOB_ID)
    
    # Display spinner and time
    echo -ne "\r\033[1;34m$SYMBOL\033[0m Searching... [${ELAPSED}s elapsed]"
    
    # Check if complete
    if echo "$STATUS_RESPONSE" | grep -q '"status":"complete"'; then
        PUB_KEY=$(echo $STATUS_RESPONSE | grep -o '"public_key":"[^"]*' | sed 's/"public_key":"//')
        PRIV_KEY=$(echo $STATUS_RESPONSE | grep -o '"private_key":"[^"]*' | sed 's/"private_key":"//')
        
        echo -e "\r\033[1;32m✅ Address found in ${ELAPSED} seconds!\033[0m                   "
        echo -e "\033[1;36m📝 Results:\033[0m"
        echo -e "  Public key:  \033[0;32m$PUB_KEY\033[0m"
        echo -e "  Private key: \033[0;33m$PRIV_KEY\033[0m"
        echo -e "\n\033[1;31m⚠️  IMPORTANT: Save your private key securely!\033[0m"
        break
    fi
    
    # Check if error
    if echo "$STATUS_RESPONSE" | grep -q '"error"'; then
        ERROR=$(echo $STATUS_RESPONSE | grep -o '"error":"[^"]*' | sed 's/"error":"//')
        echo -e "\r\033[1;31m❌ Error: $ERROR\033[0m                                  "
        break
    fi
    
    sleep 0.1
done
