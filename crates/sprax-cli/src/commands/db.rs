use clap::{Args, Subcommand};
use sprax_node::NodeService;
use sprax_storage::{create_backup, rebuild_chain_indexes, restore_backup, verify_snapshot};
use std::path::PathBuf;

#[derive(Debug, Args)]
pub(crate) struct DbArgs {
    #[command(subcommand)]
    pub(crate) command: DbSubcommands,
}

#[derive(Debug, Subcommand)]
pub(crate) enum DbSubcommands {
    /// Create a consistent, point-in-time snapshot of the ledger database
    Backup {
        /// Destination directory for the backup snapshot
        #[arg(long)]
        out: PathBuf,

        /// Node home directory
        #[arg(long, default_value = ".sprx")]
        home: PathBuf,
    },
    /// Restore a backup snapshot into a fresh or non-colliding directory
    Restore {
        /// Path to the snapshot directory or manifest file
        #[arg(long)]
        snapshot: PathBuf,

        /// Destination directory for the restored node home
        #[arg(long)]
        out: PathBuf,
    },
    /// Verify file integrity and checksums of a snapshot manifest
    Verify {
        /// Path to the snapshot directory or manifest file
        #[arg(long)]
        snapshot: PathBuf,
    },
    /// Rebuild block hash and transaction receipt indexes from canonical blocks
    RebuildIndexes {
        /// Node home directory
        #[arg(long, default_value = ".sprx")]
        home: PathBuf,
    },
    /// Clear the safety guard preventing a restored node from consensus signing
    UnblockSigner {
        /// Node home directory
        #[arg(long, default_value = ".sprx")]
        home: PathBuf,
    },
}

pub(crate) fn execute(args: &DbArgs) -> anyhow::Result<()> {
    match &args.command {
        DbSubcommands::Backup { out, home } => {
            let node = NodeService::new_or_load(home.clone())?;
            let header = node.latest_header();
            let latest_hash = sprax_crypto::Hasher::block_hash(&header)?;
            let meta = sprax_storage::SnapshotMetadata {
                chain_id: node.chain_id(),
                genesis_fingerprint: sprax_types::Hash32::ZERO,
                finalized_height: node.height(),
                finalized_block_hash: latest_hash,
                state_root: header.state_root,
            };

            let data_dir = home.join("data");
            println!("Creating backup snapshot from {data_dir:?} to {out:?}...");
            let manifest = create_backup(&data_dir, out, meta)?;
            println!("============================================================");
            println!("  BACKUP SNAPSHOT CREATED SUCCESSFULLY");
            println!("============================================================");
            println!("  Chain ID        : {}", manifest.chain_id);
            println!("  Finalized Height: #{}", manifest.finalized_height);
            println!("  Block Hash      : {}", manifest.finalized_block_hash);
            println!("  State Root      : {}", manifest.state_root);
            println!("  Created At      : {}", manifest.created_at_unix_secs);
            println!("  Files Count     : {}", manifest.files.len());
            let manifest_file = out.join("snapshot_manifest.json");
            println!("  Manifest Path   : {manifest_file:?}");
            println!("============================================================");
        }
        DbSubcommands::Restore { snapshot, out } => {
            let manifest_path = if snapshot.is_file() {
                snapshot.clone()
            } else {
                snapshot.join("snapshot_manifest.json")
            };

            let data_dir = out.join("data");
            println!("Restoring backup from {manifest_path:?} to {data_dir:?}...");
            let manifest = restore_backup(&manifest_path, &data_dir, None, None)?;
            println!("============================================================");
            println!("  BACKUP RESTORED SUCCESSFULLY");
            println!("============================================================");
            println!("  Chain ID        : {}", manifest.chain_id);
            println!("  Finalized Height: #{}", manifest.finalized_height);
            println!("  Block Hash      : {}", manifest.finalized_block_hash);
            println!("  State Root      : {}", manifest.state_root);
            println!("  Target Dir      : {out:?}");
            println!("  WARNING: Restored node is signing-disabled by default.");
            println!("  To safely resume consensus signing after verifying journals:");
            println!("    sprax db unblock-signer --home {out:?}");
            println!("============================================================");
        }
        DbSubcommands::Verify { snapshot } => {
            let manifest_path = if snapshot.is_file() {
                snapshot.clone()
            } else {
                snapshot.join("snapshot_manifest.json")
            };

            println!("Verifying snapshot manifest {manifest_path:?}...");
            let manifest = verify_snapshot(&manifest_path)?;
            println!("============================================================");
            println!("  SNAPSHOT VERIFICATION: VALID");
            println!("============================================================");
            println!("  Chain ID        : {}", manifest.chain_id);
            println!("  Finalized Height: #{}", manifest.finalized_height);
            println!("  Block Hash      : {}", manifest.finalized_block_hash);
            println!("  State Root      : {}", manifest.state_root);
            println!(
                "  All {} files matched SHA256 checksums.",
                manifest.files.len()
            );
            println!("============================================================");
        }
        DbSubcommands::RebuildIndexes { home } => {
            let node = NodeService::new_or_load(home.clone())?;
            let db_data_dir = home.join("data");
            println!("Rebuilding indexes for database at {db_data_dir:?}...");
            let count = {
                let ledger = node.ledger();
                let guard = ledger.read();
                rebuild_chain_indexes(guard.store())?
            };
            println!("============================================================");
            println!("  INDEX REBUILD COMPLETE");
            println!("============================================================");
            println!("  Blocks Indexed  : {count}");
            println!("  Finalized Height: #{}", node.height());
            println!("============================================================");
        }
        DbSubcommands::UnblockSigner { home } => {
            let node = NodeService::new_or_load(home.clone())?;
            node.unblock_restored_signer()?;
            let unblocked_dir = home.join("data");
            println!("Restored signing guard marker successfully removed from {unblocked_dir:?}.");
            println!("Consensus signing can now be safely resumed by this node.");
        }
    }
    Ok(())
}
