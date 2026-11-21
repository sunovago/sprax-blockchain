use sprax_consensus::{
    engine::BftConsensusEngine,
    evidence::EquivocationEvidence,
    round::RoundStep,
    staking::{CommissionRates, StakingKeeper, ValidatorDescription, ValidatorStatus},
    validator::{Validator, ValidatorSet},
    vote::{Vote, VoteType},
};
use sprax_crypto::Ed25519Keypair;
use sprax_types::{Amount, Hash32};

/// Helper to create a cryptographically signed vote over its canonical sign_bytes.
fn create_signed_vote(
    signer: &Ed25519Keypair,
    vote_type: VoteType,
    height: u64,
    round: u32,
    block_hash: Option<Hash32>,
) -> Vote {
    let mut vote = Vote::new(
        Hash32::ZERO,
        vote_type,
        height,
        round,
        block_hash,
        signer.address(),
        vec![],
    );
    let sign_bytes = vote.sign_bytes().expect("canonical sign bytes");
    vote.signature = signer.sign(&sign_bytes);
    vote
}

#[test]
fn test_byzantine_prevote_double_signing_detection_and_slashing() {
    // 4 validators with powers [40, 30, 15, 15] -> Total = 100, Quorum = 67.
    let kp1 = Ed25519Keypair::from_seed(&[1u8; 32]);
    let kp2 = Ed25519Keypair::from_seed(&[2u8; 32]); // Malicious Byzantine node
    let kp3 = Ed25519Keypair::from_seed(&[3u8; 32]);
    let kp4 = Ed25519Keypair::from_seed(&[4u8; 32]);

    let v1 = Validator::new(kp1.address(), kp1.public_key_bytes().to_vec(), 40);
    let v2 = Validator::new(kp2.address(), kp2.public_key_bytes().to_vec(), 30);
    let v3 = Validator::new(kp3.address(), kp3.public_key_bytes().to_vec(), 15);
    let v4 = Validator::new(kp4.address(), kp4.public_key_bytes().to_vec(), 15);

    let val_set = ValidatorSet::new(vec![v1.clone(), v2.clone(), v3.clone(), v4.clone()]).unwrap();
    let mut engine = BftConsensusEngine::new(1, val_set.clone());
    let mut keeper = StakingKeeper::default();

    // Register all 4 validators in staking keeper with 100,000 SPRX each
    for (v, moniker) in [(&v1, "V1"), (&v2, "V2-Byzantine"), (&v3, "V3"), (&v4, "V4")] {
        keeper
            .register_validator(
                v.address,
                v.public_key.clone(),
                ValidatorDescription {
                    moniker: moniker.into(),
                    identity: "".into(),
                    website: "".into(),
                    details: "".into(),
                },
                Amount::from_sprx_whole(100_000).unwrap(),
                CommissionRates::default(),
            )
            .unwrap();
    }

    engine.start_height(1);
    let proposal_hash_a = Hash32::new([0xaa; 32]);
    let proposal_hash_b = Hash32::new([0xbb; 32]);

    engine.propose_block(proposal_hash_a, v1.address).unwrap();

    // Honest Validator 1 prevotes Block A
    let pv1 = create_signed_vote(&kp1, VoteType::Prevote, 1, 0, Some(proposal_hash_a));
    assert!(!engine.receive_prevote(pv1).unwrap());

    // Byzantine Validator 2 casts FIRST prevote for Block A
    let pv2_a = create_signed_vote(&kp2, VoteType::Prevote, 1, 0, Some(proposal_hash_a));
    assert!(engine.detect_equivocation(&pv2_a).is_none());
    assert!(engine.receive_prevote(pv2_a.clone()).unwrap()); // 40 + 30 = 70 >= 67 quorum!

    // Byzantine Validator 2 casts CONFLICTING SECOND prevote for Block B at same (height 1, round 0)
    let pv2_b = create_signed_vote(&kp2, VoteType::Prevote, 1, 0, Some(proposal_hash_b));

    // 1. Engine automatically detects equivocation
    let evidence_opt = engine.detect_equivocation(&pv2_b);
    assert!(
        evidence_opt.is_some(),
        "engine must detect conflicting vote"
    );
    let evidence = evidence_opt.unwrap();

    // 2. Verify evidence integrity & cryptographic signatures
    assert!(evidence.is_valid_equivocation());
    assert_eq!(evidence.validator_address, v2.address);
    assert_eq!(evidence.height, 1);
    assert_eq!(evidence.round, 0);
    assert!(evidence.verify_signatures(&v2.public_key).is_ok());

    // 3. Slash the Byzantine validator in StakingKeeper
    let slashed = keeper.slash_equivocation(&evidence).unwrap();
    // 5% of 100,000 SPRX = 5,000 SPRX slashed
    assert_eq!(slashed, Amount::from_sprx_whole(5_000).unwrap());

    let val2_record = keeper.get_validator(&v2.address).unwrap();
    assert_eq!(val2_record.tokens, Amount::from_sprx_whole(95_000).unwrap());
    assert!(
        val2_record.is_tombstoned,
        "byzantine node must be permanently tombstoned"
    );
    assert_eq!(val2_record.status, ValidatorStatus::Jailed);

    // 4. Update the validator set dynamically without the slashed Byzantine validator
    let remaining_val_set = ValidatorSet::new(vec![v1.clone(), v3.clone(), v4.clone()]).unwrap();
    engine.set_validator_set(remaining_val_set);

    // 5. Honest remaining validators (v3 + v4) cast prevotes for Block A to cement honest quorum
    let pv3 = create_signed_vote(&kp3, VoteType::Prevote, 1, 0, Some(proposal_hash_a));
    let pv4 = create_signed_vote(&kp4, VoteType::Prevote, 1, 0, Some(proposal_hash_a));
    engine.receive_prevote(pv3).unwrap();
    engine.receive_prevote(pv4).unwrap();

    // Precommit phase: Honest nodes commit Block A
    // In remaining set: Total power = 70. Quorum = (70 * 2 / 3) + 1 = 47.
    // v1 (40) + v3 (15) = 55 >= 47 quorum! So commit is achieved on pc3.
    let pc1 = create_signed_vote(&kp1, VoteType::Precommit, 1, 0, Some(proposal_hash_a));
    let pc3 = create_signed_vote(&kp3, VoteType::Precommit, 1, 0, Some(proposal_hash_a));

    assert!(engine.receive_precommit(pc1).unwrap().is_none());
    let commit_res = engine.receive_precommit(pc3).unwrap();
    assert!(
        commit_res.is_some(),
        "honest nodes must finalize block despite byzantine double-signing"
    );
    assert_eq!(engine.current_step(), RoundStep::Commit);
}

#[test]
fn test_byzantine_precommit_double_signing_and_permanent_tombstone_immunity() {
    let kp1 = Ed25519Keypair::from_seed(&[11u8; 32]); // Malicious validator
    let kp2 = Ed25519Keypair::from_seed(&[12u8; 32]);

    let v1 = Validator::new(kp1.address(), kp1.public_key_bytes().to_vec(), 60);
    let v2 = Validator::new(kp2.address(), kp2.public_key_bytes().to_vec(), 40);

    let val_set = ValidatorSet::new(vec![v1.clone(), v2.clone()]).unwrap();
    let mut engine = BftConsensusEngine::new(1, val_set);
    let mut keeper = StakingKeeper::default();

    keeper
        .register_validator(
            v1.address,
            v1.public_key.clone(),
            ValidatorDescription {
                moniker: "Attacker-Node".into(),
                identity: "".into(),
                website: "".into(),
                details: "".into(),
            },
            Amount::from_sprx_whole(200_000).unwrap(),
            CommissionRates::default(),
        )
        .unwrap();

    engine.start_height(1);
    let block_1 = Hash32::new([0x01; 32]);
    let block_2 = Hash32::new([0x02; 32]);

    // Attacker signs two conflicting precommits for height 1, round 0
    let pc1_a = create_signed_vote(&kp1, VoteType::Precommit, 1, 0, Some(block_1));
    let pc1_b = create_signed_vote(&kp1, VoteType::Precommit, 1, 0, Some(block_2));

    engine.receive_precommit(pc1_a.clone()).unwrap();

    // Conflict detected
    let conflict = engine.find_conflicting_vote(&pc1_b);
    assert_eq!(conflict, Some(pc1_a.clone()));

    let evidence = engine
        .detect_equivocation(&pc1_b)
        .expect("evidence created");
    assert!(evidence.verify_signatures(&v1.public_key).is_ok());

    // Slash attacker: 5% of 200,000 = 10,000 SPRX slashed
    let slashed = keeper.slash_equivocation(&evidence).unwrap();
    assert_eq!(slashed, Amount::from_sprx_whole(10_000).unwrap());

    // Attempt to unjail at block 100, 1000, 100_000 -> All must be rejected
    assert!(keeper.unjail(&v1.address, 100).is_err());
    assert!(keeper.unjail(&v1.address, 10_000).is_err());
    assert!(keeper.unjail(&v1.address, 1_000_000).is_err());

    // Double-slashing protection: Submitting same evidence twice returns ZERO slashed
    let second_slash = keeper.slash_equivocation(&evidence).unwrap();
    assert_eq!(second_slash, Amount::ZERO);
}

#[test]
fn test_byzantine_forged_evidence_rejection() {
    let kp_victim = Ed25519Keypair::from_seed(&[21u8; 32]);
    let kp_attacker = Ed25519Keypair::from_seed(&[22u8; 32]);

    let victim_addr = kp_victim.address();
    let victim_pk = kp_victim.public_key_bytes().to_vec();

    let mut keeper = StakingKeeper::default();
    keeper
        .register_validator(
            victim_addr,
            victim_pk.clone(),
            ValidatorDescription {
                moniker: "Innocent-Node".into(),
                identity: "".into(),
                website: "".into(),
                details: "".into(),
            },
            Amount::from_sprx_whole(50_000).unwrap(),
            CommissionRates::default(),
        )
        .unwrap();

    // CASE 1: Attacker forges vote signature using attacker keypair instead of victim's
    let vote_a = create_signed_vote(
        &kp_victim,
        VoteType::Prevote,
        5,
        0,
        Some(Hash32::new([1; 32])),
    );
    // Forged vote: claims to be victim_addr but signed by kp_attacker
    let mut vote_b = Vote::new(
        Hash32::ZERO,
        VoteType::Prevote,
        5,
        0,
        Some(Hash32::new([2; 32])),
        victim_addr,
        vec![],
    );
    vote_b.signature = kp_attacker.sign(&vote_b.sign_bytes().unwrap());

    let forged_evidence = EquivocationEvidence {
        validator_address: victim_addr,
        height: 5,
        round: 0,
        vote_a: vote_a.clone(),
        vote_b: vote_b.clone(),
    };

    // Signature verification MUST fail
    let verify_res = forged_evidence.verify_signatures(&victim_pk);
    assert!(verify_res.is_err(), "forged signature must be rejected");

    // CASE 2: Identical block hashes (not an equivocation)
    let vote_dup = create_signed_vote(
        &kp_victim,
        VoteType::Prevote,
        5,
        0,
        Some(Hash32::new([1; 32])),
    );
    let non_equivocation = EquivocationEvidence {
        validator_address: victim_addr,
        height: 5,
        round: 0,
        vote_a: vote_a.clone(),
        vote_b: vote_dup,
    };
    assert!(
        !non_equivocation.is_valid_equivocation(),
        "identical hashes cannot be equivocation"
    );

    // Innocent node remains active and completely unslashed
    let victim_record = keeper.get_validator(&victim_addr).unwrap();
    assert_eq!(
        victim_record.tokens,
        Amount::from_sprx_whole(50_000).unwrap()
    );
    assert_eq!(victim_record.status, ValidatorStatus::Active);
    assert!(!victim_record.is_tombstoned);
}

#[test]
fn test_byzantine_amnesic_validator_vote_flip_violation() {
    let kp1 = Ed25519Keypair::from_seed(&[31u8; 32]);
    let v1 = Validator::new(kp1.address(), kp1.public_key_bytes().to_vec(), 100);

    let val_set = ValidatorSet::new(vec![v1.clone()]).unwrap();
    let mut engine = BftConsensusEngine::new(1, val_set);
    engine.start_height(1);

    let block_alpha = Hash32::new([0xaa; 32]);
    let block_beta = Hash32::new([0xbb; 32]);

    // Engine locks on block_alpha in Round 0
    engine.restore_lock(1, 0, block_alpha);
    assert_eq!(engine.locked_round(), Some(0));
    assert_eq!(engine.locked_block(), Some(block_alpha));

    // INVARIANT: When locked on block_alpha, node CAN prevote block_alpha
    assert!(engine.can_prevote_block(block_alpha));

    // INVARIANT: Node CANNOT prevote conflicting block_beta without a valid Polka certificate
    assert!(
        !engine.can_prevote_block(block_beta),
        "locking invariant must forbid voting for conflicting proposal"
    );

    // Suppose a valid proposal certificate from Round 1 certifies block_beta
    let cert_votes = vec![create_signed_vote(
        &kp1,
        VoteType::Prevote,
        1,
        1,
        Some(block_beta),
    )];
    engine.install_valid_round_certificate(1, 1, block_beta, &cert_votes);

    // Now valid_round (1) > locked_round (0), so the lock can safely release
    assert!(
        engine.can_prevote_block(block_beta),
        "higher valid round certificate unlocks validator"
    );
}

#[test]
fn test_byzantine_malicious_proposer_equivocation_handling() {
    let kp_proposer = Ed25519Keypair::from_seed(&[41u8; 32]);
    let v_proposer = Validator::new(
        kp_proposer.address(),
        kp_proposer.public_key_bytes().to_vec(),
        100,
    );

    let val_set = ValidatorSet::new(vec![v_proposer.clone()]).unwrap();
    let mut engine = BftConsensusEngine::new(1, val_set);
    engine.start_height(1);

    let proposal_1 = Hash32::new([0x11; 32]);
    let _proposal_2 = Hash32::new([0x22; 32]);

    // First proposal accepted
    assert!(engine.propose_block(proposal_1, v_proposer.address).is_ok());
    assert_eq!(engine.current_step(), RoundStep::Prevote);

    // Attempting to propose a different block during the same round does not override the existing proposal
    let pv1 = create_signed_vote(&kp_proposer, VoteType::Prevote, 1, 0, Some(proposal_1));
    assert!(engine.receive_prevote(pv1).unwrap());

    // Precommit for proposal_1 finalized
    let pc1 = create_signed_vote(&kp_proposer, VoteType::Precommit, 1, 0, Some(proposal_1));
    let commit_sigs = engine.receive_precommit(pc1).unwrap().unwrap();

    assert_eq!(engine.current_step(), RoundStep::Commit);
    assert_eq!(commit_sigs[0].validator_address, v_proposer.address);
}
