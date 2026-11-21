use clap::Args;
use sprax_core::{ChainLedger, GenesisAccount, GenesisConfig, GenesisValidator};
use sprax_crypto::Ed25519Keypair;
use sprax_node::{Environment, Keyring, NodeConfig};
use sprax_types::Amount;
use std::path::PathBuf;

#[derive(Debug, Args)]
pub(crate) struct InitArgs {
    /// Chain ID; defaults to the supplied genesis or development chain ID
    #[arg(long)]
    pub(crate) chain_id: Option<String>,
    /// Target environment (development, testnet, mainnet)
    #[arg(long, default_value = "development")]
    pub(crate) env: String,
    /// Node home directory; must be empty or absent
    #[arg(long, default_value = ".sprx")]
    pub(crate) home: PathBuf,
    /// Agreed genesis JSON; required for testnet and mainnet
    #[arg(long)]
    pub(crate) genesis: Option<PathBuf>,
}

pub(crate) fn execute(args: &InitArgs) -> anyhow::Result<()> {
    let env = match args.env.to_lowercase().as_str() {
        "development" => Environment::Development,
        "testnet" => Environment::Testnet,
        "mainnet" => Environment::Mainnet,
        other => anyhow::bail!("unknown environment: {other}"),
    };
    if args.home.exists() && args.home.read_dir()?.next().is_some() {
        anyhow::bail!("node home is not empty; initialization must not overwrite configuration, genesis or signing state");
    }
    let default_development = args.genesis.is_none() && env == Environment::Development;
    let mut genesis = match &args.genesis {
        Some(path) => GenesisConfig::load_from_file(path)?,
        None if default_development => development_genesis()?,
        None => anyhow::bail!(
            "testnet/mainnet initialization requires --genesis with the agreed public genesis JSON"
        ),
    };
    if let Some(chain_id) = &args.chain_id {
        if default_development {
            genesis.chain_id = chain_id.clone();
        } else if &genesis.chain_id != chain_id {
            anyhow::bail!("--chain-id does not match the supplied genesis");
        }
    }
    if env != Environment::Development {
        if genesis.validators.is_empty() {
            anyhow::bail!("non-development genesis requires validators");
        }
        for seed in 1..=3 {
            let key = Ed25519Keypair::from_seed(&[seed; 32]);
            if genesis
                .validators
                .iter()
                .any(|v| v.consensus_pubkey == key.public_key_bytes())
            {
                anyhow::bail!(
                    "non-development genesis contains a public development validator key"
                );
            }
        }
    }
    // Validate all accounting and identity invariants before writing any home files.
    ChainLedger::init_from_genesis(genesis.clone())?;
    let mut config = NodeConfig::for_environment(env, args.home.clone());
    config.chain_id = genesis.chain_id.clone();
    if default_development {
        config.consensus.enabled = true;
        config.consensus.local_validator_key_name = Some("alice".into());
    }
    let config_file = args.home.join("config.toml");
    let genesis_file = args.home.join("genesis.json");
    config.save_to_file(&config_file)?;
    genesis.save_to_file(&genesis_file)?;
    Keyring::open_or_create_with_development_keys(&args.home.join("keyring"), default_development)?
        .save()?;
    std::fs::create_dir_all(args.home.join("data"))?;
    println!(
        "Initialized {} node at {:?} (chain {})",
        env, args.home, config.chain_id
    );
    if !default_development {
        println!("Consensus signing is disabled. Configure the operator key and consensus settings before validator startup.");
    }
    Ok(())
}

fn development_genesis() -> anyhow::Result<GenesisConfig> {
    let mut genesis = GenesisConfig::default_development();
    genesis.accounts = [
        (1, "alice", 1_000_000),
        (2, "bob", 500_000),
        (3, "charlie", 100_000),
    ]
    .into_iter()
    .map(|(seed, name, balance)| {
        Ok(GenesisAccount {
            name: name.into(),
            address: Ed25519Keypair::from_seed(&[seed; 32]).address(),
            initial_balance: Amount::from_sprx_whole(balance)?,
        })
    })
    .collect::<anyhow::Result<_>>()?;
    let alice = Ed25519Keypair::from_seed(&[1; 32]);
    genesis.validators = vec![GenesisValidator {
        operator_address: alice.address(),
        consensus_pubkey: alice.public_key_bytes().to_vec(),
        self_stake: Amount::from_sprx_whole(100_000)?,
        moniker: "alice".into(),
    }];
    Ok(genesis)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn args(env: &str) -> InitArgs {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        InitArgs {
            chain_id: None,
            env: env.into(),
            home: std::env::temp_dir().join(format!("sprax-init-{}-{unique}", std::process::id())),
            genesis: None,
        }
    }
    #[test]
    fn rejects_unknown_environment_and_missing_public_genesis_before_writes() {
        for env in ["maninet", "mainnet", "testnet"] {
            let a = args(env);
            assert!(execute(&a).is_err());
            assert!(!a.home.exists());
        }
    }
    #[test]
    fn refuses_to_overwrite_existing_node_home() {
        let a = args("development");
        std::fs::create_dir_all(&a.home).unwrap();
        let marker = a.home.join("validator-signing.redb");
        std::fs::write(&marker, b"preserve signing state").unwrap();
        assert!(execute(&a).is_err());
        assert_eq!(std::fs::read(marker).unwrap(), b"preserve signing state");
        assert!(!a.home.join("config.toml").exists());
        std::fs::remove_dir_all(a.home).unwrap();
    }
    #[test]
    fn nondevelopment_init_rejects_public_dev_keys_and_chain_mismatch() {
        let mut a = args("mainnet");
        let input = a.home.with_extension("genesis.json");
        development_genesis().unwrap().save_to_file(&input).unwrap();
        a.genesis = Some(input.clone());
        assert!(execute(&a)
            .unwrap_err()
            .to_string()
            .contains("public development"));
        a.chain_id = Some("other-chain".into());
        assert!(execute(&a)
            .unwrap_err()
            .to_string()
            .contains("does not match"));
        assert!(!a.home.exists());
        std::fs::remove_file(input).unwrap();
    }
    #[test]
    fn explicit_mainnet_genesis_initializes_an_empty_non_signing_keyring() {
        let mut a = args("mainnet");
        let input = a.home.with_extension("genesis.json");
        let key = Ed25519Keypair::from_seed(&[42; 32]);
        let mut genesis = development_genesis().unwrap();
        genesis.chain_id = "sprax-mainnet-1".into();
        genesis.accounts = vec![GenesisAccount {
            name: "operator".into(),
            address: key.address(),
            initial_balance: Amount::from_sprx_whole(1_000_000).unwrap(),
        }];
        genesis.validators[0].operator_address = key.address();
        genesis.validators[0].consensus_pubkey = key.public_key_bytes().to_vec();
        genesis.save_to_file(&input).unwrap();
        a.genesis = Some(input.clone());
        execute(&a).unwrap();
        let config = NodeConfig::load_from_file(&a.home.join("config.toml")).unwrap();
        assert_eq!(config.environment, Environment::Mainnet);
        assert_eq!(config.chain_id, genesis.chain_id);
        assert!(!config.consensus.enabled);
        assert!(config.consensus.local_validator_key_name.is_none());
        assert!(
            Keyring::open_or_create_with_development_keys(&a.home.join("keyring"), false)
                .unwrap()
                .list()
                .is_empty()
        );
        let stored = GenesisConfig::load_from_file(&a.home.join("genesis.json")).unwrap();
        assert_eq!(
            serde_json::to_vec(&stored).unwrap(),
            serde_json::to_vec(&genesis).unwrap()
        );
        std::fs::remove_dir_all(a.home).unwrap();
        std::fs::remove_file(input).unwrap();
    }
}
