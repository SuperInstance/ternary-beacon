# ternary-beacon

**"I'm here." Discovery and presence broadcasting for fleet agents.**

In a fleet of distributed agents, the first problem is finding each other. An agent starts up, looks around, and... nothing. Who else is out there? What can they do? Where are they?

A beacon is the answer. Each agent broadcasts a beacon — a small, regular signal that says "I exist, here's what I can do, here's how to reach me." Other agents listen for beacons and build a map of the fleet. When an agent goes offline, its beacon stops. When a new agent appears, its beacon appears. The beacon stream IS the fleet's membership list.

## What's Inside

- **`Beacon`** — a presence announcement: agent ID, capabilities, endpoint, timestamp
- **`BeaconBroadcaster`** — broadcast beacons at regular intervals
- **`BeaconListener`** — listen for beacons from other agents
- **`FleetMap`** — the aggregate view of all heard beacons. Who's online, who's stale, who's new
- **`broadcast(beacon)`** — emit a beacon to the fleet
- **`listen(timeout)`** — listen for incoming beacons
- **`is_alive(agent_id, threshold)`** — has this agent's beacon been heard within the threshold?
- **`fleet_snapshot()`** — current fleet membership with last-seen timestamps

## Quick Example

```rust
use ternary_beacon::*;

// Create a beacon for this agent
let beacon = Beacon::new("oracle2")
    .capability("predict")
    .capability("simulate")
    .endpoint("construct-coordination");

// Broadcast it
broadcast(&beacon);

// Listen for other agents
let mut listener = BeaconListener::new();
let fleet = listener.fleet_snapshot();
for agent in &fleet.agents {
    println!("Agent: {}, capabilities: {:?}", agent.id, agent.capabilities);
}

// Check if a specific agent is alive
if is_alive(&fleet, "forgemaster", 60_000) {
    println!("Forgemaster is online!");
}
```

## The Deeper Truth

**Beacons are the fleet's heartbeat.** Each pulse says "I'm still here." The interval between pulses determines how quickly the fleet detects failures: a 60-second beacon means it takes up to 2 minutes to know someone's gone. A 1-second beacon means near-instant detection — but more network traffic. The beacon interval IS the fleet's reaction time.

The CORTEX.json spec (from construct-coordination) is the beacon's payload: not just "I'm here" but "here's everything about me." The beacon broadcasts the CORTEX manifest. The listener collects manifests. The FleetMap is the aggregate CORTEX of the entire fleet — a living document of who exists and what they can do.

**Use cases:**
- **Service discovery** — find available agents in a distributed fleet
- **Health monitoring** — detect agent failures via beacon absence
- **Dynamic routing** — route tasks to agents based on beacon-advertised capabilities
- **Fleet visualization** — the FleetMap IS the dashboard
- **Self-organizing systems** — agents that discover each other and self-configure

## See Also

- **ternary-lighthouse** — lighthouses observe; beacons announce
- **ternary-protocol** — wire protocol for beacon transmission
- **ternary-room** — rooms are discovered via beacons
- **ternary-anchor** — anchors maintain what beacons discover
- **ternary-mesh** — mesh networks built on beacon discovery
- **ternary-constellation** — constellations are groups discovered via beacons
- **ternary-observatory** — observatories monitor beacon streams

## Install

```bash
cargo add ternary-beacon
```

## License

MIT
