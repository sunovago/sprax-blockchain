use clap::Args;
use sprax_crypto::Hasher;
use sprax_node::{init_telemetry, NodeService};
use std::path::PathBuf;
use tracing::info;

#[derive(Debug, Args)]
pub(crate) struct StartArgs {
    /// Node home directory
    #[arg(long, default_value = ".sprx")]
    pub(crate) home: PathBuf,

    /// P2P listening port override (e.g. 26656, 26666, 26676)
    #[arg(long)]
    pub(crate) p2p_port: Option<u16>,

    /// RPC listening port override (e.g. 26657, 26667, 26677)
    #[arg(long)]
    pub(crate) rpc_port: Option<u16>,

    /// Comma-separated bootstrap peer addresses to dial (e.g. 127.0.0.1:26656,127.0.0.1:26666)
    #[arg(long)]
    pub(crate) peers: Option<String>,
}

pub(crate) async fn execute(args: &StartArgs) -> anyhow::Result<()> {
    let mut service = NodeService::new_or_load(args.home.clone())?;
    if let Some(peers) = &args.peers {
        let bootstrap_peers: Vec<String> = peers
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .collect();
        service.override_bootstrap_peers(bootstrap_peers);
    }
    init_telemetry(&service.config().telemetry);

    let latest_header = service.latest_header();
    let block_hash = Hasher::block_hash(&latest_header)?;

    let p2p_port = args.p2p_port.unwrap_or(service.config().network.p2p_port);
    let rpc_port = args.rpc_port.unwrap_or(service.config().rpc.json_rpc_port);

    println!("============================================================");
    println!("  SPRX MULTI-NODE BLOCKCHAIN DAEMON ONLINE");
    println!("============================================================");
    println!("  Chain ID        : {}", service.chain_id());
    println!("  Current Height  : #{}", service.height());
    println!("  Latest StateRoot: {}", latest_header.state_root);
    println!("  Latest BlockHash: {}", block_hash);
    println!("  P2P Port        : {p2p_port}");
    println!("  RPC Port        : {rpc_port}");
    if let Some(peers) = &args.peers {
        println!("  Bootstrap Peers : {peers}");
    }
    println!("============================================================");
    println!("  Local node running. Press Ctrl+C to terminate cleanly.");

    info!(
        chain_id = %service.chain_id(),
        height = service.height(),
        home = ?args.home,
        p2p_port = p2p_port,
        "SPRX Node daemon started"
    );

    service.start().await?;

    // Wait for SIGINT / Ctrl+C
    tokio::signal::ctrl_c().await?;
    info!("Shutdown signal received. Saving state snapshot...");
    service.stop().await?;
    println!("\nSPRX Local Blockchain Daemon stopped cleanly.");

    Ok(())
}
