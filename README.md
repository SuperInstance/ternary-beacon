# ternary-beacon — Discovery and presence protocol for fleet agents

Agents in a fleet need to find each other. This crate provides beacon broadcasting, scanning, registry management, ternary filtering, and a standard protocol message format. Inspired by Oracle1's Beacon interconnection layer.

## Why This Exists

In a dynamic fleet, agents come and go. Before agents can communicate (channels), dock (harbors), or build structures (reefs), they need to discover each other. Beacon provides the "who's out there?" layer: agents broadcast their presence, scan for neighbors, maintain a registry of known fleet members, and filter discoveries using ternary logic (include, exclude, no opinion).

## Core Concepts

- **Beacon** — Broadcasts an agent's presence. Carries capabilities, metadata, signal strength, and a timestamp. Can be activated/deactivated (go silent).
- **BeaconScanner** — Detects nearby beacons within a scan range. Ignores self-detections. Estimates distance from signal strength. Returns results sorted by signal strength (strongest first).
- **BeaconFilter** — Ternary filtering of beacon messages. Criteria include capability presence/absence, metadata matching, and minimum signal strength. Any Negative criterion rejects the message; Positive criteria boost; Neutral is no opinion.
- **BeaconRegistry** — Maintains a registry of known fleet members with expiry. Agents that haven't been seen within the expiry window are pruned. Capabilities are merged across sightings.
- **SignalStrength** — Classifies raw signal values (0-100) into categories: None, Weak, Medium, Strong, Excellent.
- **BeaconProtocol** — Standard message format with operation types (Announce, Depart, Query, Response) and a protocol version string.

## Quick Start

```toml
[dependencies]
ternary-beacon = "0.1"
```

```rust
use ternary_beacon::*;

// Agent 1 broadcasts presence
let mut beacon = Beacon::new(AgentId(1), 75, 1000);
beacon.add_capability("compute");
beacon.set_metadata("region", "us-west");
let msg = beacon.broadcast(1001);

// Agent 2 scans for neighbors
let mut scanner = BeaconScanner::new(AgentId(2), 80);
let detections = scanner.scan(&[msg.clone()]);
assert_eq!(detections.len(), 1);

// Filter by capability
let mut filter = BeaconFilter::new();
filter.add_criterion(FilterCriterion::HasCapability("compute".to_string()));
let filtered = filter.filter(&[msg.clone()]);
assert_eq!(filtered.len(), 1);

// Maintain a registry
let mut registry = BeaconRegistry::new(500); // 500ms expiry
registry.register(&msg);
assert!(registry.is_active(AgentId(1), 1100));
```

## API Overview

| Type | Description |
|------|-------------|
| `Beacon` | Broadcasts agent presence with capabilities and metadata |
| `BeaconScanner` | Detects nearby beacons, estimates distance |
| `BeaconFilter` | Ternary filtering with composable criteria |
| `FilterCriterion` | Individual filter rules (capability, signal, metadata) |
| `BeaconRegistry` | Known fleet members with expiry and capability merging |
| `SignalStrength` | Signal classification (None/Weak/Medium/Strong/Excellent) |
| `BeaconProtocol` | Standard message format with ops and version |
| `BeaconMessage` | Core data: source, signal, capabilities, metadata, timestamp |

## How It Works

**Broadcasting:** Each `Beacon` holds a `BeaconMessage` that gets updated on each `broadcast()` call (timestamp refreshed, broadcast count incremented). The message is returned by reference so callers can forward it.

**Scanning:** `BeaconScanner::scan()` takes a slice of beacon messages, filters out self-detections and out-of-range messages, classifies signal strength, estimates distance (inverse of signal), and sorts by signal strength descending.

**Filtering:** `BeaconFilter` composes multiple `FilterCriterion` instances. Each criterion returns a Ternary result. The combined result is Negative if *any* criterion is Negative, Positive if any is Positive (without any Negative), and Neutral otherwise. This is a conservative AND-of-criteria approach.

**Registry:** `BeaconRegistry` uses a HashMap keyed by AgentId. Each `register()` call merges capabilities (additive, no removal) and updates the timestamp. `prune()` removes entries whose `last_seen` is beyond the expiry window.

**Protocol:** `ProtocolMessage` wraps a `BeaconMessage` with an operation type (Announce, Depart, Query, Response) and protocol version. This provides a standard envelope for fleet-wide communication.

## Known Limitations

- No actual network transport. This is a pure data model. You need to wrap it in your own networking layer.
- Distance estimation is naive: `100 / signal_strength`. Real-world distance estimation needs calibration and environmental factors.
- Registry capability merging is additive only — capabilities are never removed even if an agent stops advertising them.
- No encryption or authentication in the protocol layer. Security must be handled externally.
- Scanner sorts all results on every scan call — O(n log n). Fine for hundreds of agents, not for millions.

## Use Cases

- **Agent discovery** — New agents broadcast beacons to announce themselves; existing agents scan to find neighbors.
- **Capability matching** — Filter beacons by capability to find agents that can perform specific tasks.
- **Fleet roster** — Registry maintains the current fleet membership, automatically expiring stale entries.
- **Departure notification** — Agents send Depart protocol messages when leaving, allowing clean deregistration.
- **Proximity estimation** — Signal strength classification enables proximity-based grouping without GPS.

## Ecosystem Context

Part of the SuperInstance ternary fleet ecosystem. This is the discovery layer — agents use beacons before they use `ternary-channel` (messaging), `ternary-harbor` (docking), or `ternary-reef` (collective structures). Beacon messages are typically carried over `ternary-channel` connections.

## License

MIT
