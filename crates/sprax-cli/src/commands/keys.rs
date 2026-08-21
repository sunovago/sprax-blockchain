use clap::{Args, Subcommand};
use sprax_crypto::HdWallet;
use sprax_node::Keyring;
use sprax_types::KeyType;
use std::path::PathBuf;

#[derive(Debug, Args)]
pub(crate) struct KeysArgs {
    #[command(subcommand)]
    pub(crate) command: KeysSubcommands,
}

#[derive(Debug, Subcommand)]
pub(crate) enum KeysSubcommands {
    /// Generate a standalone mnemonic phrase and address
    Generate {
        /// Algorithm: ed25519 (default) or secp256k1
        #[arg(long, default_value = "ed25519")]
        algo: String,
    },
    /// Create a new named key in the local keyring
    New {
        /// Unique key name (e.g. dev1, my_validator)
        name: String,

        /// Algorithm: ed25519 (default) or secp256k1
        #[arg(long, default_value = "ed25519")]
        algo: String,

        /// Node home directory containing keyring
        #[arg(long, default_value = ".sprx")]
        home: PathBuf,
    },
    /// List all keys in the local keyring
    List {
        /// Node home directory containing keyring
        #[arg(long, default_value = ".sprx")]
        home: PathBuf,
    },
    /// Show address and details for a named key
    Show {
        /// Key name
        name: String,

        /// Node home directory containing keyring
        #[arg(long, default_value = ".sprx")]
        home: PathBuf,
    },
    /// Derive account address from existing mnemonic phrase
    Derive {
        /// BIP-39 mnemonic phrase (space-separated words)
        #[arg(long)]
        mnemonic: String,

        /// Account index
        #[arg(long, default_value = "0")]
        index: u32,

        /// Algorithm: ed25519 or secp256k1
        #[arg(long, default_value = "ed25519")]
        algo: String,
    },
}

pub(crate) fn execute(args: &KeysArgs) -> anyhow::Result<()> {
    match &args.command {
        KeysSubcommands::Generate { algo } => {
            let wallet = HdWallet::generate_24_words()?;
            println!("============================================================");
            println!("GENERATED BIP-39 MNEMONIC PHRASE (STORE SECURELY OFF-LINE):");
            println!("{}", wallet.mnemonic_phrase());
            println!("============================================================");

            if algo.to_lowercase() == "secp256k1" {
                let kp = wallet.derive_secp256k1(0)?;
                println!("Algorithm: Secp256k1");
                println!("Address (Bech32): {}", kp.address());
                println!("Address (Hex):    {}", kp.address().to_hex());
            } else {
                let kp = wallet.derive_ed25519(0);
                println!("Algorithm: Ed25519");
                println!("Address (Bech32): {}", kp.address());
                println!("Address (Hex):    {}", kp.address().to_hex());
            }
        }
        KeysSubcommands::New { name, algo, home } => {
            let keyring_dir = home.join("keyring");
            let mut keyring = Keyring::open_or_create(&keyring_dir)?;
            let key_type = match algo.to_lowercase().as_str() {
                "secp256k1" => KeyType::Secp256k1,
                _ => KeyType::Ed25519,
            };

            let (address, mnemonic) = keyring.create_key(name, key_type)?;
            println!("============================================================");
            println!("  Created New Key: '{name}'");
            println!("------------------------------------------------------------");
            println!("  Address (Bech32) : {}", address);
            println!("  Address (Hex)    : {}", address.to_hex());
            println!("  Algorithm        : {:?}", key_type);
            println!("  Mnemonic Phrase  : {}", mnemonic);
            println!("============================================================");
        }
        KeysSubcommands::List { home } => {
            let keyring_dir = home.join("keyring");
            let keyring = Keyring::open_or_create(&keyring_dir)?;
            let keys = keyring.list();

            println!("============================================================");
            println!(
                "  Local Keyring ({:?}) - Total Keys: {}",
                keyring_dir,
                keys.len()
            );
            println!("------------------------------------------------------------");
            for key in keys {
                println!("  [{:<8}] {:?} -> {}", key.name, key.key_type, key.address);
            }
            println!("============================================================");
        }
        KeysSubcommands::Show { name, home } => {
            let keyring_dir = home.join("keyring");
            let keyring = Keyring::open_or_create(&keyring_dir)?;
            if let Some(key) = keyring.get(name) {
                println!("Key '{}':", name);
                println!("  Address (Bech32) : {}", key.address);
                println!("  Address (Hex)    : {}", key.address.to_hex());
                println!("  Algorithm        : {:?}", key.key_type);
            } else {
                println!("Key '{}' not found in keyring at {:?}", name, keyring_dir);
            }
        }
        KeysSubcommands::Derive {
            mnemonic,
            index,
            algo,
        } => {
            let wallet = HdWallet::from_mnemonic(mnemonic, "")?;
            if algo.to_lowercase() == "secp256k1" {
                let kp = wallet.derive_secp256k1(*index)?;
                println!("Algorithm: Secp256k1 (Index: {index})");
                println!("Address (Bech32): {}", kp.address());
                println!("Address (Hex):    {}", kp.address().to_hex());
            } else {
                let kp = wallet.derive_ed25519(*index);
                println!("Algorithm: Ed25519 (Index: {index})");
                println!("Address (Bech32): {}", kp.address());
                println!("Address (Hex):    {}", kp.address().to_hex());
            }
        }
    }
    Ok(())
}
