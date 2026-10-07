use super::contract::rpc;
use clap::{Args, Subcommand};
use serde_json::json;
use sprax_node::Keyring;
use sprax_types::{
    Address, Amount, ChainId, Hash32, KeyType, Transaction, TxBody, TxFee, TxMessage,
};
use std::path::PathBuf;

#[derive(Debug, Args)]
pub(crate) struct ValidatorArgs {
    #[arg(long, global = true, default_value = "http://127.0.0.1:26657")]
    rpc_url: String,
    #[arg(long, global = true, default_value = ".sprx")]
    home: PathBuf,
    #[arg(long, global = true, default_value = "validator")]
    from: String,
    /// Pin the agreed genesis fingerprint before signing a transaction.
    #[arg(long, global = true)]
    genesis_hash: Option<String>,
    #[arg(long, global = true, default_value = "500000000000000")]
    fee_atto: String,
    #[arg(long, global = true, default_value_t = 5_000_000)]
    gas_limit: u64,
    #[command(subcommand)]
    command: ValidatorCommand,
}
#[derive(Debug, Subcommand)]
enum ValidatorCommand {
    Register {
        #[arg(long)]
        self_stake_atto: String,
        #[arg(long)]
        moniker: String,
    },
    Jail,
    Unjail,
    Delegate {
        #[arg(long)]
        validator: String,
        #[arg(long)]
        amount_atto: String,
    },
    Unbond {
        #[arg(long)]
        validator: String,
        #[arg(long)]
        amount_atto: String,
    },
    SubmitEvidence {
        #[arg(long)]
        file: PathBuf,
    },
    List,
    Evidence,
    Policy,
}
pub(crate) fn execute(args: &ValidatorArgs) -> anyhow::Result<()> {
    let query = match args.command {
        ValidatorCommand::List => Some("sprax_getValidatorRegistry"),
        ValidatorCommand::Evidence => Some("sprax_getEquivocationEvidence"),
        ValidatorCommand::Policy => Some("sprax_getValidatorPolicy"),
        _ => None,
    };
    if let Some(method) = query {
        println!(
            "{}",
            serde_json::to_string_pretty(&rpc(&args.rpc_url, method, json!([]))?)?
        );
        return Ok(());
    }
    let expected = Hash32::from_hex(
        args.genesis_hash
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("--genesis-hash is required before signing"))?,
    )?;
    let policy = rpc(&args.rpc_url, "sprax_getValidatorPolicy", json!([]))?;
    let actual = Hash32::from_hex(
        policy["genesisFingerprint"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("missing genesis fingerprint"))?,
    )?;
    anyhow::ensure!(
        actual == expected,
        "RPC genesis differs from pinned genesis"
    );
    let keyring = Keyring::open_or_create_with_development_keys(&args.home.join("keyring"), false)?;
    let key = keyring.get_ed25519_keypair(&args.from)?;
    let status = rpc(&args.rpc_url, "sprax_getStatus", json!([]))?;
    let account = rpc(
        &args.rpc_url,
        "sprax_getAccount",
        json!([key.address().to_hex()]),
    )?;
    let nonce = account["nonce"]
        .as_u64()
        .ok_or_else(|| anyhow::anyhow!("missing account nonce"))?;
    let height = status["height"]
        .as_u64()
        .ok_or_else(|| anyhow::anyhow!("missing chain height"))?;
    let message = match &args.command {
        ValidatorCommand::Register {
            self_stake_atto,
            moniker,
        } => {
            let bytes = sprax_core::validator_lifecycle::registration_sign_bytes(
                expected,
                key.address(),
                nonce,
                &key.public_key_bytes(),
            )?;
            TxMessage::RegisterValidator {
                consensus_pubkey: key.public_key_bytes().to_vec(),
                proof: key.sign(&bytes),
                self_stake: Amount::from_atto_str(self_stake_atto)?,
                moniker: moniker.clone(),
            }
        }
        ValidatorCommand::Jail => TxMessage::JailValidator {},
        ValidatorCommand::Unjail => TxMessage::UnjailValidator {},
        ValidatorCommand::Delegate {
            validator,
            amount_atto,
        } => TxMessage::Delegate {
            validator: Address::parse(validator)?,
            amount: Amount::from_atto_str(amount_atto)?,
        },
        ValidatorCommand::Unbond {
            validator,
            amount_atto,
        } => TxMessage::Unbond {
            validator: Address::parse(validator)?,
            amount: Amount::from_atto_str(amount_atto)?,
        },
        ValidatorCommand::SubmitEvidence { file } => {
            use std::io::Read;
            let mut bytes = Vec::new();
            std::fs::File::open(file)?
                .take(4097)
                .read_to_end(&mut bytes)?;
            anyhow::ensure!(bytes.len() <= 4096, "evidence exceeds 4096 bytes");
            let evidence: sprax_consensus::EquivocationEvidence = serde_json::from_slice(&bytes)?;
            TxMessage::SubmitEquivocationEvidence {
                evidence: serde_json::to_vec(&evidence)?,
            }
        }
        _ => unreachable!("queries returned above"),
    };
    let body = TxBody {
        chain_id: ChainId::new(
            status["chainId"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("missing chain ID"))?,
        )?,
        sender: key.address(),
        nonce,
        messages: vec![message],
        fee: TxFee {
            amount: Amount::from_atto_str(&args.fee_atto)?,
            gas_limit: args.gas_limit,
            priority_fee: Amount::ZERO,
        },
        memo: String::new(),
        timeout_height: height
            .checked_add(100)
            .ok_or_else(|| anyhow::anyhow!("height overflow"))?,
    };
    let signature = key.sign(&body.sign_bytes()?);
    let tx = Transaction::new(
        body,
        KeyType::Ed25519,
        key.public_key_bytes().to_vec(),
        signature,
    )?;
    let result = rpc(&args.rpc_url, "sprax_broadcastTx", json!([tx]))?;
    println!("Submitted to mempool: {}. Confirm the finalized receipt before relying on the state change.", result["txHash"]);
    Ok(())
}
