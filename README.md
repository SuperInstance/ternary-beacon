# Ternary Beacon

**Ternary Beacon** is a discovery and presence protocol for fleet agents — providing beacon broadcasting, scanning, registry management, ternary filtering of discovery signals, and signal strength estimation.

## Why It Matters

Distributed systems need service discovery: new agents must find existing ones, and the fleet must detect when agents join or leave. Ternary Beacon provides this with ternary filtering — agents can positively include (+1), neutrally observe (0), or negatively exclude (-1) discovered peers based on capabilities, signal strength, or custom metadata. This is mDNS/service discovery enhanced with the ternary decision framework.

## How It Works

### Beacon Broadcasting

Each agent broadcasts a `BeaconMessage`:

```rust
BeaconMessage {
    source: AgentId(u64),
    signal_strength: u32,        // 0-100
    capabilities: Vec<String>,   // ["gpio", "spi", "i2c"]
    metadata: HashMap<String, String>,
    timestamp: u64,
}
```

Broadcast cost: **O(N)** where N = payload size. Timestamp enables staleness detection.

### Signal Strength Classification

```
None (0):     0-25     — unreachable
Weak (1):     26-50    — marginal, high latency
Medium (2):   51-75    — usable
Strong (3):   76-100   — reliable
Excellent (4): 100     — co-located / same host
```

Classification: **O(1)** (integer comparison). Enables proximity-based agent clustering.

### Ternary Filtering

Each beacon is classified by a ternary filter:

```
+1 (Positive): Include this agent — match capabilities, strong signal
 0 (Neutral):  No opinion — include by default, monitor
-1 (Negative): Exclude this agent — incompatible, weak signal, banned
```

Filter evaluation: **O(C)** where C = number of capability checks.

### Registry Management

The `Registry` maintains discovered agents:

```
register(beacon)   → O(1) HashMap insert
scan() → Vec<&Beacon> → O(N) iterate all
scan_with_filter(filter) → Vec<&Beacon> → O(N · C)
expire(timeout_secs) → removes stale entries → O(N)
```

Registry uses HashMap keyed by AgentId for O(1) lookup.

### Heartbeat Integration

Beacons integrate with heartbeat intervals:

```
every heartbeat_period:
    broadcast(beacon_message)
    scan_for_new_beacons()
    expire_stale_entries()
```

Cycle cost: **O(B + N)** where B = broadcast targets, N = registered agents.

## Quick Start

```rust
use ternary_beacon::{AgentId, BeaconMessage, Registry};

let mut registry = Registry::new();
let beacon = BeaconMessage::new(AgentId(1), 85, 1000)
    .with_capability("gpio")
    .with_capability("i2c")
    .with_metadata("location", "engine_room");

registry.register(beacon);
let found = registry.scan_with_capability("gpio");
println!("GPIO-capable agents: {}", found.len());
```

## API

| Type | Description |
|------|-------------|
| `BeaconMessage` | Discovery broadcast with source, signal, capabilities, metadata |
| `AgentId(u64)` | Unique agent identifier |
| `SignalStrength` | None/Weak/Medium/Strong/Excellent (0-4) |
| `Ternary` | Positive/Neutral/Negative filter result |
| `Registry` | Agent registry with scan, filter, expire |

Key methods: `register()`, `scan()`, `scan_with_capability()`, `scan_with_filter()`, `expire()`.

## Architecture Notes

Ternary Beacon provides the discovery layer for fleet formation in SuperInstance. In γ + η = C, Positive (+1) filtering enables γ (growth — incorporating compatible agents) while Negative (-1) filtering implements η (avoidance — excluding incompatible or hostile agents). The Neutral (0) state allows passive observation before commitment. Integrates with `ternary-anchor` for persistent agent positioning and `node-agent` for heartbeat integration.

See [ARCHITECTURE.md](https://github.com/SuperInstance/SuperInstance/blob/main/ARCHITECTURE.md) for fleet discovery architecture.

## References

1. Cheshire, S. & Krochmal, M. (2013). RFC 6762 — "Multicast DNS." IETF.
2. Cheshire, S. & Krochmal, M. (2013). RFC 6763 — "DNS-Based Service Discovery." IETF.
3. Deering, S. (1989). RFC 1112 — "Host Extensions for IP Multicasting." IETF.

## License

MIT
