use clap::{Args, Subcommand};
use sprax_node::NodeService;
use sprax_types::{Address, Amount, ChainId, KeyType, Transaction, TxBody, TxFee, TxMessage};
use std::path::PathBuf;

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

        /// Node home directory
        #[arg(long, default_value = ".sprx")]
        home: PathBuf,
    },
}

pub(crate) fn execute(args: &TxArgs) -> anyhow::Result<()> {
    match &args.command {
        TxSubcommands::Send {
            from,
            to,
            amount,
            fee,
            memo,
            home,
        } => {
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
            println!("  Tx Hash          : {}", tx_hash);
            println!("  Sender           : {} ({})", sender_addr, from);
            println!("  Recipient        : {} ({})", recipient_addr, to);
            println!("  Amount Sent      : {}", transfer_amount);
            println!("  Fee Paid         : {}", fee_amount);
            println!("  Tx Nonce         : {}", current_nonce);

            if node.config().consensus.enabled {
                // Multi-validator mode: leave the tx in the mempool for `ConsensusDriver` to
                // pick up on its own schedule via a real BFT round, rather than bypassing
                // proposer selection with an instant single-signer commit.
                println!("------------------------------------------------------------");
                println!("  Consensus mode active: awaiting inclusion via BFT proposer round.");
                println!("============================================================");
            } else {
                // Legacy single-node dev convenience: commit immediately (unchanged default
                // behavior for every existing single-node config/test).
                let block = node.mine_block(sender_addr)?;
                let sender_after = node.get_account(&sender_addr)?;
                let recipient_after = node.get_account(&recipient_addr)?;

                println!("  Included Block   : #{}", block.header.height);
                println!("------------------------------------------------------------");
                println!("  Updated Balances:");
                println!("  - Sender ({})    : {}", from, sender_after.balance);
                println!("  - Recipient ({}) : {}", to, recipient_after.balance);
                println!("============================================================");
            }
        }
    }
    Ok(())
}
