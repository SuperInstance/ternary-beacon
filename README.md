# ternary-beacon

**Discovery and presence protocol for fleet agents**

[![ternary](https://img.shields.io/badge/ecosystem-ternary-blue)](https://github.com/orgs/SuperInstance/repositories?q=ternary)
[![tests](https://img.shields.io/badge/tests-27-green)]()

## Overview

Discovery and presence protocol for fleet agents.

Inspired by Oracle1's Beacon interconnection layer. Models how agents find
each other in the fleet: broadcasting presence, scanning for nearby agents,
maintaining a registry, ternary filtering of beacons, and signal strength
estimation.

## Architecture

- **`AgentId`** — core data structure
- **`BeaconMessage`** — core data structure
- **`Beacon`** — core data structure
- **`DetectedBeacon`** — core data structure
- **`BeaconScanner`** — core data structure
- **`BeaconFilter`** — core data structure
- **`RegistryEntry`** — core data structure
- **`BeaconRegistry`** — core data structure
- **`ProtocolMessage`** — core data structure
- **`Ternary`** — state enumeration
- **`SignalStrength`** — state enumeration
- **`FilterCriterion`** — state enumeration
- **`BeaconOp`** — state enumeration

### Key Functions

- `new()`
- `with_capability()`
- `with_metadata()`
- `has_capability()`
- `from_raw()`
- `is_usable()`
- `new()`
- `agent_id()`
- `is_active()`
- `broadcast_range()`
- ... and 31 more

## Why Ternary?

The balanced ternary system {-1, 0, +1} (also known as Z₃) is the mathematically optimal discrete encoding:
- **More expressive than binary**: three states capture positive, neutral, and negative
- **Natural for decisions**: accept/reject/abstain, buy/hold/sell, agree/disagree/neutral
- **Self-balancing**: the 0 state acts as a universal screen, preventing pathological lock-in
- **Z₃ cyclic dynamics**: rock-paper-scissors is the only natural coordination mechanism

## Stats

| Metric | Value |
|--------|-------|
| Lines of Rust | 772 |
| Test count | 27 |
| Public types | 13 |
| Public functions | 41 |

## Ecosystem

This crate is part of the **[SuperInstance Ternary Fleet](https://github.com/orgs/SuperInstance/repositories?q=ternary)**:

- **[ternary-core](https://github.com/SuperInstance/ternary-core)** — shared traits and Z₃ arithmetic
- **[ternary-grid](https://github.com/SuperInstance/ternary-grid)** — spatial grid with {-1, 0, +1} cells
- **[ternary-graph](https://github.com/SuperInstance/ternary-graph)** — ternary-weighted graph algorithms
- **[ternary-automata](https://github.com/SuperInstance/ternary-automata)** — three-state cellular automata
- **[ternary-compiler](https://github.com/SuperInstance/ternary-compiler)** — expression compiler and optimizer

200+ crates. 4,300+ tests. One pattern.

## Research Context

The ternary approach connects to several active research areas:
- **Ternary Neural Networks** (TNNs): weights constrained to {-1, 0, +1} for efficient inference
- **Huawei's ternary chip**: 7nm ternary silicon with 60% less power consumption
- **Active inference**: free energy minimization naturally maps to ternary action selection
- **Cyclic dominance**: RPS dynamics maintain biodiversity in spatial ecology
- **Z₃ group theory**: the only algebraic group on three elements is cyclic addition mod 3

## Usage

```toml
[dependencies]
ternary-beacon = "0.1.0"
```

```rust
use ternary_beacon;
```

## License

MIT
