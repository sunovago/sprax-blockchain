use clap::Args;

#[derive(Debug, Args)]
pub(crate) struct VersionArgs {
    /// Output version in JSON format
    #[arg(long)]
    pub(crate) json: bool,
}

pub(crate) fn execute(args: &VersionArgs) -> anyhow::Result<()> {
    let version = env!("CARGO_PKG_VERSION");
    let name = env!("CARGO_PKG_NAME");
    let authors = env!("CARGO_PKG_AUTHORS");

    if args.json {
        let json_val = serde_json::json!({
            "name": name,
            "version": version,
            "authors": authors,
            "rustc": "rustc stable",
            "protocol_version": "1.0.0",
        });
        println!("{}", serde_json::to_string_pretty(&json_val)?);
    } else {
        println!("SPRX Protocol CLI ({name}) v{version}");
        println!("Authors: {authors}");
        println!("Protocol Version: 1.0.0");
    }
    Ok(())
}
