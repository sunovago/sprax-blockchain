use sprax_consensus::{Validator, ValidatorSet};
use sprax_core::{AccountState, StateAccessor};
use sprax_crypto::{Ed25519Keypair, HdWallet, Secp256k1Keypair};
use sprax_node::{Environment, NodeConfig, NodeService};
use sprax_storage::{MemKVStore, StateCommitment};
use sprax_types::{Amount, ChainId, KeyType, Transaction, TxBody, TxFee, TxMessage};
use std::path::PathBuf;

#[tokio::test]
async fn test_end_to_end_workspace_pipeline() {
    // 1. Generate HD Wallet and derive user keys
    let wallet = HdWallet::generate_24_words().expect("failed to generate wallet");
    let alice_kp = wallet.derive_ed25519(0);
    let bob_kp = wallet.derive_ed25519(1);
    let alice_addr = alice_kp.address();
    let bob_addr = bob_kp.address();

    assert_ne!(alice_addr, bob_addr);

    // 2. Initialize in-memory state store and fund Alice
    let store = MemKVStore::new();
    let initial_alice_balance = Amount::from_sprx_whole(1_000).unwrap();
    let alice_state = AccountState {
        nonce: 0,
        balance: initial_alice_balance,
        code_hash: sprax_types::Hash32::ZERO,
        storage_root: sprax_types::Hash32::ZERO,
    };
    StateAccessor::set_account(&store, &alice_addr, &alice_state).expect("set account");

    let initial_root = store.compute_root().expect("compute root");
    assert_ne!(initial_root, sprax_types::Hash32::ZERO);

    // 3. Construct and sign a transaction from Alice to Bob
    let transfer_amount = Amount::from_sprx_whole(100).unwrap();
    let tx_body = TxBody {
        chain_id: ChainId::new(ChainId::DEVNET).unwrap(),
        sender: alice_addr,
        nonce: 0,
        messages: vec![TxMessage::Transfer {
            to: bob_addr,
            amount: transfer_amount,
        }],
        fee: TxFee::default(),
        memo: "genesis transfer".to_string(),
        timeout_height: 100,
    };

    let sign_bytes = serde_json::to_vec(&tx_body).expect("serialize tx body");
    let signature = alice_kp.sign(&sign_bytes);

    let tx = Transaction::new(
        tx_body,
        KeyType::Ed25519,
        alice_kp.public_key_bytes().to_vec(),
        signature.clone(),
    )
    .expect("create tx");

    // 4. Verify transaction signature
    assert!(Ed25519Keypair::verify(&tx.public_key, &sign_bytes, &tx.signature).is_ok());

    // 5. Form validator set and verify BFT quorum & proposer selection
    let v1 = Validator::new(alice_addr, alice_kp.public_key_bytes().to_vec(), 70);
    let v2 = Validator::new(bob_addr, bob_kp.public_key_bytes().to_vec(), 30);
    let mut val_set = ValidatorSet::new(vec![v1, v2]).expect("create validator set");

    assert_eq!(val_set.total_voting_power(), 100);
    assert_eq!(val_set.quorum_threshold(), 67);
    assert!(val_set.has_quorum(70));
    assert!(!val_set.has_quorum(60));

    let proposer_1 = val_set.select_proposer();
    assert_eq!(proposer_1.address, alice_addr);

    // 6. Initialize Node Service and verify lifecycle
    let config = NodeConfig::for_environment(
        Environment::Development,
        PathBuf::from(".sprx_test_runtime"),
    );
    let service = NodeService::new_or_load(config.home_dir.clone()).expect("create node service");
    assert!(!service.is_running());

    service.start().await.expect("start service");
    assert!(service.is_running());

    service.stop().await.expect("stop service");
    assert!(!service.is_running());
}

#[test]
fn test_secp256k1_interoperability() {
    let keypair = Secp256k1Keypair::generate();
    let msg = b"EVM Precompile Interop Verification";
    let sig = keypair.sign(msg);
    let pubkey = keypair.public_key_bytes();

    assert!(Secp256k1Keypair::verify(&pubkey, msg, &sig).is_ok());
}
