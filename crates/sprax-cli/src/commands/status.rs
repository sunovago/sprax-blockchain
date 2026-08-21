use clap::Args;
use sprax_node::NodeConfig;
use std::path::PathBuf;

#[derive(Debug, Args)]
pub(crate) struct StatusArgs {
    /// Node home directory
    #[arg(long, default_value = ".sprx")]
    pub(crate) home: PathBuf,
}

pub(crate) fn execute(args: &StatusArgs) -> anyhow::Result<()> {
    let config_path = args.home.join("config.toml");
    if !config_path.exists() {
        println!("No SPRX node initialized at {:?}", args.home);
        println!(
            "Run 'sprax init --home {:?}' to create a new configuration.",
            args.home
        );
        return Ok(());
    }

    let config = NodeConfig::load_from_file(&config_path)?;
    println!("SPRX Node Status:");
    println!("  Home Directory : {:?}", config.home_dir);
    println!("  Chain ID       : {}", config.chain_id);
    println!("  Environment    : {}", config.environment);
    println!("  P2P Port       : {}", config.network.p2p_port);
    println!("  RPC Port       : {}", config.rpc.json_rpc_port);
    println!("  gRPC Port      : {}", config.rpc.grpc_port);

    Ok(())
}
