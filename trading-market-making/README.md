# Trading and Market Making — Code Examples

Runnable code for the [Trading and Market Making with Triton](https://blog.triton.one) guide.

## Structure

```
typescript/
  step1-dragonmouth.ts    — Stream real-time DEX data with Dragon's Mouth
  step2-deshred.ts        — Get earliest signal with Deshred
  step3-priority-fee.ts   — Estimate a competitive priority fee
  step4-send-jet.ts       — Send transactions via Jet
  combined.ts             — Full trading loop combining all four steps
  package.json

rust/
  src/
    step1_dragonmouth.rs
    step2_deshred.rs
    step3_priority_fee.rs
    step4_send_jet.rs
  Cargo.toml

python/
  step1_dragonmouth.py
  step2_deshred.py
  step3_priority_fee.py
  step4_send_jet.py
  requirements.txt
```

## Setup

Replace `your-endpoint.rpcpool.com` and `your-token` in each file with your Triton endpoint and token. [Get one here](https://app.triton.one).

**TypeScript**
```bash
cd typescript
npm install
npm run step1
```

**Rust**
```bash
cd rust
cargo run --bin step1_dragonmouth
```

**Python**

Generate the yellowstone gRPC stubs first:
```bash
pip install grpcio grpcio-tools
python -m grpc_tools.protoc -I./proto --python_out=. --grpc_python_out=. geyser.proto
# Proto files: https://github.com/rpcpool/yellowstone-grpc/tree/master/yellowstone-grpc-proto/proto
```

Then:
```bash
cd python
pip install -r requirements.txt
python step1_dragonmouth.py
```
