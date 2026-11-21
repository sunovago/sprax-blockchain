//! Bounded, cryptographically validated recovery for missing block proposals.
//!
//! Provides in-memory bounded caching of recent verified proposals, request correlation,
//! deduplication, rate limits, and verification pipelines so honest validators can recover
//! missed proposals without stalling consensus or exposing the node to unbounded resource attacks.

use sprax_consensus::{SignedProposal, ValidatorSet};
use sprax_crypto::Hasher;
use sprax_network::{NetworkMessage, PeerId, ProposalResponseMsg};
use sprax_types::{Address, Hash32};
use std::{
    collections::{BTreeMap, HashMap},
    time::{Duration, Instant},
};
use tracing::{debug, warn};

/// Maximum number of recent verified proposals retained in the recovery cache.
pub const MAX_CACHE_ENTRIES: usize = 32;

/// Maximum aggregate serialized byte budget for the proposal cache (16 MiB).
pub const MAX_CACHE_BYTES: usize = 16 * 1024 * 1024;

/// Maximum concurrent in-flight proposal recovery requests.
pub const MAX_PENDING_REQUESTS: usize = 16;

/// Default timeout for a pending proposal recovery request.
pub const DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_millis(400);

/// Maximum retry attempts before giving up on a specific (height, round, block_hash) target.
pub const MAX_REQUEST_RETRIES: usize = 3;

/// Bounded LRU-style cache of validated, signed block proposals.
#[derive(Debug, Default)]
pub struct BoundedProposalCache {
    /// Keyed by (height, round)
    entries: BTreeMap<(u64, u32), (SignedProposal, usize)>,
    total_bytes: usize,
}

impl BoundedProposalCache {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts a validated proposal into the cache, enforcing entry and byte budgets.
    pub fn insert(&mut self, proposal: SignedProposal) {
        let height = proposal.block.header.height;
        let round = proposal.round;
        let key = (height, round);

        if self.entries.contains_key(&key) {
            return;
        }

        let estimated_bytes = serde_json::to_vec(&proposal)
            .map(|v| v.len())
            .unwrap_or(4096);
        if estimated_bytes > MAX_CACHE_BYTES {
            warn!(height, round, "proposal exceeds total cache byte budget");
            return;
        }

        // Evict oldest entries until within entry and byte limits
        while self.entries.len() >= MAX_CACHE_ENTRIES
            || (self.total_bytes + estimated_bytes > MAX_CACHE_BYTES && !self.entries.is_empty())
        {
            if let Some((old_key, (_, old_bytes))) = self.entries.pop_first() {
                self.total_bytes = self.total_bytes.saturating_sub(old_bytes);
                debug!(
                    height = old_key.0,
                    round = old_key.1,
                    "evicted proposal from recovery cache"
                );
            }
        }

        self.total_bytes += estimated_bytes;
        self.entries.insert(key, (proposal, estimated_bytes));
    }

    /// Fetches a proposal matching the exact request coordinates.
    #[must_use]
    pub fn get(
        &self,
        genesis: Hash32,
        height: u64,
        round: u32,
        block_hash: Option<Hash32>,
    ) -> Option<SignedProposal> {
        let (proposal, _) = self.entries.get(&(height, round))?;
        if proposal.genesis != genesis {
            return None;
        }
        if let Some(expected_hash) = block_hash {
            let actual_hash = Hasher::block_hash(&proposal.block.header).ok()?;
            if actual_hash != expected_hash {
                return None;
            }
        }
        Some(proposal.clone())
    }

    /// Prunes all cached proposals older than the given minimum height.
    pub fn prune_older_than(&mut self, min_height: u64) {
        let keys_to_remove: Vec<(u64, u32)> = self
            .entries
            .keys()
            .filter(|(h, _)| *h < min_height)
            .copied()
            .collect();

        for key in keys_to_remove {
            if let Some((_, bytes)) = self.entries.remove(&key) {
                self.total_bytes = self.total_bytes.saturating_sub(bytes);
            }
        }
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    #[must_use]
    pub fn total_bytes(&self) -> usize {
        self.total_bytes
    }
}

/// Metadata tracked for an active, pending proposal recovery request.
#[derive(Debug, Clone)]
pub struct PendingProposalRequest {
    pub request_id: u64,
    pub genesis: Hash32,
    pub height: u64,
    pub round: u32,
    pub block_hash: Option<Hash32>,
    pub expected_proposer: Address,
    pub target_peer: Option<PeerId>,
    pub created_at: Instant,
    pub retries: usize,
}

/// Specific reason an incoming proposal response was rejected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecoveryRejectReason {
    Unsolicited,
    GenesisMismatch,
    HeightMismatch,
    RoundMismatch,
    BlockHashMismatch,
    ProposerMismatch,
    InvalidSignature(String),
    LedgerValidationFailed(String),
}

impl std::fmt::Display for RecoveryRejectReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unsolicited => write!(f, "unsolicited or expired request ID"),
            Self::GenesisMismatch => write!(f, "genesis fingerprint mismatch"),
            Self::HeightMismatch => write!(f, "height mismatch"),
            Self::RoundMismatch => write!(f, "round mismatch"),
            Self::BlockHashMismatch => write!(f, "block hash mismatch"),
            Self::ProposerMismatch => write!(f, "proposer address mismatch"),
            Self::InvalidSignature(e) => write!(f, "invalid proposal signature: {e}"),
            Self::LedgerValidationFailed(e) => write!(f, "ledger validation failed: {e}"),
        }
    }
}

/// Observability snapshot for missing proposal recovery operations.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RecoveryMetrics {
    pub requests_sent: u64,
    pub responses_received: u64,
    pub proposals_recovered: u64,
    pub unsolicited_dropped: u64,
    pub rejected_invalid: u64,
    pub pending_requests_count: usize,
    pub cached_proposals_count: usize,
    pub cache_bytes_used: usize,
}

/// State machine managing outbound proposal requests, correlation, and response admission.
#[derive(Debug)]
pub struct ProposalRecoveryManager {
    genesis: Hash32,
    next_request_id: u64,
    pending: HashMap<u64, PendingProposalRequest>,
    last_request_time: HashMap<(u64, u32, Option<Hash32>), Instant>,
    // Bounded telemetry counters
    requests_sent: u64,
    responses_received: u64,
    proposals_recovered: u64,
    unsolicited_dropped: u64,
    rejected_invalid: u64,
}

impl ProposalRecoveryManager {
    #[must_use]
    pub fn new(genesis: Hash32) -> Self {
        Self {
            genesis,
            next_request_id: 1,
            pending: HashMap::new(),
            last_request_time: HashMap::new(),
            requests_sent: 0,
            responses_received: 0,
            proposals_recovered: 0,
            unsolicited_dropped: 0,
            rejected_invalid: 0,
        }
    }

    /// Evaluates whether a request should be dispatched. Returns `Some(NetworkMessage)` if eligible.
    pub fn create_request(
        &mut self,
        height: u64,
        round: u32,
        block_hash: Option<Hash32>,
        expected_proposer: Address,
        target_peer: Option<PeerId>,
    ) -> Option<(u64, NetworkMessage)> {
        // Enforce maximum concurrent pending request cap
        if self.pending.len() >= MAX_PENDING_REQUESTS {
            warn!(
                height,
                round, "max pending proposal recovery requests reached; throttling"
            );
            return None;
        }

        let dedup_key = (height, round, block_hash);
        let now = Instant::now();

        // Enforce backoff between duplicate requests for the exact same target
        if let Some(last) = self.last_request_time.get(&dedup_key) {
            if now.duration_since(*last) < Duration::from_millis(200) {
                return None;
            }
        }

        let request_id = self.next_request_id;
        self.next_request_id = self.next_request_id.wrapping_add(1).max(1);

        let req = PendingProposalRequest {
            request_id,
            genesis: self.genesis,
            height,
            round,
            block_hash,
            expected_proposer,
            target_peer,
            created_at: now,
            retries: 0,
        };

        self.pending.insert(request_id, req);
        self.last_request_time.insert(dedup_key, now);
        self.requests_sent = self.requests_sent.saturating_add(1);

        let msg = NetworkMessage::ProposalRequest {
            request_id,
            genesis: self.genesis,
            height,
            round,
            block_hash,
        };

        Some((request_id, msg))
    }

    /// Correlates an inbound response against pending requests and validates wire consistency.
    ///
    /// If unsolicited or invalid, drops immediately and updates negative observability counters.
    pub fn correlate_response(
        &mut self,
        resp: &ProposalResponseMsg,
    ) -> Result<PendingProposalRequest, RecoveryRejectReason> {
        self.responses_received = self.responses_received.saturating_add(1);

        let pending = match self.pending.remove(&resp.request_id) {
            Some(req) => req,
            None => {
                self.unsolicited_dropped = self.unsolicited_dropped.saturating_add(1);
                return Err(RecoveryRejectReason::Unsolicited);
            }
        };

        if resp.genesis != self.genesis || resp.genesis != pending.genesis {
            self.rejected_invalid = self.rejected_invalid.saturating_add(1);
            return Err(RecoveryRejectReason::GenesisMismatch);
        }

        if resp.height != pending.height || resp.proposal.block.header.height != pending.height {
            self.rejected_invalid = self.rejected_invalid.saturating_add(1);
            return Err(RecoveryRejectReason::HeightMismatch);
        }

        if resp.round != pending.round || resp.proposal.round != pending.round {
            self.rejected_invalid = self.rejected_invalid.saturating_add(1);
            return Err(RecoveryRejectReason::RoundMismatch);
        }

        if let Some(expected_hash) = pending.block_hash {
            let actual_hash = Hasher::block_hash(&resp.proposal.block.header)
                .map_err(|_| RecoveryRejectReason::BlockHashMismatch)?;
            if actual_hash != expected_hash {
                self.rejected_invalid = self.rejected_invalid.saturating_add(1);
                return Err(RecoveryRejectReason::BlockHashMismatch);
            }
        }

        if resp.proposal.signer != pending.expected_proposer {
            self.rejected_invalid = self.rejected_invalid.saturating_add(1);
            return Err(RecoveryRejectReason::ProposerMismatch);
        }

        Ok(pending)
    }

    /// Marks that a proposal was verified and accepted into the consensus engine.
    pub fn record_success(&mut self) {
        self.proposals_recovered = self.proposals_recovered.saturating_add(1);
    }

    /// Marks that a response was cryptographically or ledger-invalid.
    pub fn record_invalid(&mut self) {
        self.rejected_invalid = self.rejected_invalid.saturating_add(1);
    }

    /// Cleans up expired pending requests.
    pub fn prune_expired(&mut self, timeout: Duration) -> usize {
        let now = Instant::now();
        let initial_len = self.pending.len();
        self.pending
            .retain(|_, req| now.duration_since(req.created_at) < timeout);
        initial_len.saturating_sub(self.pending.len())
    }

    /// Purges all requests and deduplication timestamps for heights strictly below `current_height`.
    pub fn cleanup_height(&mut self, current_height: u64) {
        self.pending.retain(|_, req| req.height >= current_height);
        self.last_request_time
            .retain(|(h, _, _), _| *h >= current_height);
    }

    #[must_use]
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    #[must_use]
    pub fn metrics(&self, cache: &BoundedProposalCache) -> RecoveryMetrics {
        RecoveryMetrics {
            requests_sent: self.requests_sent,
            responses_received: self.responses_received,
            proposals_recovered: self.proposals_recovered,
            unsolicited_dropped: self.unsolicited_dropped,
            rejected_invalid: self.rejected_invalid,
            pending_requests_count: self.pending.len(),
            cached_proposals_count: cache.len(),
            cache_bytes_used: cache.total_bytes(),
        }
    }
}

/// Helper function to perform full cryptographic and consensus verification of a recovered proposal.
pub fn verify_recovered_proposal(
    proposal: &SignedProposal,
    genesis: Hash32,
    expected_proposer: Address,
    validators: &ValidatorSet,
) -> Result<(), RecoveryRejectReason> {
    proposal
        .verify(genesis, expected_proposer, validators)
        .map_err(|e| RecoveryRejectReason::InvalidSignature(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use sprax_crypto::Ed25519Keypair;
    use sprax_types::{Block, BlockBody, BlockHeader};

    fn make_proposal(
        genesis: Hash32,
        height: u64,
        round: u32,
        key: &Ed25519Keypair,
    ) -> SignedProposal {
        let mut header = BlockHeader::genesis("sprax-testnet-1", Hash32::new([1; 32]));
        header.height = height;
        header.proposer = key.address();
        let mut proposal = SignedProposal {
            genesis,
            signer: key.address(),
            round,
            valid_round: None,
            valid_round_votes: Vec::new(),
            block: Block {
                header,
                body: BlockBody::default(),
                last_commit: vec![],
            },
            signature: Vec::new(),
        };
        proposal.signature = key.sign(&proposal.sign_bytes().unwrap());
        proposal
    }

    #[test]
    fn test_bounded_proposal_cache_eviction() {
        let mut cache = BoundedProposalCache::new();
        let key = Ed25519Keypair::generate();
        let genesis = Hash32::new([42; 32]);

        for r in 0..(MAX_CACHE_ENTRIES + 5) as u32 {
            let p = make_proposal(genesis, 1, r, &key);
            cache.insert(p);
        }

        assert_eq!(cache.len(), MAX_CACHE_ENTRIES);
        assert!(cache.total_bytes() <= MAX_CACHE_BYTES);
        // The earliest entries (0..5) should have been evicted
        assert!(cache.get(genesis, 1, 0, None).is_none());
        assert!(cache
            .get(genesis, 1, (MAX_CACHE_ENTRIES + 4) as u32, None)
            .is_some());
    }

    #[test]
    fn test_request_correlation_and_unsolicited_rejection() {
        let genesis = Hash32::new([99; 32]);
        let mut manager = ProposalRecoveryManager::new(genesis);
        let key = Ed25519Keypair::generate();

        let (req_id, msg) = manager
            .create_request(10, 1, None, key.address(), None)
            .unwrap();

        assert_eq!(req_id, 1);
        assert!(matches!(
            msg,
            NetworkMessage::ProposalRequest { request_id: 1, .. }
        ));

        let proposal = make_proposal(genesis, 10, 1, &key);
        let resp = ProposalResponseMsg {
            peer_id: PeerId::new("peer-1").unwrap(),
            request_id: req_id,
            genesis,
            height: 10,
            round: 1,
            proposal: Box::new(proposal),
        };

        // Correlate valid response
        let correlated = manager.correlate_response(&resp);
        assert!(correlated.is_ok());
        assert_eq!(manager.pending_count(), 0);

        // Submitting same response again must fail as unsolicited
        let duplicate = manager.correlate_response(&resp);
        assert_eq!(duplicate.unwrap_err(), RecoveryRejectReason::Unsolicited);
    }

    #[test]
    fn test_mismatched_response_coordinates_rejected() {
        let genesis = Hash32::new([99; 32]);
        let mut manager = ProposalRecoveryManager::new(genesis);
        let key = Ed25519Keypair::generate();

        let (req_id, _) = manager
            .create_request(10, 1, None, key.address(), None)
            .unwrap();

        let bad_height_prop = make_proposal(genesis, 11, 1, &key);
        let bad_resp = ProposalResponseMsg {
            peer_id: PeerId::new("peer-1").unwrap(),
            request_id: req_id,
            genesis,
            height: 11,
            round: 1,
            proposal: Box::new(bad_height_prop),
        };

        assert_eq!(
            manager.correlate_response(&bad_resp).unwrap_err(),
            RecoveryRejectReason::HeightMismatch
        );
    }
}
