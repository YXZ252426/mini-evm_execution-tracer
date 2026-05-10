# Mini EVM Execution Tracer

A minimal Ethereum Virtual Machine execution tracer built on top of [`revm`](https://github.com/bluealloy/revm).

This project executes synthetic Ethereum transactions locally and exports structured JSON traces for opcode-level execution, gas usage, calls, logs, memory previews, storage touches, and final state diffs.

It is intentionally small: the goal is to demonstrate execution-layer fundamentals without hiding the EVM behind a high-level contract testing framework.

## Why This Exists

Ethereum transaction execution is a state transition, not just a function call.

This tracer is designed around that idea:

- bytecode is installed into a local in-memory account
- a synthetic transaction is executed against that account
- an inspector observes each interpreter step
- the final state transition is summarized as a storage diff

The project is useful as a compact reference for how an execution client-style tracer is assembled from lower-level EVM components.

## Features

- Execute local contract bytecode with a synthetic transaction
- Build an in-memory `revm` database using `CacheDB`
- Configure caller balance, target account bytecode, calldata, value, and gas limit
- Record opcode-level traces:
  - program counter
  - opcode byte and opcode name
  - gas remaining
  - stack top values
  - memory size
  - bounded memory preview
- Record per-step storage touches for `SLOAD` and `SSTORE`
- Capture `LOG0` through `LOG4` event logs
- Capture call traces and nested call trees
- Export final storage state diffs
- Write deterministic JSON output for further analysis

## Architecture

```text
CLI Input
  |
  v
Executor
  |
  v
revm Context + CacheDB + TxEnv
  |
  v
EVM execution
  |
  v
MiniTracer inspector hooks
  |
  v
Trace JSON + terminal summary
```

Key modules:

- `src/cli.rs` defines the command-line interface.
- `src/executor.rs` constructs the local EVM environment and runs the transaction.
- `src/tracer.rs` implements the `revm` inspector hooks.
- `src/types.rs` defines the serialized trace output.
- `src/output.rs` writes summaries and JSON output.

## Quick Start

```bash
cargo run -- trace-local \
  --contract examples/sstore.hex \
  --from 0x1000000000000000000000000000000000000000 \
  --to 0x2000000000000000000000000000000000000000 \
  --json trace.json
```

Expected terminal summary:

```text
tx status: success
gas used: 43106
steps: 4
calls: 1
logs: 0
trace written to trace.json
```

## Comprehensive Example

The repository includes a more complete demo:

- `examples/comprehensive.hex`
- `examples/comprehensive.asm`

It exercises memory, storage, logs, calls, call tree construction, and final state diff generation in one transaction.

```bash
cargo run -- trace-local \
  --contract examples/comprehensive.hex \
  --from 0x1000000000000000000000000000000000000000 \
  --to 0x2000000000000000000000000000000000000000 \
  --json comprehensive-trace.json
```

Expected summary:

```text
tx status: success
gas used: 69465
steps: 25
calls: 2
logs: 1
trace written to comprehensive-trace.json
```

The comprehensive bytecode performs:

- `MSTORE` to make memory visible in step traces
- `SSTORE` to write storage slot `0x0`
- `SSTORE` to write storage slot `0x2`
- `SLOAD` to read storage slot `0x0`
- `LOG1` to emit an event
- `CALL` to precompile address `0x01`

## CLI

```text
cargo run -- trace-local \
  --contract <BYTECODE_HEX_FILE> \
  --from <CALLER_ADDRESS> \
  --to <CONTRACT_ADDRESS> \
  [--calldata <HEX_CALLDATA>] \
  [--value <ETH_VALUE_AS_U256>] \
  [--gas-limit <GAS_LIMIT>] \
  [--max-steps <N>] \
  [--json <OUTPUT_PATH>]
```

Arguments:

- `--contract`: path to a hex-encoded runtime bytecode file
- `--from`: synthetic transaction caller
- `--to`: account where the bytecode will be installed
- `--calldata`: optional calldata, defaults to `0x`
- `--value`: optional call value, defaults to `0`
- `--gas-limit`: optional transaction gas limit, defaults to `30000000`
- `--max-steps`: optional cap for opcode step recording
- `--json`: optional output file path

## JSON Output

The output contains:

```json
{
  "summary": {
    "success": true,
    "gas_used": 69465,
    "step_count": 25,
    "call_count": 2,
    "log_count": 1
  },
  "steps": [],
  "calls": [],
  "call_tree": [],
  "logs": [],
  "state_diff": []
}
```

An opcode step includes bounded execution context:

```json
{
  "depth": 0,
  "pc": 4,
  "opcode": 85,
  "opcode_hex": "0x55",
  "opcode_name": "SSTORE",
  "gas_remaining": 29978994,
  "stack_top": ["0x0", "0x1"],
  "memory_size": 32,
  "memory_preview_size": 32,
  "memory_preview": "0x000000000000000000000000000000000000000000000000000000000000002a",
  "memory_truncated": false,
  "storage": [
    {
      "op": "SSTORE",
      "address": "0x2000000000000000000000000000000000000000",
      "slot": "0x0",
      "value_before": "0x0",
      "value_after": "0x1",
      "write_value": "0x1"
    }
  ]
}
```

Final storage diffs summarize the transaction-level state transition:

```json
{
  "address": "0x2000000000000000000000000000000000000000",
  "storage": [
    {
      "slot": "0x0",
      "before": "0x0",
      "after": "0x1"
    }
  ]
}
```

## Design Notes

### Step Trace vs State Diff

Per-step traces are intentionally bounded. The tracer records stack top values, memory size, and a small memory preview, but it does not dump full memory on every opcode.

Storage is handled similarly. Each step only records storage information when the opcode touches storage, currently `SLOAD` and `SSTORE`. The final `state_diff` then summarizes all storage slots that changed during the transaction.

This avoids generating output proportional to:

```text
steps * accounts * storage_slots
```

while still preserving the information needed to understand execution.

### Inspector Hooks

The tracer uses `revm` inspector hooks:

- `step` records the opcode-level pre-execution snapshot
- `step_end` completes storage observations after `SLOAD` or `SSTORE`
- `log` captures emitted EVM logs
- `call` and `call_end` capture call frames and results

`step_end` matters for storage because the final value of a storage slot is only known after the opcode has executed.

## Current Limitations

This project does not claim full historical mainnet transaction replay.

Full historical replay requires archive state from an Ethereum execution client or archive node. This project instead focuses on local synthetic execution, where bytecode and transaction inputs are provided directly.

Other current limitations:

- no block-level replay
- no historical account/state loading
- no source map or Solidity-level decoding
- no ABI decoding for calldata or logs
- no persistent database backend

## Development

Run checks:

```bash
cargo check
cargo test
```

Run the examples:

```bash
cargo run -- trace-local \
  --contract examples/sstore.hex \
  --from 0x1000000000000000000000000000000000000000 \
  --to 0x2000000000000000000000000000000000000000 \
  --json trace.json

cargo run -- trace-local \
  --contract examples/comprehensive.hex \
  --from 0x1000000000000000000000000000000000000000 \
  --to 0x2000000000000000000000000000000000000000 \
  --json comprehensive-trace.json
```

## What This Demonstrates

This repository demonstrates practical familiarity with:

- EVM execution semantics
- opcode-level tracing
- stack, memory, storage, calls, logs, and gas
- local EVM state construction
- `revm` database/context/transaction setup
- inspector-based execution instrumentation
- JSON trace design with bounded output size

