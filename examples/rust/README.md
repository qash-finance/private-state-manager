# PSM Rust Example

This example demonstrates a multi-client end-to-end flow using the Private State Manager (PSM) with Miden multisig accounts.

## Available Binaries

### `main` (Default - Local Node)
Uses a local Miden node for transaction execution.

**Prerequisites:**
- Miden node running on `http://localhost:57291`
- PSM server running on `http://localhost:50051`

**Run:**
```bash
cargo run --bin main
```

### `main_mockchain` (Testing)
Uses MockChain for fast testing without requiring a local Miden node.

**Prerequisites:**
- PSM server running on `http://localhost:50051`

**Run:**
```bash
cargo run --bin main_mockchain
```

## Flow Overview

1. **Setup**: Generate Falcon keypairs for two clients
2. **Step 1**: Connect to PSM server and get server's public key
3. **Step 2**: Create multisig PSM account (2-of-2 with PSM auth)
4. **Step 3**: Client 1 configures the account in PSM
5. **Step 4**: Client 2 retrieves the account state from PSM
6. **Step 5**: Client 2 simulates a transaction to update to 3-of-3 multisig
7. **Step 6**: Push transaction summary to PSM for server signature
8. **Step 7**: Execute transaction with all signatures (PSM + 2 clients)

## Starting Services

### PSM Server
```bash
cargo run --package private-state-manager-server --bin server
```

### Miden Node (for `main` only)
Follow the [Miden node setup instructions](https://docs.polygon.technology/miden/) to run a local node on port 57291.
