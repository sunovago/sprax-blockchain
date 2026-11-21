use clap::{Args, Subcommand};
use sprax_faucet::{FaucetServer, FaucetService};
use sprax_node::NodeService;
use sprax_types::ChainId;
use std::{path::PathBuf, sync::Arc};

#[derive(Debug, Args)]
pub(crate) struct FaucetArgs {
    #[command(subcommand)]
    pub(crate) command: FaucetSubcommands,
}

#[derive(Debug, Subcommand)]
pub(crate) enum FaucetSubcommands {
    /// Start the automated testnet faucet HTTP daemon
    Start {
        /// HTTP server listening port (default: 8080)
        #[arg(long, default_value = "8080")]
        port: u16,

        /// Faucet operator key name in keyring (e.g. alice, faucet)
        #[arg(long, default_value = "alice")]
        key: String,

        /// Max payout in tSPRX per request (default: 100)
        #[arg(long, default_value = "100")]
        payout: u64,

        /// Rate limit cooldown per address/IP in seconds (default: 86400 / 24 hours)
        #[arg(long, default_value = "86400")]
        rate_limit: u64,

        /// Node home directory
        #[arg(long, default_value = ".sprx")]
        home: PathBuf,
    },

    /// Request testnet tokens from a running faucet service
    Request {
        /// Recipient address (Bech32 sprax1... or Hex 0x...)
        #[arg(long)]
        to: String,

        /// Amount in tSPRX (optional, default: 100)
        #[arg(long)]
        amount: Option<u64>,

        /// Faucet HTTP server URL
        #[arg(long, default_value = "http://127.0.0.1:8080")]
        faucet_url: String,
    },

    /// Query stats and balance of a running faucet service
    Status {
        /// Faucet HTTP server URL
        #[arg(long, default_value = "http://127.0.0.1:8080")]
        faucet_url: String,
    },
}

pub(crate) async fn execute(args: &FaucetArgs) -> anyhow::Result<()> {
    match &args.command {
        FaucetSubcommands::Start {
            port,
            key,
            payout,
            rate_limit,
            home,
        } => {
            let node = NodeService::new_or_load(home.clone())?;
            let faucet_kp = {
                let keyring = node.keyring();
                let keyring_guard = keyring.read();
                keyring_guard.get_ed25519_keypair(key).map_err(|e| {
                    anyhow::anyhow!("faucet operator key '{key}' not found in keyring: {e}")
                })?
            };

            let chain_id_str = node.chain_id();
            let chain_id = ChainId::new(&chain_id_str)
                .map_err(|e| anyhow::anyhow!("invalid chain id: {e}"))?;

            let faucet_addr = faucet_kp.address();
            let faucet = Arc::new(FaucetService::new(
                faucet_kp,
                chain_id,
                *rate_limit,
                *payout,
            ));

            let ledger = node.ledger();

            println!("============================================================");
            println!("  SPRX PUBLIC TESTNET FAUCET DAEMON");
            println!("============================================================");
            println!("  Chain ID         : {chain_id_str}");
            println!("  Faucet Address   : {faucet_addr}");
            let faucet_hex = faucet_addr.to_hex();
            println!("  Faucet Hex       : {faucet_hex}");
            println!("  Max Payout       : {payout} tSPRX");
            let window_hours = rate_limit / 3600;
            println!("  Rate Limit Window: {rate_limit}s ({window_hours} hours)");
            println!("  HTTP Endpoint    : http://0.0.0.0:{port}");
            println!("============================================================");
            println!("  Faucet service running. Press Ctrl+C to terminate cleanly.");

            FaucetServer::run(faucet, ledger, *port)
                .await
                .map_err(|e| anyhow::anyhow!("faucet server failed: {e}"))?;

            Ok(())
        }

        FaucetSubcommands::Request {
            to,
            amount,
            faucet_url,
        } => {
            let endpoint = format!("{}/claim", faucet_url.trim_end_matches('/'));
            let payload = serde_json::json!({
                "recipient": to,
                "amount_sprx": amount,
            });

            println!("Submitting faucet claim request to {endpoint} for recipient {to}...");

            let response = ureq::post(&endpoint)
                .set("Content-Type", "application/json")
                .send_json(payload);

            match response {
                Ok(resp) => {
                    let body: serde_json::Value = resp.into_json()?;
                    println!("============================================================");
                    println!("  FAUCET CLAIM SUCCESSFUL");
                    println!("============================================================");
                    println!(
                        "  Recipient     : {}",
                        body.get("recipient").unwrap_or(&serde_json::Value::Null)
                    );
                    println!(
                        "  Amount        : {}",
                        body.get("amount_sprx").unwrap_or(&serde_json::Value::Null)
                    );
                    println!(
                        "  Tx Hash       : {}",
                        body.get("tx_hash").unwrap_or(&serde_json::Value::Null)
                    );
                    println!(
                        "  Notice        : {}",
                        body.get("network_notice")
                            .unwrap_or(&serde_json::Value::Null)
                    );
                    println!("============================================================");
                    Ok(())
                }
                Err(ureq::Error::Status(code, resp)) => {
                    let err_body: serde_json::Value = resp.into_json().unwrap_or_default();
                    let msg = err_body
                        .get("error")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown error");
                    anyhow::bail!("Faucet request failed (HTTP {code}): {msg}");
                }
                Err(e) => anyhow::bail!("Failed to connect to faucet server at {endpoint}: {e}"),
            }
        }

        FaucetSubcommands::Status { faucet_url } => {
            let endpoint = format!("{}/stats", faucet_url.trim_end_matches('/'));
            let resp = ureq::get(&endpoint).call();

            match resp {
                Ok(r) => {
                    let stats: sprax_faucet::FaucetStats = r.into_json()?;
                    println!("============================================================");
                    println!("  SPRX FAUCET POOL STATUS");
                    println!("============================================================");
                    println!("  Network        : {}", stats.network);
                    println!("  Faucet Address : {}", stats.faucet_address);
                    println!("  Balance        : {}", stats.available_balance);
                    println!("  Total Disbursed: {}", stats.total_disbursed);
                    println!("  Total Claims   : {}", stats.total_claims_count);
                    println!("  Max Payout     : {}", stats.max_payout_per_request);
                    println!("  Rate Limit     : {}s", stats.rate_limit_window_secs);
                    println!("  Disclaimer     : {}", stats.disclaimer);
                    println!("============================================================");
                    Ok(())
                }
                Err(e) => anyhow::bail!("Failed to query faucet stats from {endpoint}: {e}"),
            }
        }
    }
}
