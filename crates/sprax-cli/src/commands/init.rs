use clap::Args;
use sprax_core::{GenesisAccount, GenesisConfig, GenesisValidator};
use sprax_crypto::Ed25519Keypair;
use sprax_node::{Environment, Keyring, NodeConfig};
use sprax_types::Amount;
use std::path::PathBuf;

#[derive(Debug, Args)]
pub(crate) struct InitArgs {
    /// Chain ID for the new network
    #[arg(long, default_value = "sprax-devnet-1")]
    pub(crate) chain_id: String,

    /// Target environment (development, testnet, mainnet)
    #[arg(long, default_value = "development")]
    pub(crate) env: String,

    /// Node home directory
    #[arg(long, default_value = ".sprx")]
    pub(crate) home: PathBuf,
}

pub(crate) fn execute(args: &InitArgs) -> anyhow::Result<()> {
    let env = match args.env.to_lowercase().as_str() {
        "mainnet" => Environment::Mainnet,
        "testnet" => Environment::Testnet,
        _ => Environment::Development,
    };

    let mut config = NodeConfig::for_environment(env, args.home.clone());
    config.chain_id = args.chain_id.clone();
    // Single-node devnet: alice is the sole genesis validator (see below), so she alone
    // signs and drives the BFT consensus loop for this local chain.
    config.consensus.enabled = true;
    config.consensus.local_validator_key_name = Some("alice".to_string());

    let config_file_path = args.home.join("config.toml");
    config.save_to_file(&config_file_path)?;

    // Initialize local development keyring
    let keyring_dir = args.home.join("keyring");
    let mut keyring = Keyring::open_or_create(&keyring_dir)?;
    keyring.init_default_dev_keys()?;
    keyring.save()?;

    // Initialize Genesis configuration with accounts matching keyring
    let alice_kp = Ed25519Keypair::from_seed(&[1u8; 32]);
    let bob_kp = Ed25519Keypair::from_seed(&[2u8; 32]);
    let charlie_kp = Ed25519Keypair::from_seed(&[3u8; 32]);
    let alice_addr = alice_kp.address();
    let bob_addr = bob_kp.address();
    let charlie_addr = charlie_kp.address();

    let mut genesis = GenesisConfig::default_development();
    genesis.chain_id = args.chain_id.clone();
    genesis.accounts = vec![
        GenesisAccount {
            name: "alice".to_string(),
            address: alice_addr,
            initial_balance: Amount::from_sprx_whole(1_000_000).unwrap(),
        },
        GenesisAccount {
            name: "bob".to_string(),
            address: bob_addr,
            initial_balance: Amount::from_sprx_whole(500_000).unwrap(),
        },
        GenesisAccount {
            name: "charlie".to_string(),
            address: charlie_addr,
            initial_balance: Amount::from_sprx_whole(100_000).unwrap(),
        },
    ];
    // Alice is registered as the sole genesis validator (100% voting power) so this
    // single-node devnet has quorum by itself and finalizes blocks on `sprax start`
    // without needing peer nodes. Bob and charlie remain funded accounts for tx testing.
    genesis.validators = vec![GenesisValidator {
        operator_address: alice_addr,
        consensus_pubkey: alice_kp.public_key_bytes().to_vec(),
        self_stake: Amount::from_sprx_whole(100_000).unwrap(),
        moniker: "alice".to_string(),
    }];

    let genesis_file_path = args.home.join("genesis.json");
    genesis.save_to_file(&genesis_file_path)?;

    // Initialize empty data directory
    std::fs::create_dir_all(args.home.join("data"))?;

    println!("============================================================");
    println!("  SPRX Local Blockchain Initialized Successfully");
    println!("============================================================");
    println!("  Home Directory : {:?}", args.home);
    println!("  Chain ID       : {}", config.chain_id);
    println!("  Environment    : {}", config.environment);
    println!("  Config File    : {:?}", config_file_path);
    println!("  Genesis File   : {:?}", genesis_file_path);
    println!("  Keyring Dir    : {:?}", keyring_dir);
    println!("------------------------------------------------------------");
    println!("  Pre-Funded Development Accounts:");
    println!("  - alice   : {} (1,000,000.00 SPRX)", alice_addr);
    println!("  - bob     : {} (  500,000.00 SPRX)", bob_addr);
    println!("  - charlie : {} (  100,000.00 SPRX)", charlie_addr);
    println!("============================================================");
    println!(
        "  Run 'sprax start --home {:?}' to launch the local chain.",
        args.home
    );

    Ok(())
}
