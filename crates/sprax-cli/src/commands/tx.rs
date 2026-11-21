use clap::{Args, Subcommand};
use serde_json::{json, Value};
use sprax_node::{Keyring, NodeService};
use sprax_types::{Address, Amount, ChainId, KeyType, Transaction, TxBody, TxFee, TxMessage};
use std::{path::PathBuf, time::Duration};

#[derive(Debug, Args)]
pub(crate) struct TxArgs {
    #[command(subcommand)]
    pub(crate) command: TxSubcommands,
}

#[derive(Debug, Subcommand)]
pub(crate) enum TxSubcommands {
    /// Send native SPRX tokens from one account to another
    Send {
        /// Sender key name in local keyring (e.g. alice, bob)
        #[arg(long)]
        from: String,

        /// Recipient address (Bech32 sprax1... or Hex 0x...) or key name
        #[arg(long)]
        to: String,

        /// Amount in SPRX whole units (e.g. 10, 50.5)
        #[arg(long)]
        amount: f64,

        /// Fee in SPRX (optional, default: 0.0005 SPRX)
        #[arg(long, default_value = "0.0005")]
        fee: f64,

        /// Optional transaction memo
        #[arg(long, default_value = "")]
        memo: String,

        /// Target node RPC URL (optional, defaults to http://127.0.0.1:26657)
        #[arg(long, default_value = "http://127.0.0.1:26657")]
        rpc_url: String,

        /// Node home directory
        #[arg(long, default_value = ".sprx")]
        home: PathBuf,
    },
    /// Register a new validator candidate with proof of possession
    RegisterValidator {
        /// Key name in local keyring (operator account)
        #[arg(long)]
        from: String,

        /// Consensus Ed25519 public key (64 hex characters / 32 bytes)
        #[arg(long)]
        consensus_pubkey: String,

        /// Validator moniker / human readable display name
        #[arg(long)]
        moniker: String,

        /// Self-stake amount in whole SPRX units (minimum: 1000 SPRX)
        #[arg(long, default_value = "1000.0")]
        self_stake: f64,

        /// Proof of possession Ed25519 signature (128 hex characters / 64 bytes)
        #[arg(long)]
        proof_of_possession: String,

        /// Fee in SPRX (optional, default: 0.001 SPRX)
        #[arg(long, default_value = "0.001")]
        fee: f64,

        /// Optional transaction memo
        #[arg(long, default_value = "")]
        memo: String,

        /// Target node RPC URL
        #[arg(long, default_value = "http://127.0.0.1:26657")]
        rpc_url: String,

        /// Node home directory
        #[arg(long, default_value = ".sprx")]
        home: PathBuf,
    },
    /// Unjail a validator after cooldown period has elapsed
    Unjail {
        /// Key name in local keyring (operator account)
        #[arg(long)]
        from: String,

        /// Fee in SPRX (optional, default: 0.001 SPRX)
        #[arg(long, default_value = "0.001")]
        fee: f64,

        /// Optional transaction memo
        #[arg(long, default_value = "")]
        memo: String,

        /// Target node RPC URL
        #[arg(long, default_value = "http://127.0.0.1:26657")]
        rpc_url: String,

        /// Node home directory
        #[arg(long, default_value = ".sprx")]
        home: PathBuf,
    },
    /// Submit finalized double-sign equivocation evidence
    SubmitEvidence {
        /// Key name in local keyring (reporter account)
        #[arg(long)]
        from: String,

        /// Path to JSON file containing EquivocationEvidence
        #[arg(long)]
        evidence: PathBuf,

        /// Fee in SPRX (optional, default: 0.001 SPRX)
        #[arg(long, default_value = "0.001")]
        fee: f64,

        /// Optional transaction memo
        #[arg(long, default_value = "")]
        memo: String,

        /// Target node RPC URL
        #[arg(long, default_value = "http://127.0.0.1:26657")]
        rpc_url: String,

        /// Node home directory
        #[arg(long, default_value = ".sprx")]
        home: PathBuf,
    },
}

fn rpc_call(url: &str, method: &str, params: Value) -> anyhow::Result<Value> {
    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(10))
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

pub(crate) fn execute(args: &TxArgs) -> anyhow::Result<()> {
    match &args.command {
        TxSubcommands::Send {
            from,
            to,
            amount,
            fee,
            memo,
            rpc_url,
            home,
        } => {
            // Check if RPC node is available
            let rpc_status = rpc_call(rpc_url, "sprax_getStatus", json!([]));
            if let Ok(status) = rpc_status {
                let keyring =
                    Keyring::open_or_create_with_development_keys(&home.join("keyring"), false)?;
                let sender_kp = keyring.get_ed25519_keypair(from)?;
                let sender_addr = sender_kp.address();

                let recipient_addr = if let Ok(addr) = Address::parse(to) {
                    addr
                } else if let Some(rec) = keyring.get(to) {
                    rec.address
                } else {
                    return Err(anyhow::anyhow!(
                        "invalid recipient address or key name: '{to}'"
                    ));
                };

                let account = rpc_call(rpc_url, "sprax_getAccount", json!([sender_addr.to_hex()]))?;
                let current_nonce = account["nonce"]
                    .as_u64()
                    .ok_or_else(|| anyhow::anyhow!("failed to read account nonce from RPC"))?;

                let chain_id_str = status["chainId"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("missing chainId from RPC"))?;
                let chain_id = ChainId::new(chain_id_str)?;

                let height = status["height"]
                    .as_u64()
                    .or_else(|| status["latestBlockHeight"].as_u64())
                    .unwrap_or(0);

                let transfer_atto = (*amount * 1_000_000_000_000_000_000.0) as u128;
                let transfer_amount = Amount::from_atto(transfer_atto);

                let fee_atto = (*fee * 1_000_000_000_000_000_000.0) as u128;
                let fee_amount = Amount::from_atto(fee_atto);

                let tx_body = TxBody {
                    chain_id,
                    sender: sender_addr,
                    nonce: current_nonce,
                    messages: vec![TxMessage::Transfer {
                        to: recipient_addr,
                        amount: transfer_amount,
                    }],
                    fee: TxFee {
                        amount: fee_amount,
                        gas_limit: 200_000,
                        priority_fee: Amount::ZERO,
                    },
                    memo: memo.clone(),
                    timeout_height: height + 100,
                };

                let sign_bytes = tx_body.sign_bytes()?;
                let signature = sender_kp.sign(&sign_bytes);

                let tx = Transaction::new(
                    tx_body,
                    KeyType::Ed25519,
                    sender_kp.public_key_bytes().to_vec(),
                    signature,
                )?;

                let res = rpc_call(rpc_url, "sprax_broadcastTx", json!([tx]))?;
                let tx_hash = res["txHash"]
                    .as_str()
                    .or_else(|| res["tx_hash"].as_str())
                    .unwrap_or("");

                println!("============================================================");
                println!("  TRANSACTION SUBMITTED VIA RPC");
                println!("============================================================");
                println!("  RPC Node         : {rpc_url}");
                println!("  Tx Hash          : {tx_hash}");
                println!("  Sender           : {sender_addr} ({from})");
                println!("  Recipient        : {recipient_addr} ({to})");
                println!("  Amount Sent      : {transfer_amount}");
                println!("  Fee Paid         : {fee_amount}");
                println!("  Tx Nonce         : {current_nonce}");
                println!("============================================================");

                // Wait for receipt confirmation
                let agent = ureq::AgentBuilder::new()
                    .timeout(Duration::from_secs(5))
                    .build();
                let receipt_url = format!("{}/txs/{}", rpc_url.trim_end_matches('/'), tx_hash);
                let deadline = std::time::Instant::now() + Duration::from_secs(10);
                while std::time::Instant::now() < deadline {
                    std::thread::sleep(Duration::from_millis(500));
                    if let Ok(resp) = agent.get(&receipt_url).call() {
                        if let Ok(tx_info) = resp.into_json::<Value>() {
                            if let Some(receipt) = tx_info.get("receipt") {
                                let success = receipt["success"].as_bool().unwrap_or(false);
                                if success {
                                    println!(
                                        "✓ Transaction finalized in block #{}!",
                                        tx_info["height"]
                                    );
                                    return Ok(());
                                }
                            }
                        }
                    }
                }
                println!("✓ Transaction in mempool, awaiting block commit.");
                return Ok(());
            }

            // Fallback: offline direct NodeService execution
            let node = NodeService::new_or_load(home.clone())?;
            let keyring = node.keyring();
            let keyring_guard = keyring.read();

            // Resolve Sender Keypair
            let sender_kp = keyring_guard
                .get_ed25519_keypair(from)
                .map_err(|e| anyhow::anyhow!("failed to get sender key '{from}': {e}"))?;
            let sender_addr = sender_kp.address();

            // Resolve Recipient Address (either Bech32/Hex or key name in keyring)
            let recipient_addr = if let Ok(addr) = Address::parse(to) {
                addr
            } else if let Some(rec) = keyring_guard.get(to) {
                rec.address
            } else {
                return Err(anyhow::anyhow!(
                    "invalid recipient address or key name: '{to}'"
                ));
            };

            drop(keyring_guard);

            // Convert amount and fee to base atomic units (atto-SPRX)
            let transfer_atto = (*amount * 1_000_000_000_000_000_000.0) as u128;
            let transfer_amount = Amount::from_atto(transfer_atto);

            let fee_atto = (*fee * 1_000_000_000_000_000_000.0) as u128;
            let fee_amount = Amount::from_atto(fee_atto);

            // Fetch sender current nonce
            let sender_account = node.get_account(&sender_addr)?;
            let current_nonce = sender_account.nonce;

            let chain_id_str = node.chain_id();
            let chain_id = ChainId::new(&chain_id_str)
                .map_err(|e| anyhow::anyhow!("invalid chain id: {e}"))?;

            let tx_body = TxBody {
                chain_id,
                sender: sender_addr,
                nonce: current_nonce,
                messages: vec![TxMessage::Transfer {
                    to: recipient_addr,
                    amount: transfer_amount,
                }],
                fee: TxFee {
                    amount: fee_amount,
                    gas_limit: 200_000,
                    priority_fee: Amount::ZERO,
                },
                memo: memo.clone(),
                timeout_height: node.height() + 100,
            };

            let sign_bytes = tx_body
                .sign_bytes()
                .map_err(|e| anyhow::anyhow!("failed to serialize sign bytes: {e}"))?;
            let signature = sender_kp.sign(&sign_bytes);

            let tx = Transaction::new(
                tx_body,
                KeyType::Ed25519,
                sender_kp.public_key_bytes().to_vec(),
                signature,
            )
            .map_err(|e| anyhow::anyhow!("failed to create transaction: {e}"))?;

            // Submit transaction to local node mempool
            let tx_hash = node.submit_transaction(tx)?;

            println!("============================================================");
            println!("  TRANSACTION SUBMITTED");
            println!("============================================================");
            println!("  Tx Hash          : {tx_hash}");
            println!("  Sender           : {sender_addr} ({from})");
            println!("  Recipient        : {recipient_addr} ({to})");
            println!("  Amount Sent      : {transfer_amount}");
            println!("  Fee Paid         : {fee_amount}");
            println!("  Tx Nonce         : {current_nonce}");

            if node.config().consensus.enabled {
                println!("------------------------------------------------------------");
                println!("  Consensus mode active: awaiting inclusion via BFT proposer round.");
                println!("============================================================");
            } else {
                let block = node.mine_block(sender_addr)?;
                let sender_after = node.get_account(&sender_addr)?;
                let recipient_after = node.get_account(&recipient_addr)?;

                println!("  Included Block   : #{}", block.header.height);
                println!("------------------------------------------------------------");
                println!("  Updated Balances:");
                println!("  - Sender ({from})    : {}", sender_after.balance);
                println!("  - Recipient ({to}) : {}", recipient_after.balance);
                println!("============================================================");
            }
        }
        TxSubcommands::RegisterValidator {
            from,
            consensus_pubkey,
            moniker,
            self_stake,
            proof_of_possession,
            fee,
            memo,
            rpc_url: _,
            home,
        } => {
            let pubkey_bytes = hex::decode(consensus_pubkey)
                .map_err(|e| anyhow::anyhow!("invalid consensus_pubkey hex: {e}"))?;
            let pop_bytes = hex::decode(proof_of_possession)
                .map_err(|e| anyhow::anyhow!("invalid proof_of_possession hex: {e}"))?;

            let node = NodeService::new_or_load(home.clone())?;
            let keyring = node.keyring();
            let keyring_guard = keyring.read();
            let sender_kp = keyring_guard
                .get_ed25519_keypair(from)
                .map_err(|e| anyhow::anyhow!("failed to get sender key '{from}': {e}"))?;
            let sender_addr = sender_kp.address();
            drop(keyring_guard);

            let stake_atto = (*self_stake * 1_000_000_000_000_000_000.0) as u128;
            let stake_amount = Amount::from_atto(stake_atto);
            let fee_atto = (*fee * 1_000_000_000_000_000_000.0) as u128;
            let fee_amount = Amount::from_atto(fee_atto);

            let sender_account = node.get_account(&sender_addr)?;
            let current_nonce = sender_account.nonce;
            let chain_id = ChainId::new(node.chain_id())?;

            let tx_body = TxBody {
                chain_id,
                sender: sender_addr,
                nonce: current_nonce,
                messages: vec![TxMessage::RegisterValidator {
                    consensus_pubkey: pubkey_bytes,
                    moniker: moniker.clone(),
                    self_stake: stake_amount,
                    proof_of_possession: pop_bytes,
                }],
                fee: TxFee {
                    amount: fee_amount,
                    gas_limit: 300_000,
                    priority_fee: Amount::ZERO,
                },
                memo: memo.clone(),
                timeout_height: node.height() + 100,
            };

            let sign_bytes = tx_body.sign_bytes()?;
            let signature = sender_kp.sign(&sign_bytes);
            let tx = Transaction::new(
                tx_body,
                KeyType::Ed25519,
                sender_kp.public_key_bytes().to_vec(),
                signature,
            )?;

            let tx_hash = node.submit_transaction(tx)?;
            println!("============================================================");
            println!("  VALIDATOR REGISTRATION SUBMITTED");
            println!("============================================================");
            println!("  Tx Hash          : {tx_hash}");
            println!("  Operator         : {sender_addr} ({from})");
            println!("  Moniker          : {moniker}");
            println!("  Self Stake       : {stake_amount}");
            println!("  Fee Paid         : {fee_amount}");
            println!("============================================================");
        }
        TxSubcommands::Unjail {
            from,
            fee,
            memo,
            rpc_url: _,
            home,
        } => {
            let node = NodeService::new_or_load(home.clone())?;
            let keyring = node.keyring();
            let keyring_guard = keyring.read();
            let sender_kp = keyring_guard
                .get_ed25519_keypair(from)
                .map_err(|e| anyhow::anyhow!("failed to get sender key '{from}': {e}"))?;
            let sender_addr = sender_kp.address();
            drop(keyring_guard);

            let fee_atto = (*fee * 1_000_000_000_000_000_000.0) as u128;
            let fee_amount = Amount::from_atto(fee_atto);

            let sender_account = node.get_account(&sender_addr)?;
            let current_nonce = sender_account.nonce;
            let chain_id = ChainId::new(node.chain_id())?;

            let tx_body = TxBody {
                chain_id,
                sender: sender_addr,
                nonce: current_nonce,
                messages: vec![TxMessage::Unjail],
                fee: TxFee {
                    amount: fee_amount,
                    gas_limit: 100_000,
                    priority_fee: Amount::ZERO,
                },
                memo: memo.clone(),
                timeout_height: node.height() + 100,
            };

            let sign_bytes = tx_body.sign_bytes()?;
            let signature = sender_kp.sign(&sign_bytes);
            let tx = Transaction::new(
                tx_body,
                KeyType::Ed25519,
                sender_kp.public_key_bytes().to_vec(),
                signature,
            )?;

            let tx_hash = node.submit_transaction(tx)?;
            println!("============================================================");
            println!("  UNJAIL TRANSACTION SUBMITTED");
            println!("============================================================");
            println!("  Tx Hash          : {tx_hash}");
            println!("  Operator         : {sender_addr} ({from})");
            println!("  Fee Paid         : {fee_amount}");
            println!("============================================================");
        }
        TxSubcommands::SubmitEvidence {
            from,
            evidence,
            fee,
            memo,
            rpc_url: _,
            home,
        } => {
            let data = std::fs::read_to_string(evidence)
                .map_err(|e| anyhow::anyhow!("failed to read evidence file: {e}"))?;
            let ev: sprax_types::EquivocationEvidence = serde_json::from_str(&data)
                .map_err(|e| anyhow::anyhow!("failed to parse EquivocationEvidence JSON: {e}"))?;

            let node = NodeService::new_or_load(home.clone())?;
            let keyring = node.keyring();
            let keyring_guard = keyring.read();
            let sender_kp = keyring_guard
                .get_ed25519_keypair(from)
                .map_err(|e| anyhow::anyhow!("failed to get sender key '{from}': {e}"))?;
            let sender_addr = sender_kp.address();
            drop(keyring_guard);

            let fee_atto = (*fee * 1_000_000_000_000_000_000.0) as u128;
            let fee_amount = Amount::from_atto(fee_atto);

            let sender_account = node.get_account(&sender_addr)?;
            let current_nonce = sender_account.nonce;
            let chain_id = ChainId::new(node.chain_id())?;

            let tx_body = TxBody {
                chain_id,
                sender: sender_addr,
                nonce: current_nonce,
                messages: vec![TxMessage::SubmitEquivocationEvidence {
                    evidence: Box::new(ev),
                }],
                fee: TxFee {
                    amount: fee_amount,
                    gas_limit: 300_000,
                    priority_fee: Amount::ZERO,
                },
                memo: memo.clone(),
                timeout_height: node.height() + 100,
            };

            let sign_bytes = tx_body.sign_bytes()?;
            let signature = sender_kp.sign(&sign_bytes);
            let tx = Transaction::new(
                tx_body,
                KeyType::Ed25519,
                sender_kp.public_key_bytes().to_vec(),
                signature,
            )?;

            let tx_hash = node.submit_transaction(tx)?;
            println!("============================================================");
            println!("  EQUIVOCATION EVIDENCE SUBMITTED");
            println!("============================================================");
            println!("  Tx Hash          : {tx_hash}");
            println!("  Reporter         : {sender_addr} ({from})");
            println!("  Fee Paid         : {fee_amount}");
            println!("============================================================");
        }
    }
    Ok(())
}
