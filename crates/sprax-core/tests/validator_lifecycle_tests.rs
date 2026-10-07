use sprax_consensus::{verify_block_commit, EquivocationEvidence, ValidatorSet, Vote, VoteType};
use sprax_core::{
    ChainLedger, GenesisAccount, GenesisConfig, GenesisValidator, StateAccessor, TxExecutor,
};
use sprax_crypto::{Ed25519Keypair, Hasher};
use sprax_storage::{MemKVStore, StateCommitment};
use sprax_types::{
    Amount, Block, ChainId, CommitSignature, Hash32, KeyType, Transaction, TxBody, TxFee, TxMessage,
};

fn coins(n: u64) -> Amount {
    Amount::from_sprx_whole(n.into()).unwrap()
}
fn setup() -> (GenesisConfig, Vec<Ed25519Keypair>, MemKVStore, ChainLedger) {
    let keys: Vec<_> = (81..84)
        .map(|seed| Ed25519Keypair::from_seed(&[seed; 32]))
        .collect();
    let mut g = GenesisConfig::default_development();
    g.consensus_params.initial_block_reward = Amount::ZERO;
    g.consensus_params.validator_policy.jail_blocks = 2;
    g.accounts = keys
        .iter()
        .map(|k| GenesisAccount {
            name: "operator".into(),
            address: k.address(),
            initial_balance: coins(1000),
        })
        .collect();
    g.validators = keys
        .iter()
        .take(2)
        .map(|k| GenesisValidator {
            operator_address: k.address(),
            consensus_pubkey: k.public_key_bytes().to_vec(),
            self_stake: coins(100),
            moniker: "validator".into(),
        })
        .collect();
    let store = MemKVStore::new();
    let ledger = ChainLedger::init_from_genesis_with_store(g.clone(), store.clone()).unwrap();
    (g, keys, store, ledger)
}
fn transaction(
    ledger: &ChainLedger,
    key: &Ed25519Keypair,
    messages: Vec<TxMessage>,
) -> Transaction {
    let body = TxBody {
        chain_id: ChainId::new(&ledger.genesis().chain_id).unwrap(),
        sender: key.address(),
        nonce: ledger.get_account(&key.address()).unwrap().nonce,
        messages,
        fee: TxFee {
            amount: Amount::ZERO,
            gas_limit: 2_000_000,
            priority_fee: Amount::ZERO,
        },
        memo: String::new(),
        timeout_height: 100,
    };
    let sig = key.sign(&body.sign_bytes().unwrap());
    Transaction::new(body, KeyType::Ed25519, key.public_key_bytes().to_vec(), sig).unwrap()
}
fn evidence(g: &GenesisConfig, key: &Ed25519Keypair, height: u64) -> EquivocationEvidence {
    let sign = |value| {
        let mut vote = Vote::new(
            g.fingerprint().unwrap(),
            VoteType::Precommit,
            height,
            0,
            Some(Hash32::new([value; 32])),
            key.address(),
            vec![],
        );
        vote.signature = key.sign(&vote.sign_bytes().unwrap());
        vote
    };
    EquivocationEvidence {
        validator_address: key.address(),
        height,
        round: 0,
        vote_a: sign(1),
        vote_b: sign(2),
    }
}
fn message(e: &EquivocationEvidence) -> TxMessage {
    TxMessage::SubmitEquivocationEvidence {
        evidence: serde_json::to_vec(e).unwrap(),
    }
}
fn certify(block: &mut Block, keys: &[&Ed25519Keypair], g: &GenesisConfig) {
    let hash = Hasher::block_hash(&block.header).unwrap();
    block.last_commit = keys
        .iter()
        .map(|key| {
            let vote = Vote::new(
                g.fingerprint().unwrap(),
                VoteType::Precommit,
                block.header.height,
                0,
                Some(hash),
                key.address(),
                vec![],
            );
            CommitSignature {
                round: 0,
                validator_address: key.address(),
                signature: key.sign(&vote.sign_bytes().unwrap()),
                timestamp_unix_secs: block.header.timestamp_unix_secs,
            }
        })
        .collect();
}
#[test]
fn finalized_slash_updates_delegations_supply_next_height_power_and_survives_restart() {
    let (g, keys, store, mut source) = setup();
    let mut replica = ChainLedger::init_from_genesis(g.clone()).unwrap();
    let delegate = transaction(
        &source,
        &keys[2],
        vec![TxMessage::Delegate {
            validator: keys[0].address(),
            amount: coins(40),
        }],
    );
    source.submit_transaction(delegate).unwrap();
    replica
        .apply_block(source.mine_block(keys[1].address()).unwrap())
        .unwrap();
    replica
        .apply_block(source.mine_block(keys[1].address()).unwrap())
        .unwrap();
    let before_set = ValidatorSet::from_canonical(source.active_validators().unwrap()).unwrap();
    let proof = evidence(&g, &keys[0], 2);
    source
        .submit_transaction(transaction(&source, &keys[1], vec![message(&proof)]))
        .unwrap();
    let before = source.state_root().unwrap();
    let mut proposal = source.build_proposal(keys[1].address()).unwrap();
    source.validate_proposal(proposal.clone()).unwrap();
    assert_eq!(source.state_root().unwrap(), before);
    assert!(
        !source
            .validator_registry()
            .unwrap()
            .iter()
            .find(|v| v.operator_address == keys[0].address())
            .unwrap()
            .tombstoned
    );
    certify(&mut proposal, &[&keys[0], &keys[1]], &g);
    verify_block_commit(&proposal, &before_set, g.fingerprint().unwrap()).unwrap();
    source.apply_block(proposal.clone()).unwrap();
    replica.apply_block(proposal).unwrap();
    assert_eq!(source.state_root().unwrap(), replica.state_root().unwrap());
    assert_eq!(
        source
            .get_validator_stake(&keys[0].address())
            .unwrap()
            .tokens,
        coins(133)
    );
    assert_eq!(
        source
            .get_delegation(&keys[0].address(), &keys[0].address())
            .unwrap()
            .balance,
        coins(95)
    );
    assert_eq!(
        source
            .get_delegation(&keys[2].address(), &keys[0].address())
            .unwrap()
            .balance,
        coins(38)
    );
    assert_eq!(source.get_supply_state().unwrap().total_burned, coins(7));
    assert_eq!(
        source.get_supply_state().unwrap().circulating_supply,
        coins(2993)
    );
    assert_eq!(source.active_validators().unwrap().len(), 1);
    let mut next = source.mine_block(keys[1].address()).unwrap();
    certify(&mut next, &[&keys[1]], &g);
    verify_block_commit(
        &next,
        &ValidatorSet::from_canonical(source.active_validators().unwrap()).unwrap(),
        g.fingerprint().unwrap(),
    )
    .unwrap();
    replica.apply_block(next).unwrap();
    let reopened = ChainLedger::open_or_init(store, || Ok(g.clone())).unwrap();
    assert_eq!(
        reopened.state_root().unwrap(),
        replica.state_root().unwrap()
    );
    assert_eq!(
        reopened.validator_registry().unwrap(),
        replica.validator_registry().unwrap()
    );
}
#[test]
fn unbond_before_evidence_cannot_escape_and_maturity_returns_only_remainder() {
    let (g, keys, store, mut ledger) = setup();
    ledger.mine_block(keys[1].address()).unwrap();
    ledger
        .submit_transaction(transaction(
            &ledger,
            &keys[0],
            vec![TxMessage::Unbond {
                validator: keys[0].address(),
                amount: coins(40),
            }],
        ))
        .unwrap();
    ledger
        .submit_transaction(transaction(
            &ledger,
            &keys[1],
            vec![message(&evidence(&g, &keys[0], 1))],
        ))
        .unwrap();
    ledger.mine_block(keys[1].address()).unwrap();
    assert_eq!(
        ledger
            .get_validator_stake(&keys[0].address())
            .unwrap()
            .tokens,
        coins(57)
    );
    assert_eq!(ledger.get_supply_state().unwrap().total_burned, coins(5));
    let pending = StateAccessor::scan_matured_unbondings(&store, 12).unwrap();
    assert_eq!(pending[0].1.amount, coins(38));
    while ledger.height() < 12 {
        ledger.mine_block(keys[1].address()).unwrap();
    }
    assert_eq!(
        ledger.get_account(&keys[0].address()).unwrap().balance,
        coins(938)
    );
    assert!(StateAccessor::scan_matured_unbondings(&store, 12)
        .unwrap()
        .is_empty());
}
#[test]
fn forged_foreign_future_inactive_expired_and_replayed_evidence_is_atomic() {
    let (g, keys, store, mut ledger) = setup();
    ledger.mine_block(keys[1].address()).unwrap();
    let good = evidence(&g, &keys[0], 1);
    let mut forged = good.clone();
    forged.vote_a.signature[0] ^= 1;
    let mut foreign_g = g.clone();
    foreign_g.chain_id = "sprax-other".into();
    let mut bad_meta = good.clone();
    bad_meta.round += 1;
    for bad in [
        forged,
        evidence(&foreign_g, &keys[0], 1),
        evidence(&g, &keys[0], 2),
        evidence(&g, &keys[2], 1),
        bad_meta,
    ] {
        let before = store.compute_root().unwrap();
        let tx = transaction(&ledger, &keys[1], vec![message(&bad)]);
        assert!(TxExecutor::default()
            .execute_transaction(&store, &tx, 2, &g.chain_id, 10)
            .is_err());
        assert_eq!(store.compute_root().unwrap(), before);
    }
    let expired = transaction(&ledger, &keys[1], vec![message(&good)]);
    assert!(TxExecutor::default()
        .execute_transaction(&store, &expired, 11, &g.chain_id, 10)
        .is_err());
    let invalid_after_slash = transaction(&ledger, &keys[1], vec![message(&good), message(&good)]);
    let before = store.compute_root().unwrap();
    assert!(TxExecutor::default()
        .execute_transaction(&store, &invalid_after_slash, 2, &g.chain_id, 10)
        .is_err());
    assert_eq!(store.compute_root().unwrap(), before);
    ledger
        .submit_transaction(transaction(&ledger, &keys[1], vec![message(&good)]))
        .unwrap();
    ledger.mine_block(keys[1].address()).unwrap();
    let mut reversed = good;
    std::mem::swap(&mut reversed.vote_a, &mut reversed.vote_b);
    let tx = transaction(&ledger, &keys[1], vec![message(&reversed)]);
    let before = store.compute_root().unwrap();
    assert!(TxExecutor::default()
        .execute_transaction(&store, &tx, 3, &g.chain_id, 10)
        .is_err());
    assert_eq!(store.compute_root().unwrap(), before);
    let unjail = transaction(&ledger, &keys[0], vec![TxMessage::UnjailValidator {}]);
    assert!(TxExecutor::default()
        .execute_transaction(&store, &unjail, 50, &g.chain_id, 10)
        .is_err());
    assert_eq!(store.compute_root().unwrap(), before);
}
#[test]
fn registration_jail_unjail_and_self_bond_exit_are_canonical() {
    let (g, keys, store, mut ledger) = setup();
    let key = &keys[2];
    let registration = || TxMessage::RegisterValidator {
        consensus_pubkey: key.public_key_bytes().to_vec(),
        proof: key.sign(
            &sprax_core::validator_lifecycle::registration_sign_bytes(
                g.fingerprint().unwrap(),
                key.address(),
                0,
                &key.public_key_bytes(),
            )
            .unwrap(),
        ),
        self_stake: coins(50),
        moniker: "new validator".into(),
    };
    let mut forged = registration();
    if let TxMessage::RegisterValidator { proof, .. } = &mut forged {
        proof[0] ^= 1;
    }
    let before = store.compute_root().unwrap();
    assert!(TxExecutor::default()
        .execute_transaction(
            &store,
            &transaction(&ledger, key, vec![forged]),
            1,
            &g.chain_id,
            10
        )
        .is_err());
    assert_eq!(store.compute_root().unwrap(), before);
    ledger
        .submit_transaction(transaction(&ledger, key, vec![registration()]))
        .unwrap();
    let initial_set = ledger.active_validator_set_hash().unwrap();
    let proposal = ledger.build_proposal(keys[0].address()).unwrap();
    assert_eq!(proposal.header.validator_set_hash, initial_set);
    assert_eq!(ledger.active_validators().unwrap().len(), 2);
    ledger.apply_block(proposal).unwrap();
    assert_eq!(ledger.active_validators().unwrap().len(), 3);
    let duplicate = transaction(&ledger, key, vec![registration()]);
    assert!(TxExecutor::default()
        .execute_transaction(&store, &duplicate, 2, &g.chain_id, 10)
        .is_err());
    ledger
        .submit_transaction(transaction(&ledger, key, vec![TxMessage::JailValidator {}]))
        .unwrap();
    ledger.mine_block(keys[0].address()).unwrap();
    assert_eq!(ledger.active_validators().unwrap().len(), 2);
    let early = transaction(&ledger, key, vec![TxMessage::UnjailValidator {}]);
    assert!(TxExecutor::default()
        .execute_transaction(&store, &early, 3, &g.chain_id, 10)
        .is_err());
    ledger.mine_block(keys[0].address()).unwrap();
    ledger
        .submit_transaction(transaction(
            &ledger,
            key,
            vec![TxMessage::UnjailValidator {}],
        ))
        .unwrap();
    ledger.mine_block(keys[0].address()).unwrap();
    assert_eq!(ledger.active_validators().unwrap().len(), 3);
    ledger
        .submit_transaction(transaction(
            &ledger,
            key,
            vec![TxMessage::Unbond {
                validator: key.address(),
                amount: coins(50),
            }],
        ))
        .unwrap();
    ledger.mine_block(keys[0].address()).unwrap();
    assert_eq!(ledger.active_validators().unwrap().len(), 2);
    assert!(
        ledger
            .validator_registry()
            .unwrap()
            .iter()
            .find(|v| v.operator_address == key.address())
            .unwrap()
            .jailed
    );
}

#[test]
fn slashing_out_of_gas_after_a_partial_loop_rolls_back_every_write() {
    let (g, keys, store, mut ledger) = setup();
    ledger
        .submit_transaction(transaction(
            &ledger,
            &keys[2],
            vec![TxMessage::Delegate {
                validator: keys[0].address(),
                amount: coins(40),
            }],
        ))
        .unwrap();
    ledger.mine_block(keys[1].address()).unwrap();
    let mut tx = transaction(&ledger, &keys[1], vec![message(&evidence(&g, &keys[0], 1))]);
    // 71,000 fixed gas plus 1,500 per delegation: first write succeeds, second fails.
    tx.body.fee.gas_limit = 73_000;
    tx.signature = keys[1].sign(&tx.body.sign_bytes().unwrap());
    let before = store.compute_root().unwrap();
    assert!(matches!(
        TxExecutor::default().execute_transaction(&store, &tx, 2, &g.chain_id, 10),
        Err(sprax_core::CoreError::OutOfGas { .. })
    ));
    assert_eq!(store.compute_root().unwrap(), before);
}
