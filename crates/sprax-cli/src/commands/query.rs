use clap::{Args, Subcommand};
use sprax_crypto::Hasher;
use sprax_node::NodeService;
use sprax_types::{Address, Hash32, TxMessage};
use std::path::PathBuf;

#[derive(Debug, Args)]
pub(crate) struct QueryArgs {
    #[command(subcommand)]
    pub(crate) command: QuerySubcommands,
}

#[derive(Debug, Subcommand)]
pub(crate) enum QuerySubcommands {
    /// Query balance and account state for an address or named key
    Balance {
        /// Account address (Bech32 sprax1... or Hex 0x...) or key name
        target: String,

        /// Node home directory
        #[arg(long, default_value = ".sprx")]
        home: PathBuf,
    },
    /// Query block by height number or 32-byte hex hash
    Block {
        /// Block height (e.g. 0, 1, 42) or hex hash (0x...)
        identifier: String,

        /// Node home directory
        #[arg(long, default_value = ".sprx")]
        home: PathBuf,
    },
    /// Query transaction details and execution receipt by hash
    Tx {
        /// Transaction hash (0x...)
        tx_hash: String,

        /// Node home directory
        #[arg(long, default_value = ".sprx")]
        home: PathBuf,
    },
    /// Query local blockchain status and latest block metadata
    Status {
        /// Node home directory
        #[arg(long, default_value = ".sprx")]
        home: PathBuf,
    },
    /// Query live node operational metrics and peer synchronization status
    Metrics {
        /// Node home directory
        #[arg(long, default_value = ".sprx")]
        home: PathBuf,
    },
    /// Query connected P2P network peers
    Peers {
        /// Node home directory
        #[arg(long, default_value = ".sprx")]
        home: PathBuf,
    },
}

pub(crate) fn execute(args: &QueryArgs) -> anyhow::Result<()> {
    match &args.command {
        QuerySubcommands::Balance { target, home } => {
            let node = NodeService::new_or_load(home.clone())?;
            let keyring = node.keyring();
            let keyring_guard = keyring.read();

            let (address, name_label) = if let Ok(addr) = Address::parse(target) {
                (addr, None)
            } else if let Some(rec) = keyring_guard.get(target) {
                (rec.address, Some(rec.name.clone()))
            } else {
                return Err(anyhow::anyhow!(
                    "invalid address or unknown key name: '{target}'"
                ));
            };

            drop(keyring_guard);

            let account = node.get_account(&address)?;
            let whole_sprx = (account.balance.as_atto() as f64) / 1_000_000_000_000_000_000.0;

            let usd_val = whole_sprx * 4.50;
            let inr_val = whole_sprx * 375.00;
            let eur_val = whole_sprx * 4.15;
            let jpy_val = whole_sprx * 650.00;

            println!("============================================================");
            println!("  ACCOUNT BALANCE QUERY");
            println!("============================================================");
            if let Some(name) = name_label {
                println!("  Key Name       : {name}");
            }
            println!("  Address (Bech32): {}", address);
            println!("  Address (Hex)   : {}", address.to_hex());
            println!("  Nonce Sequence : {}", account.nonce);
            println!("------------------------------------------------------------");
            println!("  Native Balance  : {}", account.balance);
            println!(
                "  Atomic Base Unit: {} atto-SPRX",
                account.balance.as_atto()
            );
            println!("------------------------------------------------------------");
            println!("  Global Currency Display (Reference Feeds):");
            println!("  - USD ($)       : ${:.2}", usd_val);
            println!("  - INR (₹)       : ₹{:.2}", inr_val);
            println!("  - EUR (€)       : €{:.2}", eur_val);
            println!("  - JPY (¥)       : ¥{:.0}", jpy_val);
            println!("============================================================");
        }
        QuerySubcommands::Block { identifier, home } => {
            let node = NodeService::new_or_load(home.clone())?;

            let block_opt = if let Ok(height) = identifier.parse::<u64>() {
                node.get_block_by_height(height)
            } else if let Ok(hash) = Hash32::from_hex(identifier) {
                node.get_block_by_hash(&hash)
            } else {
                return Err(anyhow::anyhow!(
                    "invalid block identifier '{identifier}' (expected integer height or hex hash)"
                ));
            };

            if let Some(block) = block_opt {
                let block_hash = Hasher::block_hash(&block.header)?;
                println!("============================================================");
                println!("  BLOCK #{} DETAILS", block.header.height);
                println!("============================================================");
                println!("  Block Hash      : {}", block_hash);
                println!("  Parent Hash     : {}", block.header.parent_hash);
                println!("  Chain ID        : {}", block.header.chain_id);
                println!("  Height          : {}", block.header.height);
                println!("  Timestamp (Unix): {}", block.header.timestamp_unix_secs);
                println!("  Proposer        : {}", block.header.proposer);
                println!(
                    "  Transactions    : {} included",
                    block.body.transactions.len()
                );
                println!("  Txs Root        : {}", block.header.txs_root);
                println!("  State Root      : {}", block.header.state_root);
                println!("------------------------------------------------------------");
                if !block.body.transactions.is_empty() {
                    for (i, tx) in block.body.transactions.iter().enumerate() {
                        let tx_hash = Hasher::tx_hash(tx)?;
                        println!("  Tx [{}]: {} (Sender: {})", i, tx_hash, tx.body.sender);
                    }
                }
                println!("============================================================");
            } else {
                println!("Block '{}' not found on chain.", identifier);
            }
        }
        QuerySubcommands::Tx { tx_hash, home } => {
            let node = NodeService::new_or_load(home.clone())?;
            let hash = Hash32::from_hex(tx_hash)
                .map_err(|e| anyhow::anyhow!("invalid tx hash format: {e}"))?;

            if let Some((tx, receipt, height)) = node.get_transaction(&hash) {
                println!("============================================================");
                println!("  TRANSACTION DETAILS");
                println!("============================================================");
                println!("  Tx Hash         : {}", hash);
                println!("  Block Height    : #{}", height);
                println!(
                    "  Execution Status: {}",
                    if receipt.success { "SUCCESS" } else { "FAILED" }
                );
                println!("  Gas Used        : {}", receipt.gas_used);
                println!("  Sender          : {}", tx.body.sender);
                println!("  Nonce           : {}", tx.body.nonce);
                println!("  Fee Paid        : {}", tx.body.fee.amount);
                println!("  Memo            : '{}'", tx.body.memo);
                println!("------------------------------------------------------------");
                println!("  Messages (Total: {}):", tx.body.messages.len());
                for (idx, msg) in tx.body.messages.iter().enumerate() {
                    match msg {
                        TxMessage::Transfer { to, amount } => {
                            println!(
                                "    [{idx}] Transfer -> Recipient: {}, Amount: {}",
                                to, amount
                            );
                        }
                        _ => {
                            println!("    [{idx}] Custom Operation: {:?}", msg);
                        }
                    }
                }
                println!("============================================================");
            } else {
                println!("Transaction '{}' not found in ledger.", tx_hash);
            }
        }
        QuerySubcommands::Status { home } => {
            let node = NodeService::new_or_load(home.clone())?;
            let header = node.latest_header();
            let latest_hash = Hasher::block_hash(&header)?;

            println!("============================================================");
            println!("  SPRX LOCAL BLOCKCHAIN STATUS");
            println!("============================================================");
            println!("  Home Directory  : {:?}", home);
            println!("  Chain ID        : {}", node.chain_id());
            println!("  Current Height  : #{}", node.height());
            println!("  Latest Block Hash: {}", latest_hash);
            println!("  State Root      : {}", header.state_root);
            println!("  P2P Port        : {}", node.config().network.p2p_port);
            println!("  RPC Port        : {}", node.config().rpc.json_rpc_port);
            println!("============================================================");
        }
        QuerySubcommands::Metrics { home } => {
            let node = NodeService::new_or_load(home.clone())?;
            let metrics = node.metrics();

            println!("============================================================");
            println!("  SPRX NODE DEVELOPMENT METRICS");
            println!("============================================================");
            println!("  Chain ID           : {}", metrics.chain_id);
            println!("  Node Status        : {}", metrics.status);
            println!("  Current Height     : #{}", metrics.height);
            println!("  Latest Block Hash  : {}", metrics.latest_block_hash);
            println!("  State Root         : {}", metrics.state_root);
            println!("  Connected Peers    : {}", metrics.connected_peers);
            println!("  Total Transactions : {}", metrics.total_transactions);
            println!("  Mempool Pending    : {}", metrics.mempool_pending);
            println!(
                "  Syncing Status     : {}",
                if metrics.is_syncing {
                    "IN_PROGRESS"
                } else {
                    "SYNCHRONIZED"
                }
            );
            println!("============================================================");
        }
        QuerySubcommands::Peers { home } => {
            let node = NodeService::new_or_load(home.clone())?;
            let peers = node.connected_peers();

            println!("============================================================");
            println!("  CONNECTED P2P PEERS (Total: {})", peers.len());
            println!("============================================================");
            if peers.is_empty() {
                println!("  No external peers currently connected.");
            } else {
                for (i, peer) in peers.iter().enumerate() {
                    println!(
                        "  [{}] PeerID: {} | Height: #{} | Score: {:.1}",
                        i, peer.peer_id, peer.height, peer.score.score
                    );
                }
            }
            println!("============================================================");
        }
    }
    Ok(())
}
