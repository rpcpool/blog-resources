# Building a Wallet — Code Examples

Runnable code for the [Building a Wallet on Solana with Triton](https://blog.triton.one) guide.

## Structure

```
typescript/
  step1-fetch-state.ts      — Fetch SOL balance and SPL token accounts
  step2-stream-updates.ts   — Stream real-time account changes with Dragon's Mouth
  step3-tx-history.ts       — Fetch transaction history
  step4-send-sol.ts         — Send SOL with priority fee via Jet
  step5-send-tokens.ts      — Send SPL tokens with priority fee via Jet
  package.json

rust/
  src/
    step1_fetch_state.rs
    step2_stream_updates.rs
    step3_tx_history.rs
    step4_send_sol.rs
    step5_send_tokens.rs
  Cargo.toml

python/
  step1_fetch_state.py
  step2_stream_updates.py
  step3_tx_history.py
  step4_send_sol.py
  step5_send_tokens.py
  requirements.txt
```

## Setup

Replace `your-endpoint.rpcpool.com` and `your-token` in each file with your Triton endpoint and token. [Get one here](https://app.triton.one).

**TypeScript**
```bash
cd typescript
npm install
npm run step1 -- YourWalletAddress
```

**Rust**
```bash
cd rust
cargo run --bin step1_fetch_state -- YourWalletAddress
```

**Python**

Generate the yellowstone gRPC stubs first (required for step2 only):
```bash
pip install grpcio grpcio-tools
python -m grpc_tools.protoc -I./proto --python_out=. --grpc_python_out=. geyser.proto
# Proto files: https://github.com/rpcpool/yellowstone-grpc/tree/master/yellowstone-grpc-proto/proto
```

Then:
```bash
cd python
pip install -r requirements.txt
python step1_fetch_state.py YourWalletAddress
```
