use sprax_consensus::{
    engine::BftConsensusEngine,
    round::RoundStep,
    validator::{Validator, ValidatorSet},
    vote::{Vote, VoteType},
};
use sprax_core::{
    genesis::{GenesisAccount, GenesisConfig},
    ledger::ChainLedger,
};
use sprax_crypto::Ed25519Keypair;
use sprax_indexer::IndexerEngine;
use sprax_types::{Amount, ChainId, Hash32, KeyType, Transaction, TxBody, TxFee, TxMessage};

#[test]
fn test_stress_high_tx_throughput_and_balance_invariants() {
    let mut keypairs = Vec::new();
    for i in 0..20 {
        keypairs.push(Ed25519Keypair::from_seed(&[(i + 1) as u8; 32]));
    }

    let mut genesis = GenesisConfig::default_development();
    genesis.accounts.clear();
    for (i, kp) in keypairs.iter().enumerate() {
        genesis.accounts.push(GenesisAccount {
            name: format!("account_{i}"),
            address: kp.address(),
            initial_balance: Amount::from_sprx_whole(100_000).unwrap(),
        });
    }

    let mut ledger = ChainLedger::init_from_genesis(genesis).unwrap();
    let indexer = IndexerEngine::new(ledger.chain_id().to_string());

    let initial_genesis_block = ledger.get_block_by_height(0).unwrap();
    indexer.index_block(initial_genesis_block, &[]).unwrap();

    let chain_id = ChainId::new("sprax-devnet-1").unwrap();

    // Generate 100 rapid transactions across accounts
    let mut total_sent = 0;
    for round in 0..5 {
        for i in 0..keypairs.len() {
            let sender_kp = &keypairs[i];
            let recipient_idx = (i + 1) % keypairs.len();
            let recipient_addr = keypairs[recipient_idx].address();

            let tx_body = TxBody {
                chain_id: chain_id.clone(),
                sender: sender_kp.address(),
                nonce: round,
                messages: vec![TxMessage::Transfer {
                    to: recipient_addr,
                    amount: Amount::from_sprx_whole(10).unwrap(),
                }],
                fee: TxFee::default(),
                memo: format!("Stress Tx #{round}-{i}"),
                timeout_height: 500,
            };

            let sign_bytes = tx_body.sign_bytes().unwrap();
            let sig = sender_kp.sign(&sign_bytes);
            let tx = Transaction::new(
                tx_body,
                KeyType::Ed25519,
                sender_kp.public_key_bytes().to_vec(),
                sig,
            )
            .unwrap();

            ledger.submit_transaction(tx).unwrap();
            total_sent += 1;
        }

        let block = ledger.mine_block(keypairs[0].address()).unwrap();
        indexer.index_block(&block, &[]).unwrap();
    }

    assert_eq!(total_sent, 100);
    assert_eq!(ledger.height(), 5);

    // Verify Invariant: Total supply is conserved (minus standard burned/collected fees)
    let mut total_balance_atto: u128 = 0;
    for kp in &keypairs {
        let acc = ledger.get_account(&kp.address()).unwrap();
        total_balance_atto += acc.balance.as_atto();
    }

    let initial_supply_atto = 20 * 100_000 * 1_000_000_000_000_000_000u128;
    let total_fees_atto = 100 * TxFee::default().amount.as_atto();
    assert_eq!(total_balance_atto + total_fees_atto, initial_supply_atto);
}

#[test]
#[allow(clippy::needless_range_loop)]
fn test_stress_validator_turnover_and_bft_resilience() {
    let mut keypairs = Vec::new();
    let mut validators = Vec::new();
    for i in 1..=4 {
        let kp = Ed25519Keypair::from_seed(&[i as u8; 32]);
        let val = Validator::new(kp.address(), kp.public_key_bytes().to_vec(), 25);
        validators.push(val);
        keypairs.push(kp);
    }

    let val_set = ValidatorSet::new(validators).unwrap();
    let mut engine = BftConsensusEngine::new(1, val_set);

    // Height 1: All 4 honest validators active
    engine.start_height(1);
    let proposal_hash = Hash32::new([0xbb; 32]);
    let proposer = engine.select_proposer();
    engine
        .propose_block(proposal_hash, proposer.address)
        .unwrap();

    for i in 0..4 {
        let pv = Vote::new(
            VoteType::Prevote,
            1,
            0,
            Some(proposal_hash),
            keypairs[i].address(),
            keypairs[i].sign(proposal_hash.as_bytes()),
        );
        engine.receive_prevote(pv).unwrap();
    }
    assert_eq!(engine.current_step(), RoundStep::Precommit);

    for i in 0..4 {
        let pc = Vote::new(
            VoteType::Precommit,
            1,
            0,
            Some(proposal_hash),
            keypairs[i].address(),
            keypairs[i].sign(proposal_hash.as_bytes()),
        );
        engine.receive_precommit(pc).unwrap();
    }
    assert_eq!(engine.current_step(), RoundStep::Commit);

    // Height 2: Validator 4 goes OFFLINE.
    // 3 out of 4 validators (75% power > 66.7% quorum) reach consensus!
    engine.start_height(2);
    let proposal_hash_2 = Hash32::new([0xcc; 32]);
    let proposer_2 = engine.select_proposer();
    engine
        .propose_block(proposal_hash_2, proposer_2.address)
        .unwrap();

    for i in 0..3 {
        let pv = Vote::new(
            VoteType::Prevote,
            2,
            0,
            Some(proposal_hash_2),
            keypairs[i].address(),
            keypairs[i].sign(proposal_hash_2.as_bytes()),
        );
        engine.receive_prevote(pv).unwrap();
    }
    assert_eq!(engine.current_step(), RoundStep::Precommit);

    for i in 0..3 {
        let pc = Vote::new(
            VoteType::Precommit,
            2,
            0,
            Some(proposal_hash_2),
            keypairs[i].address(),
            keypairs[i].sign(proposal_hash_2.as_bytes()),
        );
        engine.receive_precommit(pc).unwrap();
    }
    assert_eq!(engine.current_step(), RoundStep::Commit);
}
