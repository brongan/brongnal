#!/bin/bash

set -e

echo "Cleaning up potentially lingering sessions"
tmux kill-session -t server 2>/dev/null || true
tmux kill-session -t client1 2>/dev/null || true
tmux kill-session -t client2 2>/dev/null || true

# Function to start a tmux session
start_tmux_session() {
    local name=$1
    local command=$2
    local log_file="${name}_output.log"
    
    # Clear existing log file
    > "$log_file"
    
    # Start the tmux session with logging
    tmux new-session -d -s $name
    tmux pipe-pane -t $name "cat >> $log_file"
    tmux send-keys -t $name "$command" C-m
    
    echo "Started $name in tmux session (Logging to $log_file)"
}

# Start the server
echo "Starting server..."
start_tmux_session "server" "cargo r -p server"

# Wait for the server to start
echo "Waiting for server to start..."
timeout=30
while ! tmux capture-pane -t server -p | grep -q "Brongnal Server listening at: 0.0.0.0:8080"; do
    sleep 1
    timeout=$((timeout - 1))
    if [ $timeout -le 0 ]; then
        echo "Error: Server failed to start within 30 seconds"
        tmux capture-pane -t server -p
        exit 1
    fi
done
echo "Server started successfully"

# Function to send a message
send_message() {
    local sender=$1
    local receiver=$2
    local message=$3
    echo "Sending message from $sender to $receiver: $message"
    tmux send-keys -t $sender "$receiver $message" C-m
    sleep 2
}

# Function to check log file for expected messages
check_log_file() {
    local client=$1
    local log_file="${client}_output.log"
    local expected_messages=(
        "Hello from client 1!"
        "Hi there, client 1!"
        "How are you doing?"
        "I'm doing great, thanks!"
        "That's wonderful to hear!"
    )
    local missing_messages=()

    echo "Checking log file for $client..."
    for message in "${expected_messages[@]}"; do
        if ! grep -q "$message" "$log_file"; then
            missing_messages+=("$message")
        fi
    done

    if [ ${#missing_messages[@]} -eq 0 ]; then
        echo "All expected messages found in $log_file"
    else
        echo "Warning: Some messages are missing from $log_file:"
        for message in "${missing_messages[@]}"; do
            echo "  - $message"
        done
        echo "Full contents of $log_file:"
        cat "$log_file"
    fi
}

# Test Block 1: Both clients start before messages are sent
echo "=== Test Block 1: Both clients start before messages are sent ==="
start_tmux_session "client1" "cargo r -p client client1 http://localhost:8080"
start_tmux_session "client2" "cargo r -p client client2 http://localhost:8080"

# Wait for clients to start and register
sleep 5

# Send messages
send_message "client1" "client2" "Hello from client 1!"
send_message "client2" "client1" "Hi there, client 1!"
send_message "client1" "client2" "How are you doing?"
send_message "client2" "client1" "I'm doing great, thanks!"
send_message "client1" "client2" "That's wonderful to hear!"

# Wait for messages to be processed
sleep 5

# Check log files
check_log_file "client1"
check_log_file "client2"

# Clean up clients
tmux kill-session -t client1
tmux kill-session -t client2

# Test Block 2: Clients are registered and messages are sent separately
echo "=== Test Block 2: Clients are registered and messages are sent separately ==="

# Start and run client1
start_tmux_session "client1" "cargo r -p client client1 http://localhost:8080"
sleep 3
send_message "client1" "client2" "Hello from client 1!"
send_message "client1" "client2" "How are you doing?"
sleep 2
tmux kill-session -t client1

# Start and run client2
start_tmux_session "client2" "cargo r -p client client2 http://localhost:8080"
sleep 3
send_message "client2" "client1" "Hi there, client 1!"
send_message "client2" "client1" "I'm doing great, thanks!"
sleep 2
tmux kill-session -t client2

# Start client1 again to receive final message
start_tmux_session "client1" "cargo r -p client client1 http://localhost:8080"
sleep 3
send_message "client1" "client2" "That's wonderful to hear!"
sleep 2
tmux kill-session -t client1

# Check log files
check_log_file "client1"
check_log_file "client2"

# Clean up server
echo "Cleaning up..."
tmux kill-session -t server

echo "All tests completed. Check the output above to see if all messages were received correctly in both test blocks."