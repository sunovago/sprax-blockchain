use sprax_consensus::{
    engine::BftConsensusEngine,
    round::RoundStep,
    staking::{
        CommissionRates, StakingKeeper, StakingParams, ValidatorDescription, ValidatorStatus,
    },
    validator::{Validator, ValidatorSet},
    vote::{Vote, VoteType},
};
use sprax_crypto::Ed25519Keypair;
use sprax_network::{message::NetworkMessage, peer::PeerId, swarm::SwarmHub};
use sprax_types::{Amount, CommitSignature, Hash32};
use std::time::Duration;

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
fn test_partition_50_50_split_brain_prevention() {
    // 4 validators: Side 1 [V1: 30, V2: 25 = 55%], Side 2 [V3: 25, V4: 20 = 45%]
    // Total power = 100. Quorum threshold (>2/3) = 67.
    let kp1 = Ed25519Keypair::from_seed(&[1u8; 32]);
    let kp2 = Ed25519Keypair::from_seed(&[2u8; 32]);
    let kp3 = Ed25519Keypair::from_seed(&[3u8; 32]);
    let kp4 = Ed25519Keypair::from_seed(&[4u8; 32]);

    let v1 = Validator::new(kp1.address(), kp1.public_key_bytes().to_vec(), 30);
    let v2 = Validator::new(kp2.address(), kp2.public_key_bytes().to_vec(), 25);
    let v3 = Validator::new(kp3.address(), kp3.public_key_bytes().to_vec(), 25);
    let v4 = Validator::new(kp4.address(), kp4.public_key_bytes().to_vec(), 20);

    let val_set = ValidatorSet::new(vec![v1.clone(), v2.clone(), v3.clone(), v4.clone()]).unwrap();

    // Side 1 Consensus Engine
    let mut engine_side1 = BftConsensusEngine::new(1, val_set.clone());
    engine_side1.start_height(1);

    // Side 2 Consensus Engine
    let mut engine_side2 = BftConsensusEngine::new(1, val_set);
    engine_side2.start_height(1);

    let block_fork_a = Hash32::new([0xaa; 32]);
    let block_fork_b = Hash32::new([0xbb; 32]);

    // Side 1 proposes and votes on Block A
    engine_side1
        .propose_block(block_fork_a, v1.address)
        .unwrap();
    let pv1 = create_signed_vote(&kp1, VoteType::Prevote, 1, 0, Some(block_fork_a));
    let pv2 = create_signed_vote(&kp2, VoteType::Prevote, 1, 0, Some(block_fork_a));
    engine_side1.receive_prevote(pv1).unwrap();
    let side1_quorum = engine_side1.receive_prevote(pv2).unwrap();

    // Side 1 power = 30 + 25 = 55 < 67 -> Quorum NOT reached!
    assert!(!side1_quorum, "Side 1 (55%) must not reach prevote quorum");
    assert_eq!(engine_side1.current_step(), RoundStep::Prevote);

    // Side 2 proposes and votes on Block B
    engine_side2
        .propose_block(block_fork_b, v3.address)
        .unwrap();
    let pv3 = create_signed_vote(&kp3, VoteType::Prevote, 1, 0, Some(block_fork_b));
    let pv4 = create_signed_vote(&kp4, VoteType::Prevote, 1, 0, Some(block_fork_b));
    engine_side2.receive_prevote(pv3).unwrap();
    let side2_quorum = engine_side2.receive_prevote(pv4).unwrap();

    // Side 2 power = 25 + 20 = 45 < 67 -> Quorum NOT reached!
    assert!(!side2_quorum, "Side 2 (45%) must not reach prevote quorum");
    assert_eq!(engine_side2.current_step(), RoundStep::Prevote);

    // Neither side can forge a precommit or finalize a block
    let pc1 = create_signed_vote(&kp1, VoteType::Precommit, 1, 0, Some(block_fork_a));
    let pc3 = create_signed_vote(&kp3, VoteType::Precommit, 1, 0, Some(block_fork_b));
    assert!(engine_side1.receive_precommit(pc1).unwrap().is_none());
    assert!(engine_side2.receive_precommit(pc3).unwrap().is_none());

    // CRITICAL SAFETY INVARIANT: Both partitions remain safely stalled, zero split-brain fork
    assert_eq!(engine_side1.current_step(), RoundStep::Prevote);
    assert_eq!(engine_side2.current_step(), RoundStep::Prevote);
}

#[test]
fn test_partition_majority_minority_progress_and_reconnection() {
    // 4 validators:
    // Majority partition: [V1: 35, V2: 35 = 70% >= 67% quorum]
    // Minority partition: [V3: 15, V4: 15 = 30% < 67% quorum]
    let kp1 = Ed25519Keypair::from_seed(&[11u8; 32]);
    let kp2 = Ed25519Keypair::from_seed(&[12u8; 32]);
    let kp3 = Ed25519Keypair::from_seed(&[13u8; 32]);
    let kp4 = Ed25519Keypair::from_seed(&[14u8; 32]);

    let v1 = Validator::new(kp1.address(), kp1.public_key_bytes().to_vec(), 35);
    let v2 = Validator::new(kp2.address(), kp2.public_key_bytes().to_vec(), 35);
    let v3 = Validator::new(kp3.address(), kp3.public_key_bytes().to_vec(), 15);
    let v4 = Validator::new(kp4.address(), kp4.public_key_bytes().to_vec(), 15);

    let val_set = ValidatorSet::new(vec![v1.clone(), v2.clone(), v3.clone(), v4.clone()]).unwrap();

    let mut majority_engine = BftConsensusEngine::new(1, val_set.clone());
    let mut minority_engine = BftConsensusEngine::new(1, val_set.clone());

    majority_engine.start_height(1);
    minority_engine.start_height(1);

    let block_h1 = Hash32::new([0x11; 32]);

    // 1. Majority Partition produces & finalizes Height 1
    majority_engine.propose_block(block_h1, v1.address).unwrap();
    let pv1 = create_signed_vote(&kp1, VoteType::Prevote, 1, 0, Some(block_h1));
    let pv2 = create_signed_vote(&kp2, VoteType::Prevote, 1, 0, Some(block_h1));
    majority_engine.receive_prevote(pv1).unwrap();
    let maj_pv_quorum = majority_engine.receive_prevote(pv2).unwrap();
    assert!(
        maj_pv_quorum,
        "Majority partition has 70% power >= 67% quorum"
    );

    let pc1 = create_signed_vote(&kp1, VoteType::Precommit, 1, 0, Some(block_h1));
    let pc2 = create_signed_vote(&kp2, VoteType::Precommit, 1, 0, Some(block_h1));
    majority_engine.receive_precommit(pc1).unwrap();
    let commit_sigs_h1: Vec<CommitSignature> = majority_engine
        .receive_precommit(pc2)
        .unwrap()
        .expect("Majority partition must finalize block H1");
    assert_eq!(majority_engine.current_step(), RoundStep::Commit);
    assert_eq!(commit_sigs_h1.len(), 2);

    // 2. Minority Partition attempts Height 1 but cannot finalize (30% power)
    let minority_block = Hash32::new([0x99; 32]);
    minority_engine
        .propose_block(minority_block, v3.address)
        .unwrap();
    let pv3 = create_signed_vote(&kp3, VoteType::Prevote, 1, 0, Some(minority_block));
    let pv4 = create_signed_vote(&kp4, VoteType::Prevote, 1, 0, Some(minority_block));
    minority_engine.receive_prevote(pv3).unwrap();
    let min_pv_quorum = minority_engine.receive_prevote(pv4).unwrap();
    assert!(
        !min_pv_quorum,
        "Minority partition has only 30% < 67% quorum"
    );

    // 3. Partition Heals: Minority receives the finalized commit signatures and block H1 from Majority
    // Minority verifies that commit_sigs_h1 carry > 2/3 voting power
    let certified_power: u64 = commit_sigs_h1
        .iter()
        .filter_map(|sig| {
            val_set
                .validators()
                .iter()
                .find(|v| v.address == sig.validator_address)
                .map(|v| v.voting_power)
        })
        .sum();
    assert!(
        val_set.has_quorum(certified_power),
        "Minority confirms majority commit certificate has valid +2/3 quorum"
    );

    // Minority fast-forwards to Height 2
    minority_engine.start_height(2);
    majority_engine.start_height(2);

    // 4. Re-converged Network: All 4 validators participate in Height 2 together
    let block_h2 = Hash32::new([0x22; 32]);
    majority_engine.propose_block(block_h2, v2.address).unwrap();
    minority_engine.propose_block(block_h2, v2.address).unwrap();

    // All 4 validators prevote for block_h2
    for kp in [&kp1, &kp2, &kp3, &kp4] {
        let pv = create_signed_vote(kp, VoteType::Prevote, 2, 0, Some(block_h2));
        majority_engine.receive_prevote(pv.clone()).unwrap();
        minority_engine.receive_prevote(pv).unwrap();
    }

    assert_eq!(majority_engine.current_step(), RoundStep::Precommit);
    assert_eq!(minority_engine.current_step(), RoundStep::Precommit);

    // All 4 validators precommit for block_h2
    for kp in [&kp1, &kp2, &kp3, &kp4] {
        let pc = create_signed_vote(kp, VoteType::Precommit, 2, 0, Some(block_h2));
        majority_engine.receive_precommit(pc.clone()).unwrap();
        minority_engine.receive_precommit(pc).unwrap();
    }

    assert_eq!(majority_engine.current_step(), RoundStep::Commit);
    assert_eq!(minority_engine.current_step(), RoundStep::Commit);
}

#[tokio::test]
async fn test_partition_p2p_swarm_packet_drop_and_delay_recovery() {
    let hub = SwarmHub::new();

    let p1 = PeerId::new("val-node-1").unwrap();
    let p2 = PeerId::new("val-node-2").unwrap();
    let p3 = PeerId::new("val-node-3").unwrap();
    let p4 = PeerId::new("val-node-4").unwrap();

    let _ep1 = hub.register_node(p1.clone());
    let ep2 = hub.register_node(p2.clone());
    let ep3 = hub.register_node(p3.clone());
    let ep4 = hub.register_node(p4.clone());

    // Partition Peer 3 from Peer 1
    hub.partition_peers(p1.clone(), p3.clone());

    // Set network delay on Peer 4
    hub.set_delay(p4.clone());

    // Node 1 broadcasts a vote message
    let kp1 = Ed25519Keypair::from_seed(&[21u8; 32]);
    let vote = create_signed_vote(&kp1, VoteType::Prevote, 10, 0, Some(Hash32::new([7; 32])));
    let vote_msg = NetworkMessage::Vote(vote.clone());

    let delivered = hub.broadcast(&p1, vote_msg).await.unwrap();
    // Connected: p2, p4 (delayed). p3 is partitioned. So 2 delivered.
    assert_eq!(delivered, 2);

    // Node 2 receives immediately
    let received_by_node2 = ep2.inbound_vote_rx.lock().await.recv().await;
    assert_eq!(received_by_node2, Some(vote.clone()));

    // Node 3 (partitioned) must NOT receive the vote
    tokio::time::sleep(Duration::from_millis(20)).await;
    let node3_pending = ep3.inbound_vote_rx.lock().await.try_recv();
    assert!(
        node3_pending.is_err(),
        "partitioned node must not receive packet"
    );

    // Node 4 (delayed) receives after delay sleeps
    tokio::time::sleep(Duration::from_millis(60)).await;
    let received_by_node4 = ep4.inbound_vote_rx.lock().await.recv().await;
    assert_eq!(received_by_node4, Some(vote));

    // HEAL PARTITION between Peer 1 and Peer 3
    hub.heal_partition(&p1, &p3);
    hub.clear_delay(&p4);

    // Broadcast again after healing
    let vote2 = create_signed_vote(&kp1, VoteType::Precommit, 10, 0, Some(Hash32::new([7; 32])));
    let delivered_healed = hub
        .broadcast(&p1, NetworkMessage::Vote(vote2.clone()))
        .await
        .unwrap();
    assert_eq!(
        delivered_healed, 3,
        "all 3 peers must receive message after heal"
    );

    let received_by_node3_after_heal = ep3.inbound_vote_rx.lock().await.recv().await;
    assert_eq!(received_by_node3_after_heal, Some(vote2));
}

#[test]
fn test_partition_downtime_slashing_under_extended_isolation() {
    let params = StakingParams {
        missed_blocks_threshold: 4,
        downtime_jail_blocks: 20,
        downtime_slash_bps: 1,
        ..StakingParams::default()
    };
    let mut keeper = StakingKeeper::new(params);

    let isolated_val = Ed25519Keypair::from_seed(&[31u8; 32]);
    let active_val = Ed25519Keypair::from_seed(&[32u8; 32]);

    keeper
        .register_validator(
            isolated_val.address(),
            isolated_val.public_key_bytes().to_vec(),
            ValidatorDescription {
                moniker: "Isolated-Node".into(),
                identity: "".into(),
                website: "".into(),
                details: "".into(),
            },
            Amount::from_sprx_whole(100_000).unwrap(),
            CommissionRates::default(),
        )
        .unwrap();

    keeper
        .register_validator(
            active_val.address(),
            active_val.public_key_bytes().to_vec(),
            ValidatorDescription {
                moniker: "Active-Node".into(),
                identity: "".into(),
                website: "".into(),
                details: "".into(),
            },
            Amount::from_sprx_whole(100_000).unwrap(),
            CommissionRates::default(),
        )
        .unwrap();

    // Isolated validator misses blocks 1, 2, 3 (below threshold of 4)
    for height in 1..=3 {
        assert_eq!(
            keeper
                .slash_downtime(&isolated_val.address(), height)
                .unwrap(),
            Amount::ZERO
        );
        assert_eq!(
            keeper
                .get_validator(&isolated_val.address())
                .unwrap()
                .status,
            ValidatorStatus::Active
        );
    }

    // Miss block 4 (reaches threshold) -> Slashed and Jailed!
    let slashed = keeper.slash_downtime(&isolated_val.address(), 4).unwrap();
    assert_ne!(slashed, Amount::ZERO);

    let val_record = keeper.get_validator(&isolated_val.address()).unwrap();
    assert_eq!(val_record.status, ValidatorStatus::Jailed);

    // Active validator set updates: isolated node is no longer counted in active set
    let active_set = keeper.get_active_validator_set().unwrap();
    assert_eq!(active_set.validators().len(), 1);
    assert_eq!(active_set.validators()[0].address, active_val.address());
}

#[tokio::test]
async fn test_partition_asymmetric_gossip_relay_progression() {
    let hub = SwarmHub::new();

    let p_a = PeerId::new("val-a").unwrap();
    let p_b = PeerId::new("val-b").unwrap();
    let p_c = PeerId::new("val-c").unwrap();
    let p_d = PeerId::new("val-d").unwrap();

    let _ep_a = hub.register_node(p_a.clone());
    let ep_b = hub.register_node(p_b.clone());
    let ep_c = hub.register_node(p_c.clone());
    let ep_d = hub.register_node(p_d.clone());

    // Asymmetric partition: A and C cannot communicate directly!
    hub.partition_peers(p_a.clone(), p_c.clone());

    let kp_a = Ed25519Keypair::from_seed(&[51u8; 32]);
    let kp_b = Ed25519Keypair::from_seed(&[52u8; 32]);
    let kp_c = Ed25519Keypair::from_seed(&[53u8; 32]);
    let kp_d = Ed25519Keypair::from_seed(&[54u8; 32]);

    let v_a = Validator::new(kp_a.address(), kp_a.public_key_bytes().to_vec(), 25);
    let v_b = Validator::new(kp_b.address(), kp_b.public_key_bytes().to_vec(), 25);
    let v_c = Validator::new(kp_c.address(), kp_c.public_key_bytes().to_vec(), 25);
    let v_d = Validator::new(kp_d.address(), kp_d.public_key_bytes().to_vec(), 25);

    let val_set =
        ValidatorSet::new(vec![v_a.clone(), v_b.clone(), v_c.clone(), v_d.clone()]).unwrap();

    let mut engine_a = BftConsensusEngine::new(1, val_set.clone());
    let mut engine_b = BftConsensusEngine::new(1, val_set.clone());
    let mut engine_c = BftConsensusEngine::new(1, val_set.clone());
    let mut engine_d = BftConsensusEngine::new(1, val_set);

    for eng in [&mut engine_a, &mut engine_b, &mut engine_c, &mut engine_d] {
        eng.start_height(1);
    }

    let block_h1 = Hash32::new([0x77; 32]);
    for eng in [&mut engine_a, &mut engine_b, &mut engine_c, &mut engine_d] {
        eng.propose_block(block_h1, v_a.address).unwrap();
    }

    // Node A creates and broadcasts its Prevote
    let pv_a = create_signed_vote(&kp_a, VoteType::Prevote, 1, 0, Some(block_h1));
    let delivered_from_a = hub
        .broadcast(&p_a, NetworkMessage::Vote(pv_a.clone()))
        .await
        .unwrap();
    // Only B and D receive it directly (2 delivered, C is partitioned from A)
    assert_eq!(delivered_from_a, 2);

    let vote_at_b = ep_b.inbound_vote_rx.lock().await.recv().await.unwrap();
    let _vote_at_d = ep_d.inbound_vote_rx.lock().await.recv().await.unwrap();

    // Node B relays/gossips the vote to the network
    let delivered_from_b = hub
        .broadcast(&p_b, NetworkMessage::Vote(vote_at_b.clone()))
        .await
        .unwrap();
    assert_eq!(delivered_from_b, 3); // Delivered to A, C, D

    // Node C successfully receives A's vote via B's relay!
    let vote_at_c_from_b = ep_c.inbound_vote_rx.lock().await.recv().await.unwrap();
    assert_eq!(vote_at_c_from_b, pv_a);

    // Now all 4 nodes process votes from A, B, C, D
    let pv_b = create_signed_vote(&kp_b, VoteType::Prevote, 1, 0, Some(block_h1));
    let pv_c = create_signed_vote(&kp_c, VoteType::Prevote, 1, 0, Some(block_h1));
    let pv_d = create_signed_vote(&kp_d, VoteType::Prevote, 1, 0, Some(block_h1));

    for eng in [&mut engine_a, &mut engine_b, &mut engine_c, &mut engine_d] {
        for v in [&pv_a, &pv_b, &pv_c, &pv_d] {
            eng.receive_prevote(v.clone()).unwrap();
        }
        assert_eq!(eng.current_step(), RoundStep::Precommit);
    }

    // Precommits also progress to Commit
    let pc_a = create_signed_vote(&kp_a, VoteType::Precommit, 1, 0, Some(block_h1));
    let pc_b = create_signed_vote(&kp_b, VoteType::Precommit, 1, 0, Some(block_h1));
    let pc_c = create_signed_vote(&kp_c, VoteType::Precommit, 1, 0, Some(block_h1));
    let pc_d = create_signed_vote(&kp_d, VoteType::Precommit, 1, 0, Some(block_h1));

    for eng in [&mut engine_a, &mut engine_b, &mut engine_c, &mut engine_d] {
        for c in [&pc_a, &pc_b, &pc_c, &pc_d] {
            eng.receive_precommit(c.clone()).unwrap();
        }
        assert_eq!(eng.current_step(), RoundStep::Commit);
    }
}
