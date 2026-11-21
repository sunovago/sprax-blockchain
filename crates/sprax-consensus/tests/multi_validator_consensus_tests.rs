use sprax_consensus::{
    persistence::ConsensusSnapshot,
    reward::{RewardKeeper, RewardParams},
    round::RoundStep,
    staking::{CommissionRates, StakingKeeper, ValidatorDescription},
    validator::{Validator, ValidatorSet},
    vote::{Vote, VoteType},
    BftConsensusEngine,
};
use sprax_crypto::Ed25519Keypair;
use sprax_types::{Amount, Hash32};

/// Helper to generate N validator keypairs and associated Validators.
fn generate_validators(powers: &[u64]) -> (Vec<Ed25519Keypair>, Vec<Validator>, ValidatorSet) {
    let mut keypairs = Vec::new();
    let mut validators = Vec::new();

    for (i, &power) in powers.iter().enumerate() {
        let mut seed = [0u8; 32];
        seed[0] = (i + 1) as u8;
        seed[1] = 0xAA;
        let kp = Ed25519Keypair::from_seed(&seed);
        let val = Validator::new(kp.address(), kp.public_key_bytes().to_vec(), power);
        validators.push(val);
        keypairs.push(kp);
    }

    let val_set = ValidatorSet::new(validators.clone()).expect("valid validator set");
    (keypairs, validators, val_set)
}

fn create_signed_vote(
    kp: &Ed25519Keypair,
    genesis: Hash32,
    vote_type: VoteType,
    height: u64,
    round: u32,
    block_hash: Option<Hash32>,
) -> Vote {
    let mut vote = Vote::new(
        genesis,
        vote_type,
        height,
        round,
        block_hash,
        kp.address(),
        Vec::new(),
    );
    let sign_bytes = vote.sign_bytes().expect("canonical sign bytes");
    let sig = kp.sign(&sign_bytes);
    vote.signature = sig;
    vote
}

#[test]
fn test_1_validator_consensus_convergence() {
    let (keypairs, _, val_set) = generate_validators(&[100]);
    let genesis = Hash32::new([0x11; 32]);
    let mut engine = BftConsensusEngine::new(1, val_set);

    engine.start_height(1);
    assert_eq!(engine.current_step(), RoundStep::Propose);

    let proposer = engine.select_proposer(genesis, 1, 0).unwrap();
    assert_eq!(proposer.address, keypairs[0].address());

    let block_hash = Hash32::new([0xBA; 32]);
    engine.propose_block(block_hash, proposer.address).unwrap();
    assert_eq!(engine.current_step(), RoundStep::Prevote);

    let pv = create_signed_vote(
        &keypairs[0],
        genesis,
        VoteType::Prevote,
        1,
        0,
        Some(block_hash),
    );
    assert!(engine.receive_prevote(pv).unwrap());
    assert_eq!(engine.current_step(), RoundStep::Precommit);

    let pc = create_signed_vote(
        &keypairs[0],
        genesis,
        VoteType::Precommit,
        1,
        0,
        Some(block_hash),
    );
    let sigs = engine
        .receive_precommit(pc)
        .unwrap()
        .expect("1-val quorum reached");
    assert_eq!(engine.current_step(), RoundStep::Commit);
    assert_eq!(sigs.len(), 1);
}

#[test]
fn test_3_validator_consensus_all_required_for_equal_powers() {
    // Powers: 1, 1, 1. Total = 3. Quorum = floor(2*3/3) + 1 = 2 + 1 = 3.
    // In a 3-node equal power setup, all 3 are required to strictly exceed 2/3.
    let (keypairs, _, val_set) = generate_validators(&[1, 1, 1]);
    let genesis = Hash32::new([0x22; 32]);
    let mut engine = BftConsensusEngine::new(1, val_set);

    engine.start_height(1);
    let proposer = engine.select_proposer(genesis, 1, 0).unwrap();
    let block_hash = Hash32::new([0xBB; 32]);
    engine.propose_block(block_hash, proposer.address).unwrap();

    // Node 0 prevotes
    let pv0 = create_signed_vote(
        &keypairs[0],
        genesis,
        VoteType::Prevote,
        1,
        0,
        Some(block_hash),
    );
    assert!(!engine.receive_prevote(pv0).unwrap());

    // Node 1 prevotes (total power 2 < 3)
    let pv1 = create_signed_vote(
        &keypairs[1],
        genesis,
        VoteType::Prevote,
        1,
        0,
        Some(block_hash),
    );
    assert!(!engine.receive_prevote(pv1).unwrap());

    // Node 2 prevotes (total power 3 >= 3 -> quorum!)
    let pv2 = create_signed_vote(
        &keypairs[2],
        genesis,
        VoteType::Prevote,
        1,
        0,
        Some(block_hash),
    );
    assert!(engine.receive_prevote(pv2).unwrap());
    assert_eq!(engine.current_step(), RoundStep::Precommit);

    // Precommits: first 2 do not commit
    let pc0 = create_signed_vote(
        &keypairs[0],
        genesis,
        VoteType::Precommit,
        1,
        0,
        Some(block_hash),
    );
    assert!(engine.receive_precommit(pc0).unwrap().is_none());

    let pc1 = create_signed_vote(
        &keypairs[1],
        genesis,
        VoteType::Precommit,
        1,
        0,
        Some(block_hash),
    );
    assert!(engine.receive_precommit(pc1).unwrap().is_none());

    // 3rd precommit finalizes block
    let pc2 = create_signed_vote(
        &keypairs[2],
        genesis,
        VoteType::Precommit,
        1,
        0,
        Some(block_hash),
    );
    let commit_sigs = engine
        .receive_precommit(pc2)
        .unwrap()
        .expect("block committed");
    assert_eq!(commit_sigs.len(), 3);
}

#[test]
fn test_4_validator_consensus_bft_tolerates_1_byzantine() {
    // 4 nodes with power 1 each (total 4, quorum 3). f = 1.
    // Node 3 is byzantine/offline. Nodes 0, 1, 2 are honest and finalize.
    let (keypairs, _, val_set) = generate_validators(&[1, 1, 1, 1]);
    let genesis = Hash32::new([0x33; 32]);
    let mut engine = BftConsensusEngine::new(1, val_set);

    engine.start_height(1);
    let proposer = engine.select_proposer(genesis, 1, 0).unwrap();
    let block_hash = Hash32::new([0xCC; 32]);
    engine.propose_block(block_hash, proposer.address).unwrap();

    // Nodes 0, 1, 2 prevote
    for i in 0..2 {
        let pv = create_signed_vote(
            &keypairs[i],
            genesis,
            VoteType::Prevote,
            1,
            0,
            Some(block_hash),
        );
        assert!(!engine.receive_prevote(pv).unwrap());
    }
    let pv2 = create_signed_vote(
        &keypairs[2],
        genesis,
        VoteType::Prevote,
        1,
        0,
        Some(block_hash),
    );
    assert!(engine.receive_prevote(pv2).unwrap());
    assert_eq!(engine.current_step(), RoundStep::Precommit);

    // Nodes 0, 1, 2 precommit
    for i in 0..2 {
        let pc = create_signed_vote(
            &keypairs[i],
            genesis,
            VoteType::Precommit,
            1,
            0,
            Some(block_hash),
        );
        assert!(engine.receive_precommit(pc).unwrap().is_none());
    }
    let pc2 = create_signed_vote(
        &keypairs[2],
        genesis,
        VoteType::Precommit,
        1,
        0,
        Some(block_hash),
    );
    let commit_sigs = engine
        .receive_precommit(pc2)
        .unwrap()
        .expect("block committed");
    assert_eq!(commit_sigs.len(), 3);
}

#[test]
fn test_7_validator_consensus_bft_tolerates_2_byzantine() {
    // 7 nodes with power 1 each (total 7, quorum 5). f = 2.
    // Nodes 5 and 6 are byzantine/offline. Nodes 0..4 (5 nodes) finalize block.
    let (keypairs, _, val_set) = generate_validators(&[1, 1, 1, 1, 1, 1, 1]);
    let genesis = Hash32::new([0x44; 32]);
    let mut engine = BftConsensusEngine::new(1, val_set);

    engine.start_height(1);
    let proposer = engine.select_proposer(genesis, 1, 0).unwrap();
    let block_hash = Hash32::new([0xDD; 32]);
    engine.propose_block(block_hash, proposer.address).unwrap();

    // First 4 prevotes do not reach quorum (4 < 5)
    for i in 0..4 {
        let pv = create_signed_vote(
            &keypairs[i],
            genesis,
            VoteType::Prevote,
            1,
            0,
            Some(block_hash),
        );
        assert!(!engine.receive_prevote(pv).unwrap());
    }
    // 5th prevote achieves quorum
    let pv4 = create_signed_vote(
        &keypairs[4],
        genesis,
        VoteType::Prevote,
        1,
        0,
        Some(block_hash),
    );
    assert!(engine.receive_prevote(pv4).unwrap());
    assert_eq!(engine.current_step(), RoundStep::Precommit);

    // Precommits: 5th achieves commit
    for i in 0..4 {
        let pc = create_signed_vote(
            &keypairs[i],
            genesis,
            VoteType::Precommit,
            1,
            0,
            Some(block_hash),
        );
        assert!(engine.receive_precommit(pc).unwrap().is_none());
    }
    let pc4 = create_signed_vote(
        &keypairs[4],
        genesis,
        VoteType::Precommit,
        1,
        0,
        Some(block_hash),
    );
    let commit_sigs = engine
        .receive_precommit(pc4)
        .unwrap()
        .expect("block committed");
    assert_eq!(commit_sigs.len(), 5);
}

#[test]
fn test_10_validator_heterogeneous_voting_power_distribution() {
    // 10 validators with heterogeneous voting powers: [25, 20, 15, 12, 8, 6, 5, 4, 3, 2] = 100
    // Quorum = floor(200/3) + 1 = 67.
    let powers = [25, 20, 15, 12, 8, 6, 5, 4, 3, 2];
    let (keypairs, _, val_set) = generate_validators(&powers);
    let genesis = Hash32::new([0x55; 32]);
    let mut engine = BftConsensusEngine::new(1, val_set);

    engine.start_height(1);
    let proposer = engine.select_proposer(genesis, 1, 0).unwrap();
    let block_hash = Hash32::new([0xEE; 32]);
    engine.propose_block(block_hash, proposer.address).unwrap();

    // Nodes 0 (25), 1 (20), 2 (15) vote: 25+20+15 = 60 < 67 -> no quorum
    for i in 0..3 {
        let pv = create_signed_vote(
            &keypairs[i],
            genesis,
            VoteType::Prevote,
            1,
            0,
            Some(block_hash),
        );
        assert!(!engine.receive_prevote(pv).unwrap());
    }

    // Node 3 (12) votes: 60 + 12 = 72 >= 67 -> quorum!
    let pv3 = create_signed_vote(
        &keypairs[3],
        genesis,
        VoteType::Prevote,
        1,
        0,
        Some(block_hash),
    );
    assert!(engine.receive_prevote(pv3).unwrap());
    assert_eq!(engine.current_step(), RoundStep::Precommit);

    // Precommits from nodes 0, 1, 2 (60 power)
    for i in 0..3 {
        let pc = create_signed_vote(
            &keypairs[i],
            genesis,
            VoteType::Precommit,
            1,
            0,
            Some(block_hash),
        );
        assert!(engine.receive_precommit(pc).unwrap().is_none());
    }
    // Node 3 precommits -> commit!
    let pc3 = create_signed_vote(
        &keypairs[3],
        genesis,
        VoteType::Precommit,
        1,
        0,
        Some(block_hash),
    );
    let commit_sigs = engine
        .receive_precommit(pc3)
        .unwrap()
        .expect("block committed");
    assert_eq!(commit_sigs.len(), 4);
}

#[test]
fn test_byzantine_conflicting_proposals_round_advancement() {
    // 4 validators with power 1 each (quorum 3).
    let (keypairs, _, val_set) = generate_validators(&[1, 1, 1, 1]);
    let genesis = Hash32::new([0x66; 32]);
    let mut engine = BftConsensusEngine::new(1, val_set);

    engine.start_height(1);
    // Malicious proposer creates block A
    let block_a = Hash32::new([0x0A; 32]);
    let block_b = Hash32::new([0x0B; 32]);

    let proposer_r0 = engine.select_proposer(genesis, 1, 0).unwrap();
    engine.propose_block(block_a, proposer_r0.address).unwrap();

    // Node 0 prevotes block A, Node 1 prevotes block B (conflict / split)
    let pv0 = create_signed_vote(
        &keypairs[0],
        genesis,
        VoteType::Prevote,
        1,
        0,
        Some(block_a),
    );
    let pv1 = create_signed_vote(
        &keypairs[1],
        genesis,
        VoteType::Prevote,
        1,
        0,
        Some(block_b),
    );
    let pv2_nil = create_signed_vote(&keypairs[2], genesis, VoteType::Prevote, 1, 0, None);

    assert!(!engine.receive_prevote(pv0).unwrap());
    assert!(!engine.receive_prevote(pv1).unwrap());
    assert!(!engine.receive_prevote(pv2_nil).unwrap());

    // Round 0 times out without quorum. Engine advances to Round 1.
    engine.set_round(1);
    assert_eq!(engine.current_round(), 1);
    assert_eq!(engine.current_step(), RoundStep::Propose);

    // In Round 1, honest proposer proposes block C
    let proposer_r1 = engine.select_proposer(genesis, 1, 1).unwrap();
    let block_c = Hash32::new([0x0C; 32]);
    engine.propose_block(block_c, proposer_r1.address).unwrap();

    // 3 nodes prevote block C in Round 1
    for i in 0..3 {
        let pv = create_signed_vote(
            &keypairs[i],
            genesis,
            VoteType::Prevote,
            1,
            1,
            Some(block_c),
        );
        let reached = engine.receive_prevote(pv).unwrap();
        if i == 2 {
            assert!(reached);
        } else {
            assert!(!reached);
        }
    }
    assert_eq!(engine.current_step(), RoundStep::Precommit);

    // 3 nodes precommit block C in Round 1
    for i in 0..2 {
        let pc = create_signed_vote(
            &keypairs[i],
            genesis,
            VoteType::Precommit,
            1,
            1,
            Some(block_c),
        );
        assert!(engine.receive_precommit(pc).unwrap().is_none());
    }
    let pc2 = create_signed_vote(
        &keypairs[2],
        genesis,
        VoteType::Precommit,
        1,
        1,
        Some(block_c),
    );
    let commit_sigs = engine
        .receive_precommit(pc2)
        .unwrap()
        .expect("committed at round 1");
    assert_eq!(commit_sigs.len(), 3);
}

#[test]
fn test_consensus_snapshot_crash_and_recovery() {
    let (keypairs, _, val_set) = generate_validators(&[40, 35, 25]);
    let genesis = Hash32::new([0x77; 32]);
    let mut engine = BftConsensusEngine::new(5, val_set);

    engine.start_height(5);
    let proposer = engine.select_proposer(genesis, 5, 0).unwrap();
    let block_hash = Hash32::new([0x55; 32]);
    engine.propose_block(block_hash, proposer.address).unwrap();

    // Nodes 0 and 1 prevote (40 + 35 = 75 >= 67 quorum)
    let pv0 = create_signed_vote(
        &keypairs[0],
        genesis,
        VoteType::Prevote,
        5,
        0,
        Some(block_hash),
    );
    let pv1 = create_signed_vote(
        &keypairs[1],
        genesis,
        VoteType::Prevote,
        5,
        0,
        Some(block_hash),
    );
    assert!(!engine.receive_prevote(pv0).unwrap());
    assert!(engine.receive_prevote(pv1).unwrap());

    // Record precommit lock
    engine.record_precommit_lock(5, 0, block_hash).unwrap();
    assert_eq!(engine.locked_block(), Some(block_hash));
    assert_eq!(engine.locked_round(), Some(0));

    // Save snapshot to disk before crash
    let tmp = tempfile::tempdir().unwrap();
    let snapshot_file = tmp.path().join("consensus_snapshot.json");

    let snapshot = ConsensusSnapshot::from_engine(&engine);
    snapshot.save_to_file(&snapshot_file).unwrap();

    // SIMULATE CRASH: drop engine and restore from snapshot file
    drop(engine);

    let loaded_snapshot = ConsensusSnapshot::load_from_file(&snapshot_file).unwrap();
    let mut recovered_engine = loaded_snapshot.restore_engine();

    assert_eq!(recovered_engine.current_height(), 5);
    assert_eq!(recovered_engine.current_round(), 0);
    assert_eq!(recovered_engine.locked_block(), Some(block_hash));
    assert_eq!(recovered_engine.locked_round(), Some(0));
    assert_eq!(recovered_engine.current_step(), RoundStep::Precommit);

    // Nodes 0 and 1 precommit on recovered engine
    let pc0 = create_signed_vote(
        &keypairs[0],
        genesis,
        VoteType::Precommit,
        5,
        0,
        Some(block_hash),
    );
    let pc1 = create_signed_vote(
        &keypairs[1],
        genesis,
        VoteType::Precommit,
        5,
        0,
        Some(block_hash),
    );
    assert!(recovered_engine.receive_precommit(pc0).unwrap().is_none());
    let sigs = recovered_engine
        .receive_precommit(pc1)
        .unwrap()
        .expect("block committed");
    assert_eq!(sigs.len(), 2);
}

#[test]
fn test_reward_distribution_multi_validator_and_delegators() {
    let mut staking = StakingKeeper::default();
    let (keypairs, _, _) = generate_validators(&[40, 35, 25]);
    let val0 = keypairs[0].address();
    let val1 = keypairs[1].address();
    let del = keypairs[2].address();

    let desc = ValidatorDescription {
        moniker: "Node".into(),
        identity: "".into(),
        website: "".into(),
        details: "".into(),
    };

    let comm = CommissionRates {
        rate_bps: 500, // 5% commission
        max_rate_bps: 1_000,
        max_change_rate_bps: 100,
    };

    staking
        .register_validator(
            val0,
            keypairs[0].public_key_bytes().to_vec(),
            desc.clone(),
            Amount::from_sprx_whole(10_000).unwrap(),
            comm.clone(),
        )
        .unwrap();
    staking
        .register_validator(
            val1,
            keypairs[1].public_key_bytes().to_vec(),
            desc,
            Amount::from_sprx_whole(10_000).unwrap(),
            comm,
        )
        .unwrap();
    staking
        .delegate(del, val0, Amount::from_sprx_whole(5_000).unwrap())
        .unwrap();

    let mut reward_keeper = RewardKeeper::new(RewardParams::default());
    let distribution = reward_keeper
        .distribute_block_rewards(
            1,
            val0,
            Amount::from_sprx_whole(1).unwrap(),
            &[val0, val1],
            &staking,
        )
        .unwrap();

    assert_eq!(distribution.total_pot, Amount::from_sprx_whole(3).unwrap());
    assert!(distribution.proposer_bonus.as_atto() > 0);
    assert!(reward_keeper.community_pool().as_atto() > 0);
    assert!(reward_keeper.get_operator_commission(&val0).as_atto() > 0);
    assert!(reward_keeper.get_delegator_reward(&del, &val0).as_atto() > 0);

    // Save reward keeper state, crash and reload
    let tmp = tempfile::tempdir().unwrap();
    let reward_file = tmp.path().join("reward_state.json");
    reward_keeper.save_to_file(&reward_file).unwrap();

    let mut reloaded_reward_keeper = RewardKeeper::load_from_file(&reward_file).unwrap();
    assert_eq!(
        reloaded_reward_keeper.community_pool(),
        reward_keeper.community_pool()
    );

    // Claim reward on reloaded keeper
    let claimed = reloaded_reward_keeper
        .claim_delegator_reward(&del, &val0)
        .unwrap();
    assert!(claimed.as_atto() > 0);
    assert_eq!(
        reloaded_reward_keeper.get_delegator_reward(&del, &val0),
        Amount::ZERO
    );
}
