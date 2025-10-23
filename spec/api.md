# API (HTTP and gRPC)

## Authentication

 - Per-account auth; requests MUST include valid credentials matching account metadata.
 - For Miden, the signature is over the `account_id` (RPO256 digest of the account ID), not the full request payload.

## HTTP Endpoints

- POST /configure
  - Body: ConfigureAccountParams
  - 200: { account_id }
- POST /delta
  - Body: PushDeltaParams
  - 200: { delta }
- GET /delta?account_id=...&nonce=...
  - 200: { delta }
- GET /delta/since?account_id=...&from_nonce=...
  - 200: { merged_delta }
- GET /state?account_id=...
  - 200: { state }

Errors: structured JSON with code and message; include at minimum: `AccountNotFound`, `AuthenticationFailed`, `InvalidDelta`, `ConflictPendingDelta`, `CommitmentMismatch`, `DeltaNotFound`, `StateNotFound`.

## gRPC

- Service: StateManager (see generated descriptors). Methods mirror HTTP with analogous request/response messages.

## Idempotency and Ordering

- `push_delta` MAY be retried by clients; server SHOULD treat identical deltas (same account_id, nonce, payload) as idempotent when possible.
- Server enforces `prev_commitment` match; nonce monotonicity is network-dependent.

## Component trait (reference)

```rust
trait API {
  // Configure a new account passing an initial state and authentication credentials.
  fn configure(&self, params: ConfigureAccountParams) -> Result<ConfigureAccountResult>;

  // Push a new delta to the account, the server responds with the acknowledgement.
  fn push_delta(&self, params: PushDeltaParams) -> Result<PushDeltaResult>;

  // Get a specific delta by nonce.
  fn get_delta(&self, params: GetDeltaParams) -> Result<GetDeltaResult>;

  // Get  merged delta since a given nonce
  fn get_delta_since(&self, params: GetDeltaSinceParams) -> Result<GetDeltaSinceResult>;

  // Get the current state of the account
  fn get_state(&self, params: GetStateParams) -> Result<GetStateResult>;
}
```
