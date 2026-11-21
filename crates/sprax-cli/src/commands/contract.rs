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
        #[arg(long)]
        admin: Option<String>,
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
    Migrate {
        #[arg(long)]
        from: String,
        #[arg(long)]
        contract: String,
        #[arg(long)]
        new_code_id: String,
        #[arg(long)]
        msg: String,
    },
    UpdateAdmin {
        #[arg(long)]
        from: String,
        #[arg(long)]
        contract: String,
        #[arg(long)]
        new_admin: Option<String>,
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
        if let Ok(bytes) = serde_json::from_value::<Vec<u8>>(result["data"].clone()) {
            if let Ok(s) = String::from_utf8(bytes) {
                println!("{s}");
                return Ok(());
            }
        }
        println!("{}", serde_json::to_string_pretty(&result["data"])?);
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
            admin,
            funds_atto,
        } => {
            let value: Value = serde_json::from_str(msg)?;
            let admin_addr = admin.as_ref().map(|a| Address::parse(a)).transpose()?;
            (
                from,
                TxMessage::InstantiateContract {
                    code_id: Hash32::from_hex(code_id)?,
                    msg: serde_json::to_vec(&value)?,
                    label: label.clone(),
                    admin: admin_addr,
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
        ContractCommand::Migrate {
            from,
            contract,
            new_code_id,
            msg,
        } => {
            let value: Value = serde_json::from_str(msg)?;
            (
                from,
                TxMessage::MigrateContract {
                    contract: Address::parse(contract)?,
                    new_code_id: Hash32::from_hex(new_code_id)?,
                    msg: serde_json::to_vec(&value)?,
                },
            )
        }
        ContractCommand::UpdateAdmin {
            from,
            contract,
            new_admin,
        } => {
            let admin_addr = new_admin.as_ref().map(|a| Address::parse(a)).transpose()?;
            (
                from,
                TxMessage::UpdateAdmin {
                    contract: Address::parse(contract)?,
                    new_admin: admin_addr,
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
        .or_else(|| status["latestBlockHeight"].as_u64())
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
    let tx_hash = result["txHash"]
        .as_str()
        .or_else(|| result["tx_hash"].as_str())
        .unwrap_or("");
    println!("Submitted to mempool: {tx_hash}");

    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(5))
        .build();
    let receipt_url = format!("{}/txs/{}", args.rpc_url.trim_end_matches('/'), tx_hash);
    let deadline = std::time::Instant::now() + Duration::from_secs(12);
    while std::time::Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(500));
        if let Ok(resp) = agent.get(&receipt_url).call() {
            if let Ok(tx_info) = resp.into_json::<Value>() {
                if let Some(receipt) = tx_info.get("receipt") {
                    let success = receipt["success"].as_bool().unwrap_or(false);
                    let gas_used = receipt["gas_used"].as_u64().unwrap_or(0);
                    if success {
                        println!(
                            "✓ Transaction confirmed in block #{}! (Gas used: {})",
                            tx_info["height"], gas_used
                        );
                        if let Some(ret_hex) = receipt["return_data"].as_str() {
                            if let Ok(raw_bytes) = hex::decode(ret_hex) {
                                if let Ok(code_id) = serde_json::from_slice::<Hash32>(&raw_bytes) {
                                    println!("  Code ID: {}", code_id.to_hex());
                                } else if let Ok(contract_addr) =
                                    serde_json::from_slice::<Address>(&raw_bytes)
                                {
                                    println!(
                                        "  Contract Address: {}",
                                        contract_addr
                                            .to_bech32()
                                            .unwrap_or_else(|_| contract_addr.to_hex())
                                    );
                                }
                            }
                        }
                    } else {
                        println!("✗ Transaction execution failed!");
                    }
                    return Ok(());
                }
            }
        }
    }
    println!("Note: Transaction pending confirmation. Query status with: sprax query tx {tx_hash}");
    Ok(())
}
