use sprax_core::{ChainLedger, GenesisConfig};
use sprax_crypto::Ed25519Keypair;
use sprax_storage::{PruningStrategy, RedbStore};
use sprax_types::{Amount, ChainId, KeyType, Transaction, TxBody, TxFee, TxMessage};

#[test]
fn test_ledger_pruning_and_reopen_from_pruned_store() {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("state.redb");

    let alice_kp = Ed25519Keypair::from_seed(&[1u8; 32]);
    let alice_addr = alice_kp.address();
    let bob_kp = Ed25519Keypair::from_seed(&[2u8; 32]);
    let bob_addr = bob_kp.address();

    let mut genesis = GenesisConfig::default_development();
    genesis.chain_id = "sprax-testnet-1".to_string();
    genesis.accounts[0].address = alice_addr;

    let genesis_clone = genesis.clone();

    // 1. Initialize ledger backed by real RedbStore
    {
        let store = RedbStore::open(&db_path).unwrap();
        let mut ledger = ChainLedger::init_from_genesis_with_store(genesis.clone(), store).unwrap();
        assert_eq!(ledger.height(), 0);
        assert_eq!(ledger.earliest_height(), 0);

        // Mine blocks 1 through 10 with transactions
        for h in 1..=10 {
            let tx_body = TxBody {
                chain_id: ChainId::new("sprax-testnet-1").unwrap(),
                sender: alice_addr,
                nonce: h - 1,
                messages: vec![TxMessage::Transfer {
                    to: bob_addr,
                    amount: Amount::from_sprx_whole(10).unwrap(),
                }],
                fee: TxFee::default(),
                memo: format!("tx-block-{h}"),
                timeout_height: 100,
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

            ledger.submit_transaction(tx).unwrap();
            let block = ledger.mine_block(alice_addr).unwrap();
            assert_eq!(block.header.height, h);
        }

        assert_eq!(ledger.height(), 10);
        assert_eq!(ledger.earliest_height(), 0);

        // Verify all 11 blocks (0..=10) exist before pruning
        for h in 0..=10 {
            assert!(ledger.get_block_by_height(h).is_some());
        }

        // 2. Prune history: KeepRecent { keep_blocks: 3 }
        // Cutoff = 10 - 3 = 7. Blocks 0..7 will be pruned; blocks 7..=10 retained.
        let stats = ledger
            .prune(PruningStrategy::KeepRecent { keep_blocks: 3 })
            .unwrap();

        assert_eq!(stats.blocks_pruned, 7);
        assert_eq!(stats.earliest_retained_height, 7);
        assert_eq!(ledger.earliest_height(), 7);

        // Pruned blocks must be None
        for h in 0..7 {
            assert!(ledger.get_block_by_height(h).is_none());
        }

        // Retained blocks must be Some
        for h in 7..=10 {
            assert!(ledger.get_block_by_height(h).is_some());
        }
    }

    // 3. Re-open the ledger from disk: must successfully rebuild starting from earliest_height
    {
        let store = RedbStore::open(&db_path).unwrap();
        let mut reopened = ChainLedger::open_or_init(store, || Ok(genesis_clone.clone())).unwrap();

        assert_eq!(reopened.height(), 10);
        assert_eq!(reopened.earliest_height(), 7);

        for h in 0..7 {
            assert!(reopened.get_block_by_height(h).is_none());
        }
        for h in 7..=10 {
            assert!(reopened.get_block_by_height(h).is_some());
        }

        // 4. Continue mining block 11 on top of the pruned state
        let tx_body = TxBody {
            chain_id: ChainId::new("sprax-testnet-1").unwrap(),
            sender: alice_addr,
            nonce: 10,
            messages: vec![TxMessage::Transfer {
                to: bob_addr,
                amount: Amount::from_sprx_whole(10).unwrap(),
            }],
            fee: TxFee::default(),
            memo: "tx-block-11".into(),
            timeout_height: 100,
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

        reopened.submit_transaction(tx).unwrap();
        let block11 = reopened.mine_block(alice_addr).unwrap();
        assert_eq!(block11.header.height, 11);
        assert_eq!(reopened.height(), 11);
    }
}
