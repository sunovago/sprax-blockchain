use clap::{Parser, Subcommand};

mod commands;

#[derive(Debug, Parser)]
#[command(
    name = "sprax",
    about = "SPRX Protocol (Scalable Protocol for Real-world X) CLI & Local Node Daemon",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Initialize a new node configuration, keyring, and genesis state
    Init(commands::init::InitArgs),
    /// Start the local blockchain node daemon
    Start(commands::start::StartArgs),
    /// Key management and derivation utilities
    Keys(commands::keys::KeysArgs),
    /// Submit transactions to the local chain
    Tx(commands::tx::TxArgs),
    /// Query blockchain state, balances, blocks, and transaction receipts
    Query(commands::query::QueryArgs),
    /// Query local node status and configuration
    Status(commands::status::StatusArgs),
    /// Print binary version and build metadata
    Version(commands::version::VersionArgs),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init(args) => commands::init::execute(&args),
        Commands::Start(args) => commands::start::execute(&args).await,
        Commands::Keys(args) => commands::keys::execute(&args),
        Commands::Tx(args) => commands::tx::execute(&args),
        Commands::Query(args) => commands::query::execute(&args),
        Commands::Status(args) => commands::status::execute(&args),
        Commands::Version(args) => commands::version::execute(&args),
    }
}
