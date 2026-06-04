#![forbid(unsafe_code)]

//! Discovery and presence protocol for fleet agents.
//!
//! Inspired by Oracle1's Beacon interconnection layer. Models how agents find
//! each other in the fleet: broadcasting presence, scanning for nearby agents,
//! maintaining a registry, ternary filtering of beacons, and signal strength
//! estimation.

use std::collections::HashMap;

// ---- Core types ----

/// Ternary filter result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ternary {
    /// Positive match — include this beacon.
    Positive,
    /// Neutral — no opinion, include by default.
    Neutral,
    /// Negative match — exclude this beacon.
    Negative,
}

/// Unique agent identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AgentId(pub u64);

/// A standard beacon message format.
#[derive(Debug, Clone)]
pub struct BeaconMessage {
    pub source: AgentId,
    pub signal_strength: u32,
    pub capabilities: Vec<String>,
    pub metadata: HashMap<String, String>,
    pub timestamp: u64,
}

impl BeaconMessage {
    pub fn new(source: AgentId, signal_strength: u32, timestamp: u64) -> Self {
        Self {
            source,
            signal_strength,
            capabilities: Vec::new(),
            metadata: HashMap::new(),
            timestamp,
        }
    }

    pub fn with_capability(mut self, cap: &str) -> Self {
        self.capabilities.push(cap.to_string());
        self
    }

    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }

    pub fn has_capability(&self, cap: &str) -> bool {
        self.capabilities.iter().any(|c| c == cap)
    }
}

// ---- SignalStrength ----

/// Signal strength categories based on distance/quality.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SignalStrength {
    /// No signal or too far.
    None = 0,
    /// Very weak signal (0-25).
    Weak = 1,
    /// Medium signal (26-50).
    Medium = 2,
    /// Strong signal (51-75).
    Strong = 3,
    /// Excellent signal (76-100).
    Excellent = 4,
}

impl SignalStrength {
    /// Classify a raw signal value (0-100) into a category.
    pub fn from_raw(raw: u32) -> Self {
        match raw {
            0..=25 => SignalStrength::Weak,
            26..=50 => SignalStrength::Medium,
            51..=75 => SignalStrength::Strong,
            76..=100 => SignalStrength::Excellent,
            _ => SignalStrength::None,
        }
    }

    /// Whether the signal is usable for communication.
    pub fn is_usable(&self) -> bool {
        !matches!(self, SignalStrength::None)
    }
}

// ---- Beacon ----

/// A beacon broadcasts an agent's presence to the fleet.
#[derive(Debug, Clone)]
pub struct Beacon {
    agent_id: AgentId,
    active: bool,
    broadcast_range: u32,
    message: BeaconMessage,
    broadcast_count: u64,
}

impl Beacon {
    pub fn new(agent_id: AgentId, broadcast_range: u32, timestamp: u64) -> Self {
        Self {
            agent_id,
            active: true,
            broadcast_range,
            message: BeaconMessage::new(agent_id, broadcast_range, timestamp),
            broadcast_count: 0,
        }
    }

    pub fn agent_id(&self) -> AgentId {
        self.agent_id
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn broadcast_range(&self) -> u32 {
        self.broadcast_range
    }

    pub fn broadcast_count(&self) -> u64 {
        self.broadcast_count
    }

    /// Broadcast presence. Returns the beacon message.
    pub fn broadcast(&mut self, timestamp: u64) -> &BeaconMessage {
        self.message.timestamp = timestamp;
        self.message.signal_strength = self.broadcast_range;
        self.broadcast_count += 1;
        &self.message
    }

    /// Activate the beacon.
    pub fn activate(&mut self) {
        self.active = true;
    }

    /// Deactivate the beacon (go silent).
    pub fn deactivate(&mut self) {
        self.active = false;
    }

    /// Add a capability to broadcast.
    pub fn add_capability(&mut self, cap: &str) {
        if !self.message.capabilities.contains(&cap.to_string()) {
            self.message.capabilities.push(cap.to_string());
        }
    }

    /// Remove a capability.
    pub fn remove_capability(&mut self, cap: &str) {
        self.message.capabilities.retain(|c| c != cap);
    }

    /// Set metadata.
    pub fn set_metadata(&mut self, key: &str, value: &str) {
        self.message.metadata.insert(key.to_string(), value.to_string());
    }
}

// ---- BeaconScanner ----

/// A detected beacon result.
#[derive(Debug, Clone)]
pub struct DetectedBeacon {
    pub message: BeaconMessage,
    pub signal_class: SignalStrength,
    pub distance_estimate: u32,
}

/// Scans for nearby beacons within range.
#[derive(Debug, Clone)]
pub struct BeaconScanner {
    agent_id: AgentId,
    scan_range: u32,
    detections: Vec<DetectedBeacon>,
}

impl BeaconScanner {
    pub fn new(agent_id: AgentId, scan_range: u32) -> Self {
        Self {
            agent_id,
            scan_range,
            detections: Vec::new(),
        }
    }

    pub fn agent_id(&self) -> AgentId {
        self.agent_id
    }

    pub fn scan_range(&self) -> u32 {
        self.scan_range
    }

    /// Scan a set of beacon messages, keeping those within range.
    pub fn scan(&mut self, messages: &[BeaconMessage]) -> &[DetectedBeacon] {
        self.detections.clear();
        for msg in messages {
            if msg.source == self.agent_id {
                continue; // don't detect self
            }
            if msg.signal_strength <= self.scan_range {
                let signal_class = SignalStrength::from_raw(msg.signal_strength);
                // Simple distance estimate: inverse of signal strength
                let distance = if msg.signal_strength > 0 {
                    100 / msg.signal_strength
                } else {
                    100
                };
                self.detections.push(DetectedBeacon {
                    message: msg.clone(),
                    signal_class,
                    distance_estimate: distance,
                });
            }
        }
        // Sort by signal strength descending
        self.detections.sort_by(|a, b| b.message.signal_strength.cmp(&a.message.signal_strength));
        &self.detections
    }

    pub fn detections(&self) -> &[DetectedBeacon] {
        &self.detections
    }

    /// Count detections with at least the given signal strength.
    pub fn count_above(&self, min_signal: SignalStrength) -> usize {
        self.detections
            .iter()
            .filter(|d| d.signal_class >= min_signal)
            .count()
    }
}

// ---- BeaconFilter ----

/// Filter criteria for ternary matching.
#[derive(Debug, Clone)]
pub enum FilterCriterion {
    /// Include only beacons with this capability.
    HasCapability(String),
    /// Exclude beacons with this capability.
    NotCapability(String),
    /// Include beacons with this metadata key-value.
    HasMetadata(String, String),
    /// Include beacons above this signal threshold.
    MinSignal(u32),
}

impl FilterCriterion {
    /// Apply this criterion to a beacon message, returning a ternary result.
    pub fn apply(&self, msg: &BeaconMessage) -> Ternary {
        match self {
            FilterCriterion::HasCapability(cap) => {
                if msg.has_capability(cap) {
                    Ternary::Positive
                } else {
                    Ternary::Negative
                }
            }
            FilterCriterion::NotCapability(cap) => {
                if msg.has_capability(cap) {
                    Ternary::Negative
                } else {
                    Ternary::Positive
                }
            }
            FilterCriterion::HasMetadata(key, value) => {
                match msg.metadata.get(key) {
                    Some(v) if v == value => Ternary::Positive,
                    Some(_) => Ternary::Neutral,
                    None => Ternary::Negative,
                }
            }
            FilterCriterion::MinSignal(min) => {
                if msg.signal_strength >= *min {
                    Ternary::Positive
                } else {
                    Ternary::Negative
                }
            }
        }
    }
}

/// A filter that combines multiple criteria with ternary logic.
/// A message passes if ALL criteria are non-negative (Positive or Neutral).
#[derive(Debug, Clone)]
pub struct BeaconFilter {
    criteria: Vec<FilterCriterion>,
}

impl BeaconFilter {
    pub fn new() -> Self {
        Self {
            criteria: Vec::new(),
        }
    }

    pub fn add_criterion(&mut self, criterion: FilterCriterion) {
        self.criteria.push(criterion);
    }

    /// Apply all criteria to a message. Returns the combined ternary result.
    /// Any Negative criterion makes the whole result Negative.
    pub fn apply(&self, msg: &BeaconMessage) -> Ternary {
        let mut result = Ternary::Neutral;
        for criterion in &self.criteria {
            match criterion.apply(msg) {
                Ternary::Negative => return Ternary::Negative,
                Ternary::Positive => result = Ternary::Positive,
                Ternary::Neutral => {}
            }
        }
        result
    }

    /// Filter a list of beacon messages, keeping only those with non-negative results.
    pub fn filter<'a>(&'a self, messages: &'a [BeaconMessage]) -> Vec<&BeaconMessage> {
        messages
            .iter()
            .filter(|m| !matches!(self.apply(m), Ternary::Negative))
            .collect()
    }

    pub fn criterion_count(&self) -> usize {
        self.criteria.len()
    }
}

impl Default for BeaconFilter {
    fn default() -> Self {
        Self::new()
    }
}

// ---- BeaconRegistry ----

/// Entry in the registry.
#[derive(Debug, Clone)]
pub struct RegistryEntry {
    pub agent_id: AgentId,
    pub last_seen: u64,
    pub capabilities: Vec<String>,
    pub signal_strength: u32,
}

/// Maintains a registry of known fleet members.
#[derive(Debug, Clone)]
pub struct BeaconRegistry {
    entries: HashMap<AgentId, RegistryEntry>,
    expiry_time: u64,
}

impl BeaconRegistry {
    pub fn new(expiry_time: u64) -> Self {
        Self {
            entries: HashMap::new(),
            expiry_time,
        }
    }

    /// Register or update a beacon sighting.
    pub fn register(&mut self, msg: &BeaconMessage) {
        let entry = self
            .entries
            .entry(msg.source)
            .or_insert_with(|| RegistryEntry {
                agent_id: msg.source,
                last_seen: 0,
                capabilities: Vec::new(),
                signal_strength: 0,
            });
        entry.last_seen = msg.timestamp;
        entry.signal_strength = msg.signal_strength;
        // Merge capabilities
        for cap in &msg.capabilities {
            if !entry.capabilities.contains(cap) {
                entry.capabilities.push(cap.clone());
            }
        }
    }

    /// Check if an agent is still considered active.
    pub fn is_active(&self, agent: AgentId, current_time: u64) -> bool {
        if let Some(entry) = self.entries.get(&agent) {
            current_time.saturating_sub(entry.last_seen) <= self.expiry_time
        } else {
            false
        }
    }

    /// Get a registry entry.
    pub fn get(&self, agent: AgentId) -> Option<&RegistryEntry> {
        self.entries.get(&agent)
    }

    /// Remove expired entries.
    pub fn prune(&mut self, current_time: u64) -> usize {
        let expired: Vec<AgentId> = self
            .entries
            .iter()
            .filter(|(_, e)| current_time.saturating_sub(e.last_seen) > self.expiry_time)
            .map(|(id, _)| *id)
            .collect();
        let count = expired.len();
        for id in expired {
            self.entries.remove(&id);
        }
        count
    }

    /// All active entries at the given time.
    pub fn active_entries(&self, current_time: u64) -> Vec<&RegistryEntry> {
        self.entries
            .values()
            .filter(|e| current_time.saturating_sub(e.last_seen) <= self.expiry_time)
            .collect()
    }

    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }
}

// ---- BeaconProtocol ----

/// Standard protocol version.
pub const PROTOCOL_VERSION: &str = "ternary-beacon/1.0";

/// Protocol operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BeaconOp {
    /// Announce presence.
    Announce,
    /// Departing the fleet.
    Depart,
    /// Query for agents with specific capabilities.
    Query,
    /// Response to a query.
    Response,
}

/// A standard beacon protocol message.
#[derive(Debug, Clone)]
pub struct ProtocolMessage {
    pub version: String,
    pub op: BeaconOp,
    pub source: AgentId,
    pub timestamp: u64,
    pub payload: BeaconMessage,
}

impl ProtocolMessage {
    pub fn new(op: BeaconOp, source: AgentId, timestamp: u64, payload: BeaconMessage) -> Self {
        Self {
            version: PROTOCOL_VERSION.to_string(),
            op,
            source,
            timestamp,
            payload,
        }
    }

    /// Create an announce message.
    pub fn announce(source: AgentId, timestamp: u64, signal: u32) -> Self {
        let payload = BeaconMessage::new(source, signal, timestamp);
        Self::new(BeaconOp::Announce, source, timestamp, payload)
    }

    /// Create a depart message.
    pub fn depart(source: AgentId, timestamp: u64) -> Self {
        let payload = BeaconMessage::new(source, 0, timestamp);
        Self::new(BeaconOp::Depart, source, timestamp, payload)
    }

    pub fn is_announce(&self) -> bool {
        self.op == BeaconOp::Announce
    }

    pub fn is_depart(&self) -> bool {
        self.op == BeaconOp::Depart
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn agent(id: u64) -> AgentId {
        AgentId(id)
    }

    // -- SignalStrength tests --

    #[test]
    fn signal_classification() {
        assert_eq!(SignalStrength::from_raw(10), SignalStrength::Weak);
        assert_eq!(SignalStrength::from_raw(30), SignalStrength::Medium);
        assert_eq!(SignalStrength::from_raw(60), SignalStrength::Strong);
        assert_eq!(SignalStrength::from_raw(90), SignalStrength::Excellent);
    }

    #[test]
    fn signal_usable() {
        assert!(SignalStrength::Weak.is_usable());
        assert!(SignalStrength::Strong.is_usable());
        assert!(!SignalStrength::None.is_usable());
    }

    // -- Beacon tests --

    #[test]
    fn beacon_new() {
        let b = Beacon::new(agent(1), 75, 100);
        assert_eq!(b.agent_id(), agent(1));
        assert!(b.is_active());
        assert_eq!(b.broadcast_range(), 75);
        assert_eq!(b.broadcast_count(), 0);
    }

    #[test]
    fn beacon_broadcast() {
        let mut b = Beacon::new(agent(1), 75, 100);
        let msg = b.broadcast(200);
        assert_eq!(msg.timestamp, 200);
        assert_eq!(b.broadcast_count(), 1);
        b.broadcast(300);
        assert_eq!(b.broadcast_count(), 2);
    }

    #[test]
    fn beacon_activate_deactivate() {
        let mut b = Beacon::new(agent(1), 75, 100);
        b.deactivate();
        assert!(!b.is_active());
        b.activate();
        assert!(b.is_active());
    }

    #[test]
    fn beacon_capabilities() {
        let mut b = Beacon::new(agent(1), 75, 100);
        b.add_capability("compute");
        b.add_capability("storage");
        assert!(b.message.has_capability("compute"));
        assert!(b.message.has_capability("storage"));
        b.remove_capability("compute");
        assert!(!b.message.has_capability("compute"));
    }

    #[test]
    fn beacon_no_duplicate_capability() {
        let mut b = Beacon::new(agent(1), 75, 100);
        b.add_capability("compute");
        b.add_capability("compute");
        assert_eq!(b.message.capabilities.len(), 1);
    }

    #[test]
    fn beacon_metadata() {
        let mut b = Beacon::new(agent(1), 75, 100);
        b.set_metadata("region", "us-west");
        assert_eq!(b.message.metadata.get("region"), Some(&"us-west".to_string()));
    }

    // -- BeaconScanner tests --

    #[test]
    fn scanner_basic_scan() {
        let mut scanner = BeaconScanner::new(agent(2), 80);
        let msgs = vec![
            BeaconMessage::new(agent(1), 60, 100),
            BeaconMessage::new(agent(3), 90, 100), // out of range
        ];
        let results = scanner.scan(&msgs);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].message.source, agent(1));
    }

    #[test]
    fn scanner_ignores_self() {
        let mut scanner = BeaconScanner::new(agent(1), 100);
        let msgs = vec![
            BeaconMessage::new(agent(1), 50, 100), // self
            BeaconMessage::new(agent(2), 50, 100),
        ];
        let results = scanner.scan(&msgs);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].message.source, agent(2));
    }

    #[test]
    fn scanner_sorted_by_signal() {
        let mut scanner = BeaconScanner::new(agent(0), 100);
        let msgs = vec![
            BeaconMessage::new(agent(1), 30, 100),
            BeaconMessage::new(agent(2), 80, 100),
            BeaconMessage::new(agent(3), 50, 100),
        ];
        let results = scanner.scan(&msgs);
        assert_eq!(results[0].message.source, agent(2)); // strongest first
    }

    #[test]
    fn scanner_count_above() {
        let mut scanner = BeaconScanner::new(agent(0), 100);
        let msgs = vec![
            BeaconMessage::new(agent(1), 20, 100),
            BeaconMessage::new(agent(2), 60, 100),
        ];
        scanner.scan(&msgs);
        assert_eq!(scanner.count_above(SignalStrength::Strong), 1);
    }

    // -- BeaconFilter tests --

    #[test]
    fn filter_has_capability() {
        let filter = BeaconFilter::new(); // no criteria = pass all
        let msg = BeaconMessage::new(agent(1), 50, 100);
        assert!(!matches!(filter.apply(&msg), Ternary::Negative));
    }

    #[test]
    fn filter_positive_match() {
        let mut f = BeaconFilter::new();
        f.add_criterion(FilterCriterion::HasCapability("compute".to_string()));
        let msg = BeaconMessage::new(agent(1), 50, 100).with_capability("compute");
        assert_eq!(f.apply(&msg), Ternary::Positive);
    }

    #[test]
    fn filter_negative_match() {
        let mut f = BeaconFilter::new();
        f.add_criterion(FilterCriterion::HasCapability("compute".to_string()));
        let msg = BeaconMessage::new(agent(1), 50, 100);
        assert_eq!(f.apply(&msg), Ternary::Negative);
    }

    #[test]
    fn filter_not_capability() {
        let mut f = BeaconFilter::new();
        f.add_criterion(FilterCriterion::NotCapability("legacy".to_string()));
        let msg_good = BeaconMessage::new(agent(1), 50, 100).with_capability("compute");
        let msg_bad = BeaconMessage::new(agent(2), 50, 100).with_capability("legacy");
        assert_eq!(f.apply(&msg_good), Ternary::Positive);
        assert_eq!(f.apply(&msg_bad), Ternary::Negative);
    }

    #[test]
    fn filter_min_signal() {
        let mut f = BeaconFilter::new();
        f.add_criterion(FilterCriterion::MinSignal(50));
        let msg_strong = BeaconMessage::new(agent(1), 70, 100);
        let msg_weak = BeaconMessage::new(agent(2), 30, 100);
        assert_eq!(f.apply(&msg_strong), Ternary::Positive);
        assert_eq!(f.apply(&msg_weak), Ternary::Negative);
    }

    #[test]
    fn filter_metadata_match() {
        let mut f = BeaconFilter::new();
        f.add_criterion(FilterCriterion::HasMetadata("region".to_string(), "us-west".to_string()));
        let msg = BeaconMessage::new(agent(1), 50, 100).with_metadata("region", "us-west");
        assert_eq!(f.apply(&msg), Ternary::Positive);
    }

    #[test]
    fn filter_combined_any_negative_rejects() {
        let mut f = BeaconFilter::new();
        f.add_criterion(FilterCriterion::HasCapability("compute".to_string()));
        f.add_criterion(FilterCriterion::MinSignal(50));
        let msg = BeaconMessage::new(agent(1), 30, 100).with_capability("compute");
        assert_eq!(f.apply(&msg), Ternary::Negative); // fails signal check
    }

    #[test]
    fn filter_filter_method() {
        let mut f = BeaconFilter::new();
        f.add_criterion(FilterCriterion::MinSignal(50));
        let msgs = vec![
            BeaconMessage::new(agent(1), 70, 100),
            BeaconMessage::new(agent(2), 30, 100),
        ];
        let filtered = f.filter(&msgs);
        assert_eq!(filtered.len(), 1);
    }

    // -- BeaconRegistry tests --

    #[test]
    fn registry_register_and_get() {
        let mut reg = BeaconRegistry::new(100);
        let msg = BeaconMessage::new(agent(1), 50, 100).with_capability("compute");
        reg.register(&msg);
        let entry = reg.get(agent(1)).unwrap();
        assert_eq!(entry.last_seen, 100);
        assert!(entry.capabilities.contains(&"compute".to_string()));
    }

    #[test]
    fn registry_is_active() {
        let mut reg = BeaconRegistry::new(100);
        let msg = BeaconMessage::new(agent(1), 50, 100);
        reg.register(&msg);
        assert!(reg.is_active(agent(1), 150));
        assert!(!reg.is_active(agent(1), 250)); // expired
    }

    #[test]
    fn registry_prune() {
        let mut reg = BeaconRegistry::new(100);
        reg.register(&BeaconMessage::new(agent(1), 50, 100));
        reg.register(&BeaconMessage::new(agent(2), 50, 200));
        let pruned = reg.prune(250);
        assert_eq!(pruned, 1); // agent 1 expired
        assert_eq!(reg.entry_count(), 1);
    }

    #[test]
    fn registry_capability_merge() {
        let mut reg = BeaconRegistry::new(100);
        reg.register(&BeaconMessage::new(agent(1), 50, 100).with_capability("a"));
        reg.register(&BeaconMessage::new(agent(1), 50, 200).with_capability("b"));
        let entry = reg.get(agent(1)).unwrap();
        assert!(entry.capabilities.contains(&"a".to_string()));
        assert!(entry.capabilities.contains(&"b".to_string()));
    }

    #[test]
    fn registry_active_entries() {
        let mut reg = BeaconRegistry::new(100);
        reg.register(&BeaconMessage::new(agent(1), 50, 100));
        reg.register(&BeaconMessage::new(agent(2), 50, 150));
        let active = reg.active_entries(180);
        assert_eq!(active.len(), 2);
    }

    // -- ProtocolMessage tests --

    #[test]
    fn protocol_announce() {
        let msg = ProtocolMessage::announce(agent(1), 100, 75);
        assert!(msg.is_announce());
        assert!(!msg.is_depart());
        assert_eq!(msg.version, PROTOCOL_VERSION);
        assert_eq!(msg.payload.signal_strength, 75);
    }

    #[test]
    fn protocol_depart() {
        let msg = ProtocolMessage::depart(agent(1), 100);
        assert!(msg.is_depart());
        assert_eq!(msg.payload.signal_strength, 0);
    }
}
