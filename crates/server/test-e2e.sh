#!/bin/bash

# End-to-End Test Script for Private State Manager
# Tests both gRPC and HTTP endpoints

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
GRPC_HOST="localhost:50051"
HTTP_HOST="localhost:3000"
ACCOUNT_ID="test_account_$(date +%s)"
TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
CLEANUP_PERFORMED=false

# Helper functions
print_header() {
    echo -e "\n${BLUE}========================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}========================================${NC}\n"
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
    exit 1
}

print_info() {
    echo -e "${YELLOW}→ $1${NC}"
}

# Check required tools
check_requirements() {
    print_header "Checking Requirements"

    if ! command -v grpcurl &> /dev/null; then
        print_error "grpcurl is not installed. Install with: brew install grpcurl"
    fi
    print_success "grpcurl is installed"

    if ! command -v curl &> /dev/null; then
        print_error "curl is not installed"
    fi
    print_success "curl is installed"

    if ! command -v jq &> /dev/null; then
        print_error "jq is not installed. Install with: brew install jq"
    fi
    print_success "jq is installed"
}

# Wait for services to be ready
wait_for_services() {
    print_header "Waiting for Services"

    print_info "Waiting for gRPC server on $GRPC_HOST..."
    for i in {1..30}; do
        if grpcurl -plaintext $GRPC_HOST list &> /dev/null; then
            print_success "gRPC server is ready"
            break
        fi
        if [ $i -eq 30 ]; then
            print_error "gRPC server failed to start"
        fi
        sleep 1
    done

    print_info "Waiting for HTTP server on $HTTP_HOST..."
    for i in {1..30}; do
        if curl -s http://$HTTP_HOST/ &> /dev/null; then
            print_success "HTTP server is ready"
            break
        fi
        if [ $i -eq 30 ]; then
            print_error "HTTP server failed to start"
        fi
        sleep 1
    done
}

# Test gRPC endpoints
test_grpc() {
    print_header "Testing gRPC Endpoints"

    # Test 1: Configure account
    print_info "Test 1: Configure account via gRPC"
    CONFIGURE_RESPONSE=$(grpcurl -plaintext -d "{
        \"account_id\": \"$ACCOUNT_ID\",
        \"initial_state\": \"{\\\"balance\\\": 1000}\",
        \"storage_type\": \"local\",
        \"cosigner_pubkeys\": []
    }" $GRPC_HOST state_manager.StateManager/Configure)

    SUCCESS=$(echo "$CONFIGURE_RESPONSE" | jq -r '.success')
    if [ "$SUCCESS" != "true" ]; then
        MESSAGE=$(echo "$CONFIGURE_RESPONSE" | jq -r '.message')
        print_error "Failed to configure account: $MESSAGE"
    fi
    print_success "Account configured: $ACCOUNT_ID"

    # Test 2: Get account state
    print_info "Test 2: Get account state via gRPC"
    STATE_RESPONSE=$(grpcurl -plaintext -d "{
        \"account_id\": \"$ACCOUNT_ID\"
    }" $GRPC_HOST state_manager.StateManager/GetState)

    SUCCESS=$(echo "$STATE_RESPONSE" | jq -r '.success')
    if [ "$SUCCESS" != "true" ]; then
        MESSAGE=$(echo "$STATE_RESPONSE" | jq -r '.message')
        print_error "Failed to get state: $MESSAGE"
    fi

    STATE_JSON=$(echo "$STATE_RESPONSE" | jq -r '.state.stateJson')
    if [ "$STATE_JSON" == "{\"balance\":1000}" ]; then
        print_success "State verified: $STATE_JSON"
    else
        print_error "State mismatch. Expected: {\"balance\":1000}, Got: $STATE_JSON"
    fi

    # Test 3: Push first delta
    print_info "Test 3: Push first delta via gRPC"
    DELTA1_RESPONSE=$(grpcurl -plaintext -d "{
        \"account_id\": \"$ACCOUNT_ID\",
        \"nonce\": 0,
        \"prev_commitment\": \"\",
        \"delta_hash\": \"hash_0\",
        \"delta_payload\": \"{\\\"operation\\\": \\\"transfer\\\", \\\"amount\\\": 100}\",
        \"ack_sig\": \"sig_0\",
        \"publisher_pubkey\": \"pubkey_1\",
        \"publisher_sig\": \"pub_sig_0\",
        \"candidate_at\": \"$TIMESTAMP\"
    }" $GRPC_HOST state_manager.StateManager/PushDelta)

    SUCCESS=$(echo "$DELTA1_RESPONSE" | jq -r '.success')
    if [ "$SUCCESS" != "true" ]; then
        MESSAGE=$(echo "$DELTA1_RESPONSE" | jq -r '.message')
        print_error "Failed to push first delta: $MESSAGE"
    fi
    print_success "First delta pushed (nonce: 0)"

    # Test 4: Push second delta (with a gap in nonce)
    print_info "Test 4: Push second delta with non-sequential nonce via gRPC"
    DELTA2_RESPONSE=$(grpcurl -plaintext -d "{
        \"account_id\": \"$ACCOUNT_ID\",
        \"nonce\": 5,
        \"prev_commitment\": \"hash_0\",
        \"delta_hash\": \"hash_5\",
        \"delta_payload\": \"{\\\"operation\\\": \\\"transfer\\\", \\\"amount\\\": 50}\",
        \"ack_sig\": \"sig_5\",
        \"publisher_pubkey\": \"pubkey_1\",
        \"publisher_sig\": \"pub_sig_5\",
        \"candidate_at\": \"$TIMESTAMP\"
    }" $GRPC_HOST state_manager.StateManager/PushDelta)

    SUCCESS=$(echo "$DELTA2_RESPONSE" | jq -r '.success')
    if [ "$SUCCESS" != "true" ]; then
        MESSAGE=$(echo "$DELTA2_RESPONSE" | jq -r '.message')
        print_error "Failed to push second delta: $MESSAGE"
    fi
    print_success "Second delta pushed (nonce: 5, non-sequential)"

    # Test 5: Get specific delta
    print_info "Test 5: Get specific delta (nonce: 0) via gRPC"
    GET_DELTA_RESPONSE=$(grpcurl -plaintext -d "{
        \"account_id\": \"$ACCOUNT_ID\",
        \"nonce\": 0
    }" $GRPC_HOST state_manager.StateManager/GetDelta)

    SUCCESS=$(echo "$GET_DELTA_RESPONSE" | jq -r '.success')
    if [ "$SUCCESS" != "true" ]; then
        MESSAGE=$(echo "$GET_DELTA_RESPONSE" | jq -r '.message')
        print_error "Failed to get delta: $MESSAGE"
    fi

    # Note: protobuf3 omits default values (0 for uint64) in JSON serialization
    # So nonce: 0 won't appear in the JSON. We verify via deltaHash instead.
    DELTA_HASH=$(echo "$GET_DELTA_RESPONSE" | jq -r '.delta.deltaHash')
    if [ "$DELTA_HASH" == "hash_0" ]; then
        print_success "Delta retrieved and verified (nonce: 0, hash: hash_0)"
    else
        print_error "Delta hash mismatch. Expected: hash_0, Got: $DELTA_HASH"
    fi

    # Test 6: Get latest delta head
    print_info "Test 6: Get latest delta head via gRPC"
    HEAD_RESPONSE=$(grpcurl -plaintext -d "{
        \"account_id\": \"$ACCOUNT_ID\"
    }" $GRPC_HOST state_manager.StateManager/GetDeltaHead)

    SUCCESS=$(echo "$HEAD_RESPONSE" | jq -r '.success')
    if [ "$SUCCESS" != "true" ]; then
        MESSAGE=$(echo "$HEAD_RESPONSE" | jq -r '.message')
        print_error "Failed to get delta head: $MESSAGE"
    fi

    LATEST_NONCE=$(echo "$HEAD_RESPONSE" | jq -r '.latestNonce')
    if [ "$LATEST_NONCE" == "5" ]; then
        print_success "Latest nonce verified: $LATEST_NONCE (correctly identifies max nonce)"
    else
        print_error "Latest nonce mismatch. Expected: 5, Got: $LATEST_NONCE"
    fi
}

# Test HTTP endpoints
test_http() {
    print_header "Testing HTTP Endpoints"

    ACCOUNT_ID_HTTP="test_http_$(date +%s)"

    # Test 1: Configure account via HTTP
    print_info "Test 1: Configure account via HTTP"
    HTTP_CONFIGURE_RESPONSE=$(curl -s -X POST http://$HTTP_HOST/configure \
        -H "Content-Type: application/json" \
        -d "{
            \"account_id\": \"$ACCOUNT_ID_HTTP\",
            \"initial_state\": {\"balance\": 2000},
            \"storage_type\": \"local\",
            \"cosigner_pubkeys\": []
        }")

    SUCCESS=$(echo "$HTTP_CONFIGURE_RESPONSE" | jq -r '.success')
    if [ "$SUCCESS" == "true" ]; then
        print_success "Account configured via HTTP: $ACCOUNT_ID_HTTP"
    else
        MESSAGE=$(echo "$HTTP_CONFIGURE_RESPONSE" | jq -r '.message')
        print_error "Failed to configure account via HTTP: $MESSAGE"
    fi

    # Test 2: Get account state via HTTP
    print_info "Test 2: Get account state via HTTP"
    HTTP_STATE_RESPONSE=$(curl -s "http://$HTTP_HOST/state?account_id=$ACCOUNT_ID_HTTP")

    # Parse and normalize JSON (remove whitespace)
    STATE_JSON=$(echo "$HTTP_STATE_RESPONSE" | jq -r '.state_json' | jq -c .)
    if [ "$STATE_JSON" == "{\"balance\":2000}" ]; then
        print_success "State verified via HTTP: $STATE_JSON"
    else
        print_error "State mismatch via HTTP. Expected: {\"balance\":2000}, Got: $STATE_JSON"
    fi

    # Test 3: Push delta via HTTP
    print_info "Test 3: Push delta via HTTP"
    HTTP_DELTA1_RESPONSE=$(curl -s -X POST http://$HTTP_HOST/delta \
        -H "Content-Type: application/json" \
        -d "{
            \"account_id\": \"$ACCOUNT_ID_HTTP\",
            \"nonce\": 10,
            \"prev_commitment\": \"\",
            \"delta_hash\": \"hash_http_10\",
            \"delta_payload\": {\"operation\": \"http_transfer\", \"amount\": 200},
            \"ack_sig\": \"sig_http_10\",
            \"publisher_pubkey\": \"pubkey_http\",
            \"publisher_sig\": \"pub_sig_http_10\",
            \"candidate_at\": \"$TIMESTAMP\",
            \"canonical_at\": null,
            \"discarded_at\": null
        }")

    DELTA_HASH=$(echo "$HTTP_DELTA1_RESPONSE" | jq -r '.delta_hash')
    if [ "$DELTA_HASH" == "hash_http_10" ]; then
        print_success "Delta pushed via HTTP (nonce: 10)"
    else
        print_error "Failed to push delta via HTTP. Response: $HTTP_DELTA1_RESPONSE"
    fi

    # Test 4: Get specific delta via HTTP
    print_info "Test 4: Get specific delta via HTTP"
    HTTP_GET_DELTA=$(curl -s "http://$HTTP_HOST/delta?account_id=$ACCOUNT_ID_HTTP&nonce=10")

    DELTA_HASH=$(echo "$HTTP_GET_DELTA" | jq -r '.delta_hash')
    if [ "$DELTA_HASH" == "hash_http_10" ]; then
        print_success "Delta retrieved via HTTP (nonce: 10)"
    else
        print_error "Failed to get delta via HTTP. Got: $HTTP_GET_DELTA"
    fi

    # Test 5: Get latest nonce via HTTP
    print_info "Test 5: Get latest nonce via HTTP"
    HTTP_HEAD=$(curl -s "http://$HTTP_HOST/head?account_id=$ACCOUNT_ID_HTTP")

    LATEST_NONCE=$(echo "$HTTP_HEAD" | jq -r '.latest_nonce')
    if [ "$LATEST_NONCE" == "10" ]; then
        print_success "Latest nonce via HTTP: $LATEST_NONCE"
    else
        print_error "Latest nonce mismatch via HTTP. Expected: 10, Got: $LATEST_NONCE"
    fi

    # Cleanup HTTP test account
    print_info "Cleaning up HTTP test account"
    APP_PATH="${PSM_APP_PATH:-/var/psm/app}"
    if [ -d "$APP_PATH/$ACCOUNT_ID_HTTP" ]; then
        rm -rf "$APP_PATH/$ACCOUNT_ID_HTTP"
    fi
    METADATA_FILE="$APP_PATH/.metadata/accounts.json"
    if [ -f "$METADATA_FILE" ] && command -v jq &> /dev/null; then
        TEMP_FILE=$(mktemp)
        jq "del(.accounts[\"$ACCOUNT_ID_HTTP\"])" "$METADATA_FILE" > "$TEMP_FILE"
        mv "$TEMP_FILE" "$METADATA_FILE"
    fi
    print_success "HTTP test account cleaned up"
}

# Test error cases
test_error_cases() {
    print_header "Testing Error Cases"

    # Test 1: Configure duplicate account
    print_info "Test 1: Try to configure duplicate account"
    DUPLICATE_RESPONSE=$(grpcurl -plaintext -d "{
        \"account_id\": \"$ACCOUNT_ID\",
        \"initial_state\": \"{}\",
        \"storage_type\": \"local\",
        \"cosigner_pubkeys\": []
    }" $GRPC_HOST state_manager.StateManager/Configure)

    SUCCESS=$(echo "$DUPLICATE_RESPONSE" | jq -r '.success // "false"')
    MESSAGE=$(echo "$DUPLICATE_RESPONSE" | jq -r '.message')
    if [ "$SUCCESS" == "false" ] && [[ "$MESSAGE" == *"already exists"* ]]; then
        print_success "Correctly rejected duplicate account"
    else
        print_error "Should have rejected duplicate account. Response: $DUPLICATE_RESPONSE"
    fi

    # Test 2: Get state for non-existent account
    print_info "Test 2: Try to get state for non-existent account"
    NONEXISTENT_RESPONSE=$(grpcurl -plaintext -d "{
        \"account_id\": \"nonexistent_account\"
    }" $GRPC_HOST state_manager.StateManager/GetState)

    SUCCESS=$(echo "$NONEXISTENT_RESPONSE" | jq -r '.success // "false"')
    MESSAGE=$(echo "$NONEXISTENT_RESPONSE" | jq -r '.message')
    if [ "$SUCCESS" == "false" ] && [[ "$MESSAGE" == *"not found"* ]]; then
        print_success "Correctly rejected non-existent account"
    else
        print_error "Should have rejected non-existent account. Response: $NONEXISTENT_RESPONSE"
    fi

    # Test 3: Get delta for non-existent nonce
    print_info "Test 3: Try to get delta for non-existent nonce"
    NONEXISTENT_DELTA=$(grpcurl -plaintext -d "{
        \"account_id\": \"$ACCOUNT_ID\",
        \"nonce\": 999
    }" $GRPC_HOST state_manager.StateManager/GetDelta)

    SUCCESS=$(echo "$NONEXISTENT_DELTA" | jq -r '.success // "false"')
    MESSAGE=$(echo "$NONEXISTENT_DELTA" | jq -r '.message')
    if [ "$SUCCESS" == "false" ] && [[ "$MESSAGE" == *"Failed to fetch delta"* ]]; then
        print_success "Correctly rejected non-existent delta"
    else
        print_error "Should have rejected non-existent delta. Response: $NONEXISTENT_DELTA"
    fi
}

# Cleanup test data
cleanup_test_data() {
    # Prevent multiple cleanups
    if [ "$CLEANUP_PERFORMED" = true ]; then
        return
    fi

    print_header "Cleaning Up Test Data"

    # Get the app path from environment or use default
    APP_PATH="${PSM_APP_PATH:-/var/psm/app}"

    print_info "Removing test account data from: $APP_PATH"

    # Remove account directory (contains state.json and deltas/)
    if [ -d "$APP_PATH/$ACCOUNT_ID" ]; then
        rm -rf "$APP_PATH/$ACCOUNT_ID"
        print_success "Removed account directory: $APP_PATH/$ACCOUNT_ID"
    else
        print_info "No account directory found (may have already been cleaned)"
    fi

    # Remove metadata entry
    METADATA_FILE="$APP_PATH/.metadata/accounts.json"
    if [ -f "$METADATA_FILE" ]; then
        # Use jq to remove the account from metadata
        if command -v jq &> /dev/null; then
            TEMP_FILE=$(mktemp)
            jq "del(.accounts[\"$ACCOUNT_ID\"])" "$METADATA_FILE" > "$TEMP_FILE"
            mv "$TEMP_FILE" "$METADATA_FILE"
            print_success "Removed account metadata for: $ACCOUNT_ID"
        else
            print_info "jq not available, skipping metadata cleanup"
        fi
    else
        print_info "No metadata file found"
    fi

    CLEANUP_PERFORMED=true
    print_success "Cleanup complete"
}

# Trap to ensure cleanup on exit
trap cleanup_test_data EXIT INT TERM

# Main test execution
main() {
    print_header "Private State Manager E2E Tests"
    echo "Account ID: $ACCOUNT_ID"
    echo "Timestamp: $TIMESTAMP"

    check_requirements
    wait_for_services
    test_grpc
    test_http
    test_error_cases

    print_header "All Tests Passed! 🎉"
    echo -e "${GREEN}All gRPC endpoints are working correctly${NC}"
    echo -e "${GREEN}All HTTP endpoints are working correctly${NC}"
    echo -e "${YELLOW}Cleanup will run automatically on exit${NC}"
}

# Run main
main
