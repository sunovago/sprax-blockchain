use clap::{Args, Subcommand};
use serde_json::{json, Value};
use sprax_node::Keyring;
use sprax_types::{
    Address, Amount, ChainId, Hash32, KeyType, Transaction, TxBody, TxFee, TxMessage,
};
use std::{path::PathBuf, time::Duration};

#[derive(Debug, Args)]
pub(crate) struct ContractArgs {
    #[arg(long, global = true, default_value = "http://127.0.0.1:26657")]
    rpc_url: String,
    #[arg(long, global = true, default_value = ".sprx")]
    home: PathBuf,
    #[arg(long, global = true, default_value = "500000000000000")]
    fee_atto: String,
    #[arg(long, global = true, default_value_t = 5_000_000)]
    gas_limit: u64,
    #[command(subcommand)]
    command: ContractCommand,
}

#[derive(Debug, Subcommand)]
enum ContractCommand {
    StoreCode {
        #[arg(long)]
        from: String,
        #[arg(long)]
        wasm: PathBuf,
    },
    Instantiate {
        #[arg(long)]
        from: String,
        #[arg(long)]
        code_id: String,
        #[arg(long)]
        msg: String,
        #[arg(long)]
        label: String,
        #[arg(long, default_value = "0")]
        funds_atto: String,
    },
    Execute {
        #[arg(long)]
        from: String,
        #[arg(long)]
        contract: String,
        #[arg(long)]
        msg: String,
        #[arg(long, default_value = "0")]
        funds_atto: String,
    },
    Query {
        #[arg(long)]
        contract: String,
        #[arg(long)]
        msg: String,
    },
}

fn rpc(url: &str, method: &str, params: Value) -> anyhow::Result<Value> {
    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(30))
        .build();
    let response: Value = agent
        .post(url)
        .send_json(json!({"jsonrpc":"2.0","id":1,"method":method,"params":params}))?
        .into_json()?;
    if let Some(error) = response.get("error") {
        anyhow::bail!("RPC error: {error}");
    }
    response
        .get("result")
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("missing RPC result"))
}

pub(crate) fn execute(args: &ContractArgs) -> anyhow::Result<()> {
    if let ContractCommand::Query { contract, msg } = &args.command {
        let address = Address::parse(contract)?;
        let message: Value = serde_json::from_str(msg)?;
        let result = rpc(
            &args.rpc_url,
            "sprax_queryContract",
            json!([address.to_hex(), message, 200_000]),
        )?;
        let bytes: Vec<u8> = serde_json::from_value(result["data"].clone())?;
        println!("{}", String::from_utf8(bytes)?);
        return Ok(());
    }
    let (from, message) = match &args.command {
        ContractCommand::StoreCode { from, wasm } => (
            from,
            TxMessage::StoreCode {
                wasm_bytecode: std::fs::read(wasm)?,
            },
        ),
        ContractCommand::Instantiate {
            from,
            code_id,
            msg,
            label,
            funds_atto,
        } => {
            let value: Value = serde_json::from_str(msg)?;
            (
                from,
                TxMessage::InstantiateContract {
                    code_id: Hash32::from_hex(code_id)?,
                    msg: serde_json::to_vec(&value)?,
                    label: label.clone(),
                    funds: Amount::from_atto_str(funds_atto)?,
                },
            )
        }
        ContractCommand::Execute {
            from,
            contract,
            msg,
            funds_atto,
        } => {
            let value: Value = serde_json::from_str(msg)?;
            (
                from,
                TxMessage::ContractCall {
                    contract: Address::parse(contract)?,
                    data: serde_json::to_vec(&value)?,
                    funds: Amount::from_atto_str(funds_atto)?,
                },
            )
        }
        ContractCommand::Query { .. } => unreachable!("query handled above"),
    };
    let keyring = Keyring::open_or_create_with_development_keys(&args.home.join("keyring"), false)?;
    let key = keyring.get_ed25519_keypair(from)?;
    let status = rpc(&args.rpc_url, "sprax_getStatus", json!([]))?;
    let account = rpc(
        &args.rpc_url,
        "sprax_getAccount",
        json!([key.address().to_hex()]),
    )?;
    let chain_id = status["chainId"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("RPC status omitted chain ID"))?;
    let height = status["height"]
        .as_u64()
        .ok_or_else(|| anyhow::anyhow!("RPC status omitted height"))?;
    let nonce = account["nonce"]
        .as_u64()
        .ok_or_else(|| anyhow::anyhow!("RPC account omitted nonce"))?;
    let body = TxBody {
        chain_id: ChainId::new(chain_id)?,
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
    let transaction = Transaction::new(
        body,
        KeyType::Ed25519,
        key.public_key_bytes().to_vec(),
        signature,
    )?;
    let result = rpc(&args.rpc_url, "sprax_broadcastTx", json!([transaction]))?;
    println!("Submitted to mempool: {}", result["txHash"]);
    println!(
        "Wait for the transaction receipt before using returned code IDs or contract addresses."
    );
    Ok(())
}
