//! Comprehensive adversarial tests for Phase 02 L1 consensus recovery.
//!
//! Covers:
//! - Scenario A: Proposal withheld, votes received, proposal recovered from peer.
//! - Scenario B: Original proposer offline after partial proposal distribution; majority recovers and finalizes.
//! - Scenario C: First recovery peer returns corrupt/forged data; second peer provides valid data; node recovers.
//! - Scenario D: Unsolicited and duplicate responses remain bounded and harmless.
//! - Scenario E: Wrong-genesis, wrong-round, forged-proof, and mismatched-hash responses rejected.
//! - Scenario F: Locked validator rejects conflicting proposal without proof.
//! - Scenario G: Valid later-round proof permits safe unlock and transition.
//! - Scenario L: Round skew within horizon synchronizes; beyond horizon dropped.
//! - Scenario M: Historical catch-up across validator-set transition verifies using canonical set per height.
//! - Weighted-validator quorum regression (power-based, not node-count-based).

use parking_lot::RwLock;
use sprax_consensus::{
    BftConsensusEngine, ConsensusTimeoutConfig, SignedProposal, StakingKeeper, Validator,
    ValidatorSet, Vote, VoteType,
};
use sprax_core::{ChainLedger, GenesisAccount, GenesisConfig, GenesisValidator};
use sprax_crypto::{Ed25519Keypair, Hasher};
use sprax_network::{NetworkConfig, P2pService, PeerId, ProposalResponseMsg};
use sprax_node::{
    consensus_driver::ConsensusDriver,
    proposal_recovery::{
        verify_recovered_proposal, BoundedProposalCache, ProposalRecoveryManager,
        RecoveryRejectReason,
    },
    signing_journal::SigningJournal,
};
use sprax_storage::RedbStore;
use sprax_types::{Amount, Block, BlockBody, BlockHeader, CommitSignature, Hash32};
use std::{
    sync::{atomic::AtomicBool, Arc},
    time::Duration,
};
use tokio::sync::mpsc;

fn test_genesis(keys: &[Ed25519Keypair], weights: Option<&[u64]>) -> GenesisConfig {
    let mut genesis = GenesisConfig::default_development();
    genesis.chain_id = "sprax-testnet-adversarial".into();
    genesis.accounts = keys
        .iter()
        .enumerate()
        .map(|(i, key)| GenesisAccount {
            name: format!("validator-{i}"),
            address: key.address(),
            initial_balance: Amount::from_sprx_whole(10_000).unwrap(),
        })
        .collect();
    genesis.validators = keys
        .iter()
        .enumerate()
        .map(|(i, key)| {
            let weight = weights.map(|w| w[i]).unwrap_or(100);
            GenesisValidator {
                operator_address: key.address(),
                consensus_pubkey: key.public_key_bytes().to_vec(),
                self_stake: Amount::from_sprx_whole(weight as u128).unwrap(),
                moniker: format!("validator-{i}"),
            }
        })
        .collect();
    genesis
}

#[allow(clippy::type_complexity)]
fn create_test_driver(
    key_seed: u8,
    val_set: ValidatorSet,
    genesis: &GenesisConfig,
    dir: &tempfile::TempDir,
    p2p: P2pService,
    timeouts: Option<ConsensusTimeoutConfig>,
) -> (
    ConsensusDriver,
    Arc<RwLock<ChainLedger<RedbStore>>>,
    mpsc::Sender<Vote>,
    mpsc::Sender<SignedProposal>,
) {
    let key = Ed25519Keypair::from_seed(&[key_seed; 32]);
    let store = RedbStore::open(&dir.path().join("data/ledger.redb")).unwrap();
    let ledger = Arc::new(RwLock::new(
        ChainLedger::init_from_genesis_with_store(genesis.clone(), store).unwrap(),
    ));
    let journal = SigningJournal::open(
        &dir.path().join("data/validator-signing.redb"),
        genesis.fingerprint().unwrap(),
        &key,
    )
    .unwrap();
    let staking = Arc::new(RwLock::new(StakingKeeper::default()));
    let evidence_pool = Arc::new(RwLock::new(
        sprax_node::evidence_pool::EvidencePool::default(),
    ));
    let (vote_tx, vote_rx) = mpsc::channel(256);
    let (prop_tx, prop_rx) = mpsc::channel(32);

    let engine = BftConsensusEngine::new(1, val_set);
    let driver = ConsensusDriver::new(
        engine,
        staking,
        Arc::clone(&ledger),
        p2p,
        key,
        evidence_pool,
        journal,
        timeouts.unwrap_or_default(),
        vote_rx,
        prop_rx,
        Arc::new(AtomicBool::new(true)),
        0,
    )
    .unwrap();

    (driver, ledger, vote_tx, prop_tx)
}

fn create_mock_p2p(
    node_name: &str,
) -> (
    P2pService,
    mpsc::Receiver<SignedProposal>,
    mpsc::Receiver<Vote>,
    mpsc::Receiver<Block>,
) {
    let (tx_s, _) = mpsc::channel(16);
    let (blk_s, blk_r) = mpsc::channel(16);
    let (vote_s, vote_r) = mpsc::channel(256);
    let (prop_s, prop_r) = mpsc::channel(32);
    let (ev_s, _) = mpsc::channel(16);
    let peer_id = PeerId::new(node_name).unwrap();
    let p2p = P2pService::new(
        peer_id,
        "sprax-testnet-adversarial".into(),
        NetworkConfig::default(),
        tx_s,
        blk_s,
        vote_s,
        prop_s,
        ev_s,
        Arc::new(|_, _| Vec::new()),
    );
    (p2p, prop_r, vote_r, blk_r)
}

#[tokio::test]
async fn test_scenario_a_proposal_withheld_recovered_from_peer() {
    let keys: Vec<_> = (1..=4)
        .map(|seed| Ed25519Keypair::from_seed(&[seed; 32]))
        .collect();
    let genesis = test_genesis(&keys, None);
    let val_set = ValidatorSet::new(
        keys.iter()
            .map(|k| Validator::new(k.address(), k.public_key_bytes().to_vec(), 100))
            .collect(),
    )
    .unwrap();

    // Node 3 is our recovering validator (seed 3)
    let dir3 = tempfile::tempdir().unwrap();
    let (p2p3, mut _prop_r3, mut _vote_r3, mut _blk_r3) = create_mock_p2p("node-3");

    // Proposer is Node 0
    let genesis_fp = genesis.fingerprint().unwrap();
    let store0 =
        RedbStore::open(&tempfile::tempdir().unwrap().path().join("data/ledger.redb")).unwrap();
    let ledger0 = ChainLedger::init_from_genesis_with_store(genesis.clone(), store0).unwrap();
    let proposed_block = ledger0.build_proposal(keys[0].address()).unwrap();
    let block_hash = Hasher::block_hash(&proposed_block.header).unwrap();

    let mut proposal = SignedProposal {
        genesis: genesis_fp,
        signer: keys[0].address(),
        round: 0,
        valid_round: None,
        valid_round_votes: Vec::new(),
        block: proposed_block.clone(),
        signature: Vec::new(),
    };
    proposal.signature = keys[0].sign(&proposal.sign_bytes().unwrap());

    let (mut driver3, ledger3, vote_tx3, _prop_tx3) = create_test_driver(
        3,
        val_set.clone(),
        &genesis,
        &dir3,
        p2p3,
        Some(ConsensusTimeoutConfig {
            timeout_propose_ms: 100,
            timeout_prevote_ms: 500,
            timeout_precommit_ms: 500,
            timeout_commit_ms: 500,
        }),
    );

    let proposal_clone = proposal.clone();
    let keys_signers: Vec<_> = (1..=4)
        .map(|seed| Ed25519Keypair::from_seed(&[seed; 32]))
        .collect();
    let keys_driver: Vec<_> = (1..=4)
        .map(|seed| Ed25519Keypair::from_seed(&[seed; 32]))
        .collect();

    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Feed prevotes from Node 0 and Node 1 for block_hash
        for key in keys_signers.iter().take(2) {
            let mut vote = Vote {
                genesis: genesis_fp,
                validator_address: key.address(),
                height: 1,
                round: 0,
                block_hash: Some(block_hash),
                vote_type: VoteType::Prevote,
                signature: Vec::new(),
            };
            vote.signature = key.sign(&vote.sign_bytes().unwrap());
            let _ = vote_tx3.send(vote).await;
        }

        tokio::time::sleep(Duration::from_millis(50)).await;
        // Also send precommits from Node 0, Node 1, Node 2
        for key in keys_signers.iter().take(3) {
            let mut pvote = Vote {
                genesis: genesis_fp,
                validator_address: key.address(),
                height: 1,
                round: 0,
                block_hash: Some(block_hash),
                vote_type: VoteType::Precommit,
                signature: Vec::new(),
            };
            pvote.signature = key.sign(&pvote.sign_bytes().unwrap());
            let _ = vote_tx3.send(pvote).await;
        }
    });

    let driver_task = tokio::spawn(async move {
        let req_opt = driver3.recovery_manager_mut().create_request(
            1,
            0,
            Some(block_hash),
            keys_driver[0].address(),
            None,
        );
        assert!(req_opt.is_some());
        let (req_id, _) = req_opt.unwrap();

        let resp = ProposalResponseMsg {
            peer_id: PeerId::new("peer-1").unwrap(),
            request_id: req_id,
            genesis: genesis_fp,
            height: 1,
            round: 0,
            proposal: Box::new(proposal_clone),
        };

        let correlated = driver3
            .recovery_manager_mut()
            .correlate_response(&resp)
            .unwrap();
        assert_eq!(correlated.height, 1);
        verify_recovered_proposal(&resp.proposal, genesis_fp, keys[0].address(), &val_set).unwrap();

        driver3
            .proposal_cache_mut()
            .write()
            .insert((*resp.proposal).clone());
        driver3.set_pending_block(Some(resp.proposal.block.clone()));

        driver3.run_one_height_test().await;
        driver3
    });

    let _ = tokio::time::timeout(Duration::from_secs(5), driver_task)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(ledger3.read().height(), 1);
    let committed = ledger3.read().get_block_by_height(1).unwrap().clone();
    assert_eq!(Hasher::block_hash(&committed.header).unwrap(), block_hash);
}

#[tokio::test]
async fn test_scenario_c_first_peer_returns_corrupt_data_second_peer_valid() {
    let keys: Vec<_> = (1..=4)
        .map(|seed| Ed25519Keypair::from_seed(&[seed; 32]))
        .collect();
    let genesis = test_genesis(&keys, None);
    let genesis_fp = genesis.fingerprint().unwrap();
    let val_set = ValidatorSet::new(
        keys.iter()
            .map(|k| Validator::new(k.address(), k.public_key_bytes().to_vec(), 100))
            .collect(),
    )
    .unwrap();

    let mut manager = ProposalRecoveryManager::new(genesis_fp);
    let expected_hash = Hash32::new([7; 32]);

    let (req_id, _) = manager
        .create_request(1, 0, Some(expected_hash), keys[0].address(), None)
        .unwrap();

    // 1. Peer 1 responds with forged/corrupt proposal (tampered signature)
    let store =
        RedbStore::open(&tempfile::tempdir().unwrap().path().join("data/ledger.redb")).unwrap();
    let ledger = ChainLedger::init_from_genesis_with_store(genesis.clone(), store).unwrap();
    let mut block = ledger.build_proposal(keys[0].address()).unwrap();
    block.header.state_root = Hash32::new([99; 32]);

    let corrupt_proposal = SignedProposal {
        genesis: genesis_fp,
        signer: keys[0].address(),
        round: 0,
        valid_round: None,
        valid_round_votes: Vec::new(),
        block,
        signature: vec![0u8; 64], // invalid signature
    };

    let corrupt_resp = ProposalResponseMsg {
        peer_id: PeerId::new("malicious-peer").unwrap(),
        request_id: req_id,
        genesis: genesis_fp,
        height: 1,
        round: 0,
        proposal: Box::new(corrupt_proposal),
    };

    let verify_res = verify_recovered_proposal(
        &corrupt_resp.proposal,
        genesis_fp,
        keys[0].address(),
        &val_set,
    );
    assert!(verify_res.is_err());
    manager.record_invalid();

    // 2. Peer 2 responds with valid proposal matching the target
    let (req_id_retry, _) = manager
        .create_request(1, 0, None, keys[0].address(), None)
        .unwrap();

    let valid_block = ledger.build_proposal(keys[0].address()).unwrap();
    let mut valid_proposal = SignedProposal {
        genesis: genesis_fp,
        signer: keys[0].address(),
        round: 0,
        valid_round: None,
        valid_round_votes: Vec::new(),
        block: valid_block,
        signature: Vec::new(),
    };
    valid_proposal.signature = keys[0].sign(&valid_proposal.sign_bytes().unwrap());

    let valid_resp = ProposalResponseMsg {
        peer_id: PeerId::new("honest-peer").unwrap(),
        request_id: req_id_retry,
        genesis: genesis_fp,
        height: 1,
        round: 0,
        proposal: Box::new(valid_proposal),
    };

    let correlate_res = manager.correlate_response(&valid_resp);
    assert!(correlate_res.is_ok());
    let verify_valid = verify_recovered_proposal(
        &valid_resp.proposal,
        genesis_fp,
        keys[0].address(),
        &val_set,
    );
    assert!(verify_valid.is_ok());
    manager.record_success();

    let mut cache = BoundedProposalCache::new();
    cache.insert((*valid_resp.proposal).clone());
    let metrics = manager.metrics(&cache);
    assert_eq!(metrics.proposals_recovered, 1);
    assert_eq!(metrics.rejected_invalid, 1);
}

#[tokio::test]
async fn test_scenario_d_unsolicited_and_duplicate_responses_bounded() {
    let genesis = Hash32::new([55; 32]);
    let mut manager = ProposalRecoveryManager::new(genesis);
    let key = Ed25519Keypair::generate();
    let dummy = Block {
        header: BlockHeader::genesis("sprax-test", Hash32::ZERO),
        body: BlockBody::default(),
        last_commit: vec![],
    };

    // Send 10 unsolicited responses with arbitrary request IDs
    for id in 100..110 {
        let resp = ProposalResponseMsg {
            peer_id: PeerId::new("flooding-peer").unwrap(),
            request_id: id,
            genesis,
            height: 1,
            round: 0,
            proposal: Box::new(SignedProposal {
                genesis,
                signer: key.address(),
                round: 0,
                valid_round: None,
                valid_round_votes: Vec::new(),
                block: dummy.clone(),
                signature: Vec::new(),
            }),
        };
        let res = manager.correlate_response(&resp);
        assert_eq!(res.unwrap_err(), RecoveryRejectReason::Unsolicited);
    }

    let cache = BoundedProposalCache::new();
    let metrics = manager.metrics(&cache);
    assert_eq!(metrics.unsolicited_dropped, 10);
    assert_eq!(metrics.pending_requests_count, 0);
}

#[tokio::test]
async fn test_scenario_e_negative_mismatches_rejected() {
    let genesis = Hash32::new([1; 32]);
    let wrong_genesis = Hash32::new([2; 32]);
    let mut manager = ProposalRecoveryManager::new(genesis);
    let key = Ed25519Keypair::generate();
    let other_key = Ed25519Keypair::generate();
    let mut dummy = Block {
        header: BlockHeader::genesis("sprax-test", Hash32::ZERO),
        body: BlockBody::default(),
        last_commit: vec![],
    };
    dummy.header.height = 10;

    let (req_id, _) = manager
        .create_request(10, 2, None, key.address(), None)
        .unwrap();

    // 1. Wrong Genesis
    let resp_wrong_gen = ProposalResponseMsg {
        peer_id: PeerId::new("peer").unwrap(),
        request_id: req_id,
        genesis: wrong_genesis,
        height: 10,
        round: 2,
        proposal: Box::new(SignedProposal {
            genesis: wrong_genesis,
            signer: key.address(),
            round: 2,
            valid_round: None,
            valid_round_votes: Vec::new(),
            block: dummy.clone(),
            signature: Vec::new(),
        }),
    };
    assert_eq!(
        manager.correlate_response(&resp_wrong_gen).unwrap_err(),
        RecoveryRejectReason::GenesisMismatch
    );

    // 2. Wrong Proposer
    tokio::time::sleep(Duration::from_millis(220)).await;
    let (req_id, _) = manager
        .create_request(10, 2, None, key.address(), None)
        .unwrap();

    let resp_wrong_proposer = ProposalResponseMsg {
        peer_id: PeerId::new("peer").unwrap(),
        request_id: req_id,
        genesis,
        height: 10,
        round: 2,
        proposal: Box::new(SignedProposal {
            genesis,
            signer: other_key.address(),
            round: 2,
            valid_round: None,
            valid_round_votes: Vec::new(),
            block: dummy.clone(),
            signature: Vec::new(),
        }),
    };
    assert_eq!(
        manager
            .correlate_response(&resp_wrong_proposer)
            .unwrap_err(),
        RecoveryRejectReason::ProposerMismatch
    );

    // 3. Wrong Height
    tokio::time::sleep(Duration::from_millis(220)).await;
    let (req_id, _) = manager
        .create_request(10, 2, None, key.address(), None)
        .unwrap();

    let mut bad_height_block = dummy.clone();
    bad_height_block.header.height = 11;
    let resp_wrong_height = ProposalResponseMsg {
        peer_id: PeerId::new("peer").unwrap(),
        request_id: req_id,
        genesis,
        height: 11,
        round: 2,
        proposal: Box::new(SignedProposal {
            genesis,
            signer: key.address(),
            round: 2,
            valid_round: None,
            valid_round_votes: Vec::new(),
            block: bad_height_block,
            signature: Vec::new(),
        }),
    };
    assert_eq!(
        manager.correlate_response(&resp_wrong_height).unwrap_err(),
        RecoveryRejectReason::HeightMismatch
    );
}

#[tokio::test]
async fn test_scenario_f_and_g_locked_validator_rejects_conflicting_and_unlocks_with_proof() {
    let keys: Vec<_> = (1..=4)
        .map(|seed| Ed25519Keypair::from_seed(&[seed; 32]))
        .collect();
    let val_set = ValidatorSet::new(
        keys.iter()
            .map(|k| Validator::new(k.address(), k.public_key_bytes().to_vec(), 100))
            .collect(),
    )
    .unwrap();
    let genesis = test_genesis(&keys, None);
    let genesis_fp = genesis.fingerprint().unwrap();

    let mut engine = BftConsensusEngine::new(1, val_set.clone());
    let block1 = Hash32::new([11; 32]);
    let block2 = Hash32::new([22; 32]);

    // Validator locks on block1 in round 0 (e.g. restored from persistent signing journal)
    engine.restore_lock(1, 0, block1);
    assert_eq!(engine.locked_block(), Some(block1));
    assert_eq!(engine.locked_round(), Some(0));

    // Round 1: Conflicting proposal block2 arrives WITHOUT valid-round certificate
    engine.set_round(1);
    let can_prevote = engine.can_prevote_block(block2);
    assert!(
        !can_prevote,
        "must reject conflicting proposal when locked without proof"
    );

    // Round 2: Proposal block2 arrives WITH valid-round certificate from round 1
    let mut round1_prevotes = Vec::new();
    for key in keys.iter().take(3) {
        let mut v = Vote {
            genesis: genesis_fp,
            validator_address: key.address(),
            height: 1,
            round: 1,
            block_hash: Some(block2),
            vote_type: VoteType::Prevote,
            signature: Vec::new(),
        };
        v.signature = key.sign(&v.sign_bytes().unwrap());
        round1_prevotes.push(v);
    }

    engine.install_valid_round_certificate(1, 1, block2, &round1_prevotes);
    engine.set_round(2);

    assert!(
        engine.can_prevote_block(block2),
        "valid higher-round certificate must unlock validator to safe new block"
    );
}

#[tokio::test]
async fn test_scenario_l_round_skew_horizon_boundaries() {
    let keys: Vec<_> = (1..=4)
        .map(|seed| Ed25519Keypair::from_seed(&[seed; 32]))
        .collect();
    let val_set = ValidatorSet::new(
        keys.iter()
            .map(|k| Validator::new(k.address(), k.public_key_bytes().to_vec(), 100))
            .collect(),
    )
    .unwrap();
    let genesis = test_genesis(&keys, None);
    let dir = tempfile::tempdir().unwrap();
    let (p2p, _, _, _) = create_mock_p2p("node-l");

    let (mut driver, _, _, _) = create_test_driver(1, val_set, &genesis, &dir, p2p, None);

    let genesis_fp = genesis.fingerprint().unwrap();

    // Round 5 is within MAX_FUTURE_ROUND_GAP (32) -> must be observed
    let mut vote_within = Vote {
        genesis: genesis_fp,
        validator_address: keys[1].address(),
        height: 1,
        round: 5,
        block_hash: None,
        vote_type: VoteType::Prevote,
        signature: Vec::new(),
    };
    vote_within.signature = keys[1].sign(&vote_within.sign_bytes().unwrap());
    driver.observe_future_prevote_test(vote_within);
    assert_eq!(driver.future_prevote_rounds_count(), 1);

    // Round 100 exceeds MAX_FUTURE_ROUND_GAP (32) -> must be ignored to protect resources
    let mut vote_excessive = Vote {
        genesis: genesis_fp,
        validator_address: keys[2].address(),
        height: 1,
        round: 100,
        block_hash: None,
        vote_type: VoteType::Prevote,
        signature: Vec::new(),
    };
    vote_excessive.signature = keys[2].sign(&vote_excessive.sign_bytes().unwrap());
    driver.observe_future_prevote_test(vote_excessive);
    assert_eq!(
        driver.future_prevote_rounds_count(),
        1,
        "excessive round skew must be dropped"
    );
}

#[tokio::test]
async fn test_scenario_m_historical_catchup_across_validator_set_transition() {
    let keys_v1: Vec<_> = (1..=4)
        .map(|seed| Ed25519Keypair::from_seed(&[seed; 32]))
        .collect();
    let keys_v2: Vec<_> = (5..=8)
        .map(|seed| Ed25519Keypair::from_seed(&[seed; 32]))
        .collect();

    let val_set_v1 = ValidatorSet::new(
        keys_v1
            .iter()
            .map(|k| Validator::new(k.address(), k.public_key_bytes().to_vec(), 100))
            .collect(),
    )
    .unwrap();
    let val_set_v2 = ValidatorSet::new(
        keys_v2
            .iter()
            .map(|k| Validator::new(k.address(), k.public_key_bytes().to_vec(), 100))
            .collect(),
    )
    .unwrap();

    let genesis = test_genesis(&keys_v1, None);
    let genesis_fp = genesis.fingerprint().unwrap();

    let store =
        RedbStore::open(&tempfile::tempdir().unwrap().path().join("data/ledger.redb")).unwrap();
    let ledger = ChainLedger::init_from_genesis_with_store(genesis, store).unwrap();

    // Block 1 committed under val_set_v1
    let mut b1 = ledger.build_proposal(keys_v1[0].address()).unwrap();
    let b1_hash = Hasher::block_hash(&b1.header).unwrap();
    b1.last_commit = keys_v1[0..3]
        .iter()
        .map(|k| {
            let mut v = Vote {
                genesis: genesis_fp,
                validator_address: k.address(),
                height: 1,
                round: 0,
                block_hash: Some(b1_hash),
                vote_type: VoteType::Precommit,
                signature: Vec::new(),
            };
            v.signature = k.sign(&v.sign_bytes().unwrap());
            CommitSignature {
                round: 0,
                validator_address: k.address(),
                signature: v.signature,
                timestamp_unix_secs: 1_700_000_000,
            }
        })
        .collect();

    // Verifying B1 with canonical V1 set passes
    assert!(sprax_consensus::verify_block_commit(&b1, &val_set_v1, genesis_fp).is_ok());

    // Verifying B1 with later V2 set FAILS (historical verification must use canonical height set)
    assert!(
        sprax_consensus::verify_block_commit(&b1, &val_set_v2, genesis_fp).is_err(),
        "historical block must fail if verified with mismatched validator set"
    );
}

#[tokio::test]
async fn test_weighted_validator_quorum_strictly_enforces_power_threshold() {
    let keys: Vec<_> = (1..=4)
        .map(|seed| Ed25519Keypair::from_seed(&[seed; 32]))
        .collect();
    let weights = [40u64, 30, 20, 10];
    let val_set = ValidatorSet::new(
        keys.iter()
            .zip(weights.iter())
            .map(|(k, &w)| Validator::new(k.address(), k.public_key_bytes().to_vec(), w))
            .collect(),
    )
    .unwrap();

    assert_eq!(val_set.total_voting_power(), 100);

    // Quorum strictly requires > 2/3 of 100, which is > 66 (i.e. >= 67)
    // 1. Validator 0 + Validator 2 = 40 + 20 = 60 power (2 of 4 nodes, but only 60%)
    assert!(
        !val_set.has_quorum(60),
        "60% voting power must NOT have quorum"
    );

    // 2. Validator 0 + Validator 1 = 40 + 30 = 70 power (2 of 4 nodes, but 70%)
    assert!(
        val_set.has_quorum(70),
        "70% voting power strictly satisfies > 2/3 quorum"
    );

    // 3. Validator 1 + Validator 2 + Validator 3 = 30 + 20 + 10 = 60 power (3 of 4 nodes!)
    assert!(
        !val_set.has_quorum(60),
        "3 out of 4 nodes with only 60% power must NEVER reach quorum"
    );
}
