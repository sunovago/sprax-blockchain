use sprax_consensus::{CommissionRates, StakingKeeper, ValidatorDescription};
use sprax_core::{genesis::GenesisConfig, ledger::ChainLedger};
use sprax_crypto::{Ed25519Keypair, Hasher};
use sprax_indexer::{api::ExplorerApi, engine::IndexerEngine, models::SearchResult};
use sprax_types::{Amount, ChainId, KeyType, Transaction, TxBody, TxFee, TxMessage};

#[test]
fn test_indexer_full_pipeline_and_consistency_verification() {
    let alice_kp = Ed25519Keypair::from_seed(&[1u8; 32]);
    let bob_kp = Ed25519Keypair::from_seed(&[2u8; 32]);

    let mut genesis = GenesisConfig::default_development();
    genesis.accounts[0].address = alice_kp.address();
    genesis.accounts[1].address = bob_kp.address();

    let mut ledger = ChainLedger::init_from_genesis(genesis).unwrap();
    let indexer = IndexerEngine::new(ledger.chain_id().to_string());
    let api = ExplorerApi::new(indexer.store());

    // 1. Index Genesis Block #0
    let genesis_block = ledger.get_block_by_height(0).unwrap();
    indexer.index_block(genesis_block, &[]).unwrap();

    // Initial account update
    let alice_acc = ledger.get_account(&alice_kp.address()).unwrap();
    let bob_acc = ledger.get_account(&bob_kp.address()).unwrap();
    indexer.update_account(alice_kp.address(), alice_acc.balance, alice_acc.nonce, 0);
    indexer.update_account(bob_kp.address(), bob_acc.balance, bob_acc.nonce, 0);

    // Initial consistency check
    indexer
        .verify_consistency(&ledger)
        .expect("genesis consistency must pass");

    // 2. Submit Transaction & Mine Block #1
    let tx_body = TxBody {
        chain_id: ChainId::new("sprax-devnet-1").unwrap(),
        sender: alice_kp.address(),
        nonce: 0,
        messages: vec![TxMessage::Transfer {
            to: bob_kp.address(),
            amount: Amount::from_sprx_whole(1_500).unwrap(),
        }],
        fee: TxFee::default(),
        memo: "Indexer test transfer".into(),
        timeout_height: 10,
    };

    let sign_bytes = tx_body.sign_bytes().unwrap();
    let sig = alice_kp.sign(&sign_bytes);
    let tx = Transaction::new(
        tx_body,
        KeyType::Ed25519,
        alice_kp.public_key_bytes().to_vec(),
        sig,
    )
    .unwrap();

    let tx_hash = ledger.submit_transaction(tx).unwrap();
    let block1 = ledger.mine_block(alice_kp.address()).unwrap();

    // Replay receipts from ledger
    let (_, r, _) = ledger.get_transaction(&tx_hash).unwrap();
    indexer
        .index_block(&block1, std::slice::from_ref(r))
        .unwrap();

    // Update account balances post-tx
    let alice_post = ledger.get_account(&alice_kp.address()).unwrap();
    let bob_post = ledger.get_account(&bob_kp.address()).unwrap();
    indexer.update_account(alice_kp.address(), alice_post.balance, alice_post.nonce, 1);
    indexer.update_account(bob_kp.address(), bob_post.balance, bob_post.nonce, 1);

    // Verify consistency at Height 1
    indexer
        .verify_consistency(&ledger)
        .expect("height 1 consistency must pass");

    // 3. Test Explorer API Querying
    let blocks_resp = api.get_blocks(Some(10), Some(0));
    assert_eq!(blocks_resp.total, 2); // Block #0 and #1
    assert_eq!(blocks_resp.items[0].height, 1);
    assert_eq!(blocks_resp.items[1].height, 0);

    let block1_query = api.get_block("1").unwrap();
    assert_eq!(block1_query.height, 1);
    assert_eq!(block1_query.txs_count, 1);

    let txs_resp = api.get_transactions(Some(10), Some(0));
    assert_eq!(txs_resp.total, 1);
    assert_eq!(txs_resp.items[0].tx_hash, tx_hash);
    assert_eq!(txs_resp.items[0].sender, alice_kp.address());
    assert_eq!(txs_resp.items[0].recipient, Some(bob_kp.address()));
    assert_eq!(
        txs_resp.items[0].amount,
        Amount::from_sprx_whole(1_500).unwrap()
    );

    let alice_addr_txs = api
        .get_address_transactions(&alice_kp.address().to_string(), None, None)
        .unwrap();
    assert_eq!(alice_addr_txs.total, 1);

    let bob_addr_txs = api
        .get_address_transactions(&bob_kp.address().to_string(), None, None)
        .unwrap();
    assert_eq!(bob_addr_txs.total, 1);

    // 4. Test Search Functionality
    // Search by height "1"
    match api.search("1").unwrap() {
        SearchResult::Block(b) => assert_eq!(b.height, 1),
        _ => panic!("expected block search result"),
    }

    // Search by Tx Hash
    match api.search(&tx_hash.to_string()).unwrap() {
        SearchResult::Transaction(t) => assert_eq!(t.tx_hash, tx_hash),
        _ => panic!("expected tx search result"),
    }

    // Search by Block Hash
    let block1_hash = Hasher::block_hash(&block1.header).unwrap();
    match api.search(&block1_hash.to_string()).unwrap() {
        SearchResult::Block(b) => assert_eq!(b.hash, block1_hash),
        _ => panic!("expected block search result"),
    }

    // Search by Address
    match api.search(&alice_kp.address().to_string()).unwrap() {
        SearchResult::Address(a) => assert_eq!(a.address, alice_kp.address()),
        _ => panic!("expected address search result"),
    }

    // 5. Test Validator Sync & Network Stats
    let mut staking = StakingKeeper::default();
    staking
        .register_validator(
            alice_kp.address(),
            alice_kp.public_key_bytes().to_vec(),
            ValidatorDescription {
                moniker: "Genesis-Val-Alpha".into(),
                identity: "".into(),
                website: "https://sprax.network".into(),
                details: "Core Testnet Validator".into(),
            },
            Amount::from_sprx_whole(100_000).unwrap(),
            CommissionRates::default(),
        )
        .unwrap();

    indexer.sync_validators(&staking);
    let val_list = api.get_validators();
    assert_eq!(val_list.len(), 1);
    assert_eq!(val_list[0].moniker, "Genesis-Val-Alpha");
    assert_eq!(val_list[0].voting_power_percentage, 100.0);

    // Search by Moniker
    match api.search("Genesis-Val-Alpha").unwrap() {
        SearchResult::Validator(v) => assert_eq!(v.moniker, "Genesis-Val-Alpha"),
        _ => panic!("expected validator search result"),
    }

    let stats = api.get_stats();
    assert_eq!(stats.latest_height, 1);
    assert_eq!(stats.total_transactions, 1);
    assert_eq!(stats.active_validators_count, 1);
}
