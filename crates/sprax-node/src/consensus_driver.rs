use parking_lot::RwLock;
use sprax_consensus::{
    BftConsensusEngine, ConsensusError, ConsensusTimeoutConfig, EquivocationEvidence,
    StakingKeeper, Vote, VoteType,
};
use sprax_core::ChainLedger;
use sprax_crypto::{Ed25519Keypair, Hasher};
use sprax_network::P2pService;
use sprax_storage::RedbStore;
use sprax_types::{Address, Block, CommitSignature, Hash32};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tokio::{
    sync::mpsc,
    time::{Duration, Instant},
};
use tracing::{info, warn};

#[derive(Debug)]
pub struct ConsensusDriver {
    engine: BftConsensusEngine,
    staking: Arc<RwLock<StakingKeeper>>,
    ledger: Arc<RwLock<ChainLedger<RedbStore>>>,
    p2p: P2pService,
    local_key: Ed25519Keypair,
    timeouts: ConsensusTimeoutConfig,
    inbound_vote_rx: mpsc::UnboundedReceiver<Vote>,
    inbound_proposal_rx: mpsc::UnboundedReceiver<(u64, u32, Block)>,
    is_running: Arc<AtomicBool>,
    precommitted_this_round: bool,
    /// Height of the most recent `run_one_height` attempt, paired with `current_round`: a
    /// retry of the *same* height increments the round instead of silently reusing round 0
    /// (see [`sprax_consensus::BftConsensusEngine::set_round`] for why that matters).
    last_attempted_height: u64,
    current_round: u32,
    min_peers_before_start: usize,
}

impl ConsensusDriver {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        engine: BftConsensusEngine,
        staking: Arc<RwLock<StakingKeeper>>,
        ledger: Arc<RwLock<ChainLedger<RedbStore>>>,
        p2p: P2pService,
        local_key: Ed25519Keypair,
        timeouts: ConsensusTimeoutConfig,
        inbound_vote_rx: mpsc::UnboundedReceiver<Vote>,
        inbound_proposal_rx: mpsc::UnboundedReceiver<(u64, u32, Block)>,
        is_running: Arc<AtomicBool>,
        min_peers_before_start: usize,
    ) -> Self {
        Self {
            engine,
            staking,
            ledger,
            p2p,
            local_key,
            timeouts,
            inbound_vote_rx,
            inbound_proposal_rx,
            is_running,
            precommitted_this_round: false,
            last_attempted_height: 0,
            current_round: 0,
            min_peers_before_start,
        }
    }

    /// Drives one BFT round per height for as long as the node is running.
    pub async fn run(mut self) {
        self.wait_for_peers_if_needed().await;
        let target_ms = self
            .ledger
            .read()
            .genesis()
            .consensus_params
            .block_time_target_ms;
        let target_duration = Duration::from_millis(target_ms.max(100));

        while self.is_running.load(Ordering::SeqCst) {
            let start = Instant::now();
            self.run_one_height().await;
            let elapsed = start.elapsed();
            if let Some(remaining) = target_duration.checked_sub(elapsed) {
                tokio::time::sleep(remaining).await;
            }
        }
    }

    async fn wait_for_peers_if_needed(&self) {
        let val_set = match self.staking.read().get_active_validator_set() {
            Ok(vs) => vs,
            Err(_) => return,
        };
        let self_power = val_set
            .validators()
            .iter()
            .find(|v| v.address == self.local_key.address())
            .map(|v| v.voting_power)
            .unwrap_or(0);
        if val_set.has_quorum(self_power) {
            return;
        }
        // Wait for the *configured* peer set (not just "any one peer") before producing the
        // first proposal: each height is only ever proposed/broadcast once (no re-broadcast,
        // no rollback on failed quorum — see the milestone's documented limitations), so a
        // validator that starts racing ahead before a slightly-slower-to-connect peer has
        // joined can leave that peer permanently unable to catch up on that height.
        let deadline = Instant::now() + Duration::from_secs(10);
        while self.p2p.connected_peers_count() < self.min_peers_before_start
            && Instant::now() < deadline
        {
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    async fn run_one_height(&mut self) {
        let next_height = self.ledger.read().height() + 1;
        let round = if next_height == self.last_attempted_height {
            self.current_round += 1;
            self.current_round
        } else {
            self.last_attempted_height = next_height;
            self.current_round = 0;
            0
        };

        // Extracted into an owned Result first: `parking_lot::RwLockReadGuard` is `!Send`, and
        // holding it as a match scrutinee temporary across the `.await` below (inside the Err
        // arm) would make this function's generated future non-Send, breaking `tokio::spawn`.
        let val_set_result = self.staking.read().get_active_validator_set();
        let val_set = match val_set_result {
            Ok(vs) => vs,
            Err(e) => {
                warn!("no active validator set available, retrying: {e}");
                tokio::time::sleep(Duration::from_millis(500)).await;
                return;
            }
        };
        self.engine.set_validator_set(val_set);
        self.engine.start_height(next_height);
        self.engine.set_round(round);
        self.precommitted_this_round = false;

        let proposer = self.engine.select_proposer();
        let local_addr = self.local_key.address();

        let block_hash = if proposer.address == local_addr {
            self.propose_as_local(next_height, round, proposer.address)
                .await
        } else {
            self.await_proposal(next_height, round, proposer.address)
                .await
        };

        let Some(block_hash) = block_hash else {
            return;
        };

        if let Ok(vote) = Self::build_signed_vote(
            VoteType::Prevote,
            next_height,
            round,
            Some(block_hash),
            &self.local_key,
        ) {
            self.broadcast_and_feed_vote(vote).await;
            if self.maybe_cast_reactive_precommit(next_height, round).await {
                return;
            }
        }

        self.process_votes_until_finalized(next_height, round).await;
    }

    async fn propose_as_local(
        &mut self,
        height: u64,
        round: u32,
        proposer_addr: Address,
    ) -> Option<Hash32> {
        let mined = match self.ledger.write().mine_block(proposer_addr) {
            Ok(b) => b,
            Err(e) => {
                warn!(height, "failed to mine proposed block: {e}");
                return None;
            }
        };
        let block_hash = match Hasher::block_hash(&mined.header) {
            Ok(h) => h,
            Err(e) => {
                warn!("failed to hash proposed block: {e}");
                return None;
            }
        };
        self.p2p.broadcast_proposal(height, round, mined);
        match self.engine.propose_block(block_hash, proposer_addr) {
            Ok(()) => Some(block_hash),
            Err(e) => {
                warn!("propose_block rejected: {e}");
                None
            }
        }
    }

    async fn await_proposal(
        &mut self,
        height: u64,
        round: u32,
        expected_proposer: Address,
    ) -> Option<Hash32> {
        // Drain (rather than bail out on the first) mismatched/stale proposals within the same
        // overall deadline: a node that started slightly behind can have a backlog of Proposal
        // messages for heights/rounds it has already missed queued ahead of the one it
        // actually needs (TCP delivers in order, so the wanted one — if it was ever sent while
        // we were connected — is still somewhere in that backlog). Bailing out on the very
        // first mismatch would make such a node permanently unable to catch up. Bounded by
        // `deadline` (not reset per message) so a truly empty/slow channel still gives up
        // after `timeout_propose_ms`, same as before.
        let deadline = Instant::now() + Duration::from_millis(self.timeouts.timeout_propose_ms);
        let received = loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                break None;
            }
            match tokio::time::timeout(remaining, self.inbound_proposal_rx.recv()).await {
                Ok(Some((h, r, block))) if h == height && r == round => break Some(block),
                Ok(Some(_stale_or_future)) => continue,
                _ => break None,
            }
        };

        let Some(block) = received else {
            warn!(height, round, "propose timeout / no proposal received");
            self.cast_nil_prevote(height, round).await;
            return None;
        };

        if block.header.proposer != expected_proposer {
            warn!(
                height,
                round, "received proposal from unexpected proposer, ignoring"
            );
            self.cast_nil_prevote(height, round).await;
            return None;
        }

        let apply_result = self.ledger.write().apply_block(block.clone());
        match apply_result {
            Ok(_) => match Hasher::block_hash(&block.header) {
                Ok(bh) => {
                    let _ = self.engine.propose_block(bh, expected_proposer);
                    Some(bh)
                }
                Err(e) => {
                    warn!("failed to hash received proposal: {e}");
                    None
                }
            },
            Err(e) => {
                warn!(height, round, "rejected invalid proposal: {e}");
                self.cast_nil_prevote(height, round).await;
                None
            }
        }
    }

    async fn cast_nil_prevote(&mut self, height: u64, round: u32) {
        if let Ok(vote) =
            Self::build_signed_vote(VoteType::Prevote, height, round, None, &self.local_key)
        {
            self.broadcast_and_feed_vote(vote).await;
        }
    }

    async fn process_votes_until_finalized(&mut self, height: u64, round: u32) {
        let deadline = Instant::now()
            + Duration::from_millis(
                self.timeouts.timeout_prevote_ms
                    + self.timeouts.timeout_precommit_ms
                    + self.timeouts.timeout_commit_ms
                    + 2_000,
            );
        loop {
            let now = Instant::now();
            if now >= deadline {
                warn!(
                    height,
                    round, "consensus round timed out waiting for quorum"
                );
                return;
            }
            match tokio::time::timeout(deadline - now, self.inbound_vote_rx.recv()).await {
                Ok(Some(vote)) => {
                    if vote.height != height || vote.round != round {
                        continue;
                    }
                    if self.handle_inbound_vote(vote).await {
                        return;
                    }
                }
                _ => return,
            }
        }
    }

    async fn handle_inbound_vote(&mut self, vote: Vote) -> bool {
        let validator = match self
            .engine
            .validator_set()
            .validators()
            .iter()
            .find(|v| v.address == vote.validator_address)
            .cloned()
        {
            Some(v) => v,
            None => return false,
        };

        let sign_bytes = match vote.sign_bytes() {
            Ok(b) => b,
            Err(_) => return false,
        };
        if Ed25519Keypair::verify(&validator.public_key, &sign_bytes, &vote.signature).is_err() {
            warn!(validator = %vote.validator_address, "dropping vote with invalid signature");
            return false;
        }

        if let Some(conflicting) = self.engine.find_conflicting_vote(&vote) {
            let evidence = EquivocationEvidence {
                validator_address: vote.validator_address,
                height: vote.height,
                round: vote.round,
                vote_a: conflicting,
                vote_b: vote.clone(),
            };
            if evidence.is_valid_equivocation() {
                if let Ok(slashed) = self.staking.write().slash_equivocation(&evidence) {
                    if !slashed.is_zero() {
                        warn!(
                            validator = %vote.validator_address,
                            %slashed,
                            "equivocation detected: validator slashed"
                        );
                        self.p2p.broadcast_evidence(evidence);
                    }
                }
            }
        }

        match vote.vote_type {
            VoteType::Prevote => {
                let height = vote.height;
                let round = vote.round;
                match self.engine.receive_prevote(vote) {
                    Ok(true) => self.maybe_cast_reactive_precommit(height, round).await,
                    _ => false,
                }
            }
            VoteType::Precommit => {
                let height = vote.height;
                match self.engine.receive_precommit(vote) {
                    Ok(Some(commit_sigs)) => {
                        self.finalize_height(height, commit_sigs);
                        true
                    }
                    _ => false,
                }
            }
        }
    }

    async fn maybe_cast_reactive_precommit(&mut self, height: u64, round: u32) -> bool {
        if self.precommitted_this_round {
            return false;
        }
        let Some(bh) = self.engine.get_prevoted_block_with_quorum(height, round) else {
            return false;
        };
        let vote = match Self::build_signed_vote(
            VoteType::Precommit,
            height,
            round,
            Some(bh),
            &self.local_key,
        ) {
            Ok(v) => v,
            Err(_) => return false,
        };
        self.precommitted_this_round = true;
        self.broadcast_and_feed_vote(vote).await
    }

    async fn broadcast_and_feed_vote(&mut self, vote: Vote) -> bool {
        let vote_type = vote.vote_type;
        let height = vote.height;
        self.p2p.broadcast_vote(vote.clone());
        match vote_type {
            VoteType::Prevote => {
                let _ = self.engine.receive_prevote(vote);
                false
            }
            VoteType::Precommit => match self.engine.receive_precommit(vote) {
                Ok(Some(commit_sigs)) => {
                    self.finalize_height(height, commit_sigs);
                    true
                }
                _ => false,
            },
        }
    }

    fn finalize_height(&mut self, height: u64, commit_sigs: Vec<CommitSignature>) {
        if let Err(e) = self.ledger.write().set_last_commit(height, commit_sigs) {
            warn!(height, "failed to record finalized commit signatures: {e}");
        } else {
            info!(height, "block finalized with +2/3 precommit quorum");
        }
        // Defensive re-broadcast: peers that missed the Proposal message still converge via
        // the existing, already-tested BlockGossip path.
        if let Some(block) = self.ledger.read().get_block_by_height(height).cloned() {
            self.p2p.broadcast_block(block);
        }
    }

    fn build_signed_vote(
        vote_type: VoteType,
        height: u64,
        round: u32,
        block_hash: Option<Hash32>,
        signer: &Ed25519Keypair,
    ) -> Result<Vote, ConsensusError> {
        let mut vote = Vote::new(
            vote_type,
            height,
            round,
            block_hash,
            signer.address(),
            vec![],
        );
        let sign_bytes = vote.sign_bytes()?;
        vote.signature = signer.sign(&sign_bytes);
        Ok(vote)
    }
}

pub async fn run_evidence_listener(
    staking: Arc<RwLock<StakingKeeper>>,
    mut inbound_evidence_rx: mpsc::UnboundedReceiver<EquivocationEvidence>,
    is_running: Arc<AtomicBool>,
) {
    while is_running.load(Ordering::SeqCst) {
        match inbound_evidence_rx.recv().await {
            Some(evidence) => {
                if !evidence.is_valid_equivocation() {
                    continue;
                }
                match staking.write().slash_equivocation(&evidence) {
                    Ok(slashed) if !slashed.is_zero() => {
                        warn!(
                            validator = %evidence.validator_address,
                            %slashed,
                            "received equivocation evidence from peer: validator slashed"
                        );
                    }
                    Ok(_) => {}
                    Err(e) => warn!("failed to apply received equivocation evidence: {e}"),
                }
            }
            None => break,
        }
    }
}
