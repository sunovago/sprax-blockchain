use sprax_consensus::{
    engine::BftConsensusEngine,
    evidence::EquivocationEvidence,
    round::RoundStep,
    staking::{
        CommissionRates, StakingKeeper, StakingParams, ValidatorDescription, ValidatorStatus,
    },
    validator::{Validator, ValidatorSet},
    vote::{Vote, VoteType},
};
use sprax_crypto::Ed25519Keypair;
use sprax_types::{Address, Amount, Hash32};

#[test]
fn test_scenario_1_honest_validators_bft_round_convergence() {
    let kp1 = Ed25519Keypair::from_seed(&[1u8; 32]);
    let kp2 = Ed25519Keypair::from_seed(&[2u8; 32]);
    let kp3 = Ed25519Keypair::from_seed(&[3u8; 32]);

    let v1 = Validator::new(kp1.address(), kp1.public_key_bytes().to_vec(), 40);
    let v2 = Validator::new(kp2.address(), kp2.public_key_bytes().to_vec(), 35);
    let v3 = Validator::new(kp3.address(), kp3.public_key_bytes().to_vec(), 25);

    let val_set = ValidatorSet::new(vec![v1.clone(), v2.clone(), v3.clone()]).unwrap();
    let mut engine = BftConsensusEngine::new(1, val_set);

    engine.start_height(1);
    assert_eq!(engine.current_step(), RoundStep::Propose);

    let proposer = engine.select_proposer();
    let proposal_hash = Hash32::new([0xbb; 32]);
    engine
        .propose_block(proposal_hash, proposer.address)
        .unwrap();
    assert_eq!(engine.current_step(), RoundStep::Prevote);

    // Honest validators 1 & 2 prevote (40 + 35 = 75 >= 67 quorum)
    let pv1 = Vote::new(
        VoteType::Prevote,
        1,
        0,
        Some(proposal_hash),
        v1.address,
        kp1.sign(proposal_hash.as_bytes()),
    );
    let pv2 = Vote::new(
        VoteType::Prevote,
        1,
        0,
        Some(proposal_hash),
        v2.address,
        kp2.sign(proposal_hash.as_bytes()),
    );

    assert!(!engine.receive_prevote(pv1).unwrap());
    assert!(engine.receive_prevote(pv2).unwrap());
    assert_eq!(engine.current_step(), RoundStep::Precommit);

    // Honest validators 1 & 2 precommit
    let pc1 = Vote::new(
        VoteType::Precommit,
        1,
        0,
        Some(proposal_hash),
        v1.address,
        kp1.sign(proposal_hash.as_bytes()),
    );
    let pc2 = Vote::new(
        VoteType::Precommit,
        1,
        0,
        Some(proposal_hash),
        v2.address,
        kp2.sign(proposal_hash.as_bytes()),
    );

    assert!(engine.receive_precommit(pc1).unwrap().is_none());
    let sigs = engine
        .receive_precommit(pc2)
        .unwrap()
        .expect("block should be committed");

    assert_eq!(engine.current_step(), RoundStep::Commit);
    assert_eq!(sigs.len(), 2);
}

#[test]
fn test_scenario_2_offline_validators_below_and_above_threshold() {
    let kp1 = Ed25519Keypair::from_seed(&[1u8; 32]);
    let kp2 = Ed25519Keypair::from_seed(&[2u8; 32]);
    let kp3 = Ed25519Keypair::from_seed(&[3u8; 32]);

    let v1 = Validator::new(kp1.address(), kp1.public_key_bytes().to_vec(), 50);
    let v2 = Validator::new(kp2.address(), kp2.public_key_bytes().to_vec(), 30);
    let v3 = Validator::new(kp3.address(), kp3.public_key_bytes().to_vec(), 20); // 20% power

    let val_set = ValidatorSet::new(vec![v1.clone(), v2.clone(), v3.clone()]).unwrap();
    let mut engine = BftConsensusEngine::new(1, val_set);

    engine.start_height(1);
    let proposal_hash = Hash32::new([0xcc; 32]);
    engine.propose_block(proposal_hash, v1.address).unwrap();

    // CASE A: Validator 3 (20% power < 33.3%) is offline.
    // Validators 1 & 2 (50 + 30 = 80% power >= 67% quorum) can finalize!
    let pv1 = Vote::new(
        VoteType::Prevote,
        1,
        0,
        Some(proposal_hash),
        v1.address,
        kp1.sign(proposal_hash.as_bytes()),
    );
    let pv2 = Vote::new(
        VoteType::Prevote,
        1,
        0,
        Some(proposal_hash),
        v2.address,
        kp2.sign(proposal_hash.as_bytes()),
    );
    engine.receive_prevote(pv1).unwrap();
    engine.receive_prevote(pv2).unwrap();

    let pc1 = Vote::new(
        VoteType::Precommit,
        1,
        0,
        Some(proposal_hash),
        v1.address,
        kp1.sign(proposal_hash.as_bytes()),
    );
    let pc2 = Vote::new(
        VoteType::Precommit,
        1,
        0,
        Some(proposal_hash),
        v2.address,
        kp2.sign(proposal_hash.as_bytes()),
    );
    engine.receive_precommit(pc1).unwrap();
    let commit_res = engine.receive_precommit(pc2).unwrap();
    assert!(
        commit_res.is_some(),
        "consensus must proceed with < 1/3 offline validators"
    );

    // CASE B: Validator 1 (50% power >= 33.4%) is offline in next height.
    // Validators 2 & 3 (30 + 20 = 50% power < 67% quorum) CANNOT finalize.
    engine.start_height(2);
    let proposal2 = Hash32::new([0xdd; 32]);
    engine.propose_block(proposal2, v2.address).unwrap();

    let pv2_b = Vote::new(
        VoteType::Prevote,
        2,
        0,
        Some(proposal2),
        v2.address,
        kp2.sign(proposal2.as_bytes()),
    );
    let pv3_b = Vote::new(
        VoteType::Prevote,
        2,
        0,
        Some(proposal2),
        v3.address,
        kp3.sign(proposal2.as_bytes()),
    );

    assert!(!engine.receive_prevote(pv2_b).unwrap());
    let has_quorum = engine.receive_prevote(pv3_b).unwrap();
    assert!(
        !has_quorum,
        "consensus must halt and wait for quorum when >= 1/3 is offline"
    );
    assert_eq!(engine.current_step(), RoundStep::Prevote);
}

#[test]
fn test_scenario_3_network_partition_majority_vs_minority() {
    let kp1 = Ed25519Keypair::from_seed(&[1u8; 32]);
    let kp2 = Ed25519Keypair::from_seed(&[2u8; 32]);
    let kp3 = Ed25519Keypair::from_seed(&[3u8; 32]);
    let kp4 = Ed25519Keypair::from_seed(&[4u8; 32]);

    // Total: 100 power. Quorum: 67 power.
    let v1 = Validator::new(kp1.address(), kp1.public_key_bytes().to_vec(), 40);
    let v2 = Validator::new(kp2.address(), kp2.public_key_bytes().to_vec(), 30);
    let v3 = Validator::new(kp3.address(), kp3.public_key_bytes().to_vec(), 15);
    let v4 = Validator::new(kp4.address(), kp4.public_key_bytes().to_vec(), 15);

    let val_set = ValidatorSet::new(vec![v1.clone(), v2.clone(), v3.clone(), v4.clone()]).unwrap();

    // Partition A: [v1, v2] -> 70% power (Majority >= 67%)
    let mut engine_a = BftConsensusEngine::new(1, val_set.clone());
    engine_a.start_height(1);
    let block_a = Hash32::new([0x11; 32]);
    engine_a.propose_block(block_a, v1.address).unwrap();

    engine_a
        .receive_prevote(Vote::new(
            VoteType::Prevote,
            1,
            0,
            Some(block_a),
            v1.address,
            kp1.sign(block_a.as_bytes()),
        ))
        .unwrap();
    engine_a
        .receive_prevote(Vote::new(
            VoteType::Prevote,
            1,
            0,
            Some(block_a),
            v2.address,
            kp2.sign(block_a.as_bytes()),
        ))
        .unwrap();

    engine_a
        .receive_precommit(Vote::new(
            VoteType::Precommit,
            1,
            0,
            Some(block_a),
            v1.address,
            kp1.sign(block_a.as_bytes()),
        ))
        .unwrap();
    let res_a = engine_a
        .receive_precommit(Vote::new(
            VoteType::Precommit,
            1,
            0,
            Some(block_a),
            v2.address,
            kp2.sign(block_a.as_bytes()),
        ))
        .unwrap();
    assert!(
        res_a.is_some(),
        "majority partition with >2/3 stake must finalize"
    );

    // Partition B: [v3, v4] -> 30% power (Minority < 67%)
    let mut engine_b = BftConsensusEngine::new(1, val_set);
    engine_b.start_height(1);
    let block_b = Hash32::new([0x22; 32]);
    engine_b.propose_block(block_b, v3.address).unwrap();

    engine_b
        .receive_prevote(Vote::new(
            VoteType::Prevote,
            1,
            0,
            Some(block_b),
            v3.address,
            kp3.sign(block_b.as_bytes()),
        ))
        .unwrap();
    let res_pv_b = engine_b
        .receive_prevote(Vote::new(
            VoteType::Prevote,
            1,
            0,
            Some(block_b),
            v4.address,
            kp4.sign(block_b.as_bytes()),
        ))
        .unwrap();
    assert!(
        !res_pv_b,
        "minority partition must never reach prevote quorum"
    );
}

#[test]
fn test_scenario_4_invalid_block_rejection() {
    let kp1 = Ed25519Keypair::from_seed(&[1u8; 32]);
    let v1 = Validator::new(kp1.address(), kp1.public_key_bytes().to_vec(), 100);

    let val_set = ValidatorSet::new(vec![v1.clone()]).unwrap();
    let mut engine = BftConsensusEngine::new(1, val_set);
    engine.start_height(1);

    // Non-active validator attempts to propose block
    let attacker = Address::new([0x99; 20]);
    let res = engine.propose_block(Hash32::new([1u8; 32]), attacker);
    assert!(
        res.is_err(),
        "proposal from unauthorized validator must be rejected"
    );
}

#[test]
fn test_scenario_5_staking_delegation_unbonding_lifecycle() {
    let mut keeper = StakingKeeper::default();
    let val_addr = Address::new([1u8; 20]);
    let del_addr = Address::new([2u8; 20]);

    // 1. Validator Register
    let self_stake = Amount::from_sprx_whole(10_000).unwrap();
    keeper
        .register_validator(
            val_addr,
            vec![1; 32],
            ValidatorDescription {
                moniker: "Node-Alpha".into(),
                identity: "".into(),
                website: "".into(),
                details: "".into(),
            },
            self_stake,
            CommissionRates::default(),
        )
        .unwrap();

    // 2. Delegator Delegates 5,000 SPRX
    let del_amount = Amount::from_sprx_whole(5_000).unwrap();
    keeper.delegate(del_addr, val_addr, del_amount).unwrap();

    let val = keeper.get_validator(&val_addr).unwrap();
    assert_eq!(val.tokens, Amount::from_sprx_whole(15_000).unwrap());

    // 3. Delegator Undelegates 2,000 SPRX at height 100
    let unbonding = keeper
        .undelegate(
            del_addr,
            val_addr,
            Amount::from_sprx_whole(2_000).unwrap(),
            100,
        )
        .unwrap();
    assert_eq!(
        unbonding.completion_height,
        100 + keeper.params().unbonding_period_blocks
    );

    let val_after = keeper.get_validator(&val_addr).unwrap();
    assert_eq!(val_after.tokens, Amount::from_sprx_whole(13_000).unwrap());

    let del_after = keeper.get_delegation(&del_addr, &val_addr).unwrap();
    assert_eq!(del_after.balance, Amount::from_sprx_whole(3_000).unwrap());
}

#[test]
fn test_scenario_6_downtime_slashing_and_unjail() {
    let params = StakingParams {
        missed_blocks_threshold: 3,
        downtime_jail_blocks: 10,
        ..StakingParams::default()
    };
    let mut keeper = StakingKeeper::new(params);

    let val_addr = Address::new([1u8; 20]);
    keeper
        .register_validator(
            val_addr,
            vec![1; 32],
            ValidatorDescription {
                moniker: "Node-Beta".into(),
                identity: "".into(),
                website: "".into(),
                details: "".into(),
            },
            Amount::from_sprx_whole(100_000).unwrap(),
            CommissionRates::default(),
        )
        .unwrap();

    // Miss block 1 and 2 (below threshold)
    assert_eq!(keeper.slash_downtime(&val_addr, 1).unwrap(), Amount::ZERO);
    assert_eq!(keeper.slash_downtime(&val_addr, 2).unwrap(), Amount::ZERO);
    assert_eq!(
        keeper.get_validator(&val_addr).unwrap().status,
        ValidatorStatus::Active
    );

    // Miss block 3 (hits threshold) -> Slashed 0.01% & Jailed!
    let slashed = keeper.slash_downtime(&val_addr, 3).unwrap();
    assert_ne!(slashed, Amount::ZERO);
    assert_eq!(
        keeper.get_validator(&val_addr).unwrap().status,
        ValidatorStatus::Jailed
    );

    // Attempt to unjail early at block 5 -> Fails
    assert!(keeper.unjail(&val_addr, 5).is_err());

    // Unjail at block 15 (after jail period of 10 blocks) -> Succeeds
    keeper.unjail(&val_addr, 15).unwrap();
    assert_eq!(
        keeper.get_validator(&val_addr).unwrap().status,
        ValidatorStatus::Active
    );
}

#[test]
fn test_scenario_7_equivocation_double_signing_tombstone() {
    let mut keeper = StakingKeeper::default();
    let val_addr = Address::new([1u8; 20]);

    keeper
        .register_validator(
            val_addr,
            vec![1; 32],
            ValidatorDescription {
                moniker: "Node-Gamma".into(),
                identity: "".into(),
                website: "".into(),
                details: "".into(),
            },
            Amount::from_sprx_whole(50_000).unwrap(),
            CommissionRates::default(),
        )
        .unwrap();

    let evidence = EquivocationEvidence {
        validator_address: val_addr,
        height: 10,
        round: 0,
        vote_a: Vote::new(
            VoteType::Prevote,
            10,
            0,
            Some(Hash32::new([1u8; 32])),
            val_addr,
            vec![1; 64],
        ),
        vote_b: Vote::new(
            VoteType::Prevote,
            10,
            0,
            Some(Hash32::new([2u8; 32])),
            val_addr,
            vec![2; 64],
        ),
    };

    let slashed = keeper.slash_equivocation(&evidence).unwrap();
    assert_eq!(slashed, Amount::from_sprx_whole(2_500).unwrap()); // 5% of 50,000

    let val = keeper.get_validator(&val_addr).unwrap();
    assert!(val.is_tombstoned);
    assert_eq!(val.status, ValidatorStatus::Jailed);

    // Tombstoned validator can NEVER be unjailed
    assert!(keeper.unjail(&val_addr, 1000).is_err());
}
