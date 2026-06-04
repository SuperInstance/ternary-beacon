# Oracle1 Origin: Beacon → ternary-beacon

## Oracle1 Concept
**Layer 5: Beacon** — Discovery and registry service (part of the Keeper on port 8901). Agents broadcast their presence, capabilities, and status. The beacon layer enables fleet agents to find each other without central coordination.

From Oracle1's 6-layer interconnection model:
> Beacon — Discovery/registry — Status: Live

Oracle1's Agent API (port 8901) serves agent-to-agent lookup, while the beacon layer handles the broader discovery protocol.

### Oracle1's Beachcomb
Closely related: the **Beachcomb** polling protocol. Oracle1 runs periodic sweeps (every 15min–2hr) that scan peer repos for:
- New bottles (messages)
- New commits
- New issues
- Protocol changes
- Pull requests

Beachcomb is the discovery mechanism — agents find out about each other by polling, not by push notification.

## What We Borrowed
The **beacon/broadcast/scanning pattern** for agent discovery:
- Agents broadcast their presence periodically
- Scanners detect nearby beacons
- A registry maintains fleet membership with expiry
- Capability declarations enable matching

Specific concepts adapted:
- **Beacon broadcast** → Oracle1's agent presence announcements
- **BeaconScanner** → Oracle1's Beachcomb sweeps (polling for new content)
- **BeaconRegistry** → Oracle1's Keeper fleet registry (port 8900)
- **SignalStrength** → Oracle1's concept of agent proximity/reliability
- **Capability matching** → Oracle1's CAPABILITY.toml skill declarations

## How Our Implementation Differs

| Aspect | Oracle1's Beacon/Beachcomb | Our ternary-beacon |
|---|---|---|
| **Discovery** | Git poll (beachcomb) | `BeaconScanner::scan()` on message arrays |
| **Signal** | Binary (found/not-found) | `SignalStrength` classification (4 levels) |
| **Filtering** | None — all changes processed | `BeaconFilter` with ternary logic (Positive/Neutral/Negative) |
| **Registry** | Keeper service (PostgreSQL) | In-memory `BeaconRegistry` with expiry |
| **Capabilities** | CAPABILITY.toml files | Inline `Vec<String>` in beacon messages |
| **Protocol** | Git commits + HTTP API | Rust types with `ProtocolMessage` |
| **Ternary** | Not ternary-aware | `Ternary` enum used for filter results |

### Key Innovation: Ternary Filtering
Our `BeaconFilter` applies criteria that return ternary results:
- **Positive** — Beacon matches (include)
- **Neutral** — No opinion (include by default)
- **Negative** — Beacon rejected (exclude)

Any single Negative criterion rejects the entire beacon. This is more nuanced than Oracle1's binary found/not-found. We can express "I don't care about this capability" (Neutral) vs "I actively don't want agents with this capability" (Negative).

### Key Innovation: Signal Strength Classification
Oracle1's beachcomb has no concept of signal quality — it either finds changes or it doesn. Our `SignalStrength` classifies detections into Weak/Medium/Strong/Excellent, enabling priority-based processing of discovered agents.

### Key Innovation: Distance Estimation
Our `DetectedBeacon` includes a `distance_estimate` derived from signal strength. Oracle1 doesn't model inter-agent distance; it's a flat registry. We add spatial awareness to discovery.

## See Also
- Oracle1 Architecture Review: `construct-coordination/notes/main/ORACLE1-ARCHITECTURE-REVIEW.md`
- Oracle1-Ternary Bridge: `construct-coordination/notes/main/ORACLE1-TERNARY-BRIDGE.md`
- Beachcomb protocol: Oracle1's vessel `.i2i/peers.md` and beachcomb scripts
