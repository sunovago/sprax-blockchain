import React, { useState } from "react";
import { Terminal } from "lucide-react";
import { Network } from "@/types";
import { PageHeader } from "@/components/PageHeader";
import { CopyButton } from "@/components/CopyButton";

interface DevelopersPageProps {
  network: Network;
}

export const DevelopersPage: React.FC<DevelopersPageProps> = ({ network }) => {
  const [selectedSnippet, setSelectedSnippet] = useState<"curl" | "ts" | "rust">("ts");

  const rpcUrl = network === "mainnet" ? "https://rpc.sprax.network:2657" : network === "testnet" ? "https://testnet-rpc.sprax.network:2657" : "http://127.0.0.1:2657";
  const apiBase = network === "mainnet" ? "https://mainnet.sprax.network/api/v1" : network === "testnet" ? "https://testnet-api.sprax.network/api/v1" : "http://127.0.0.1:8080/api/v1";
  const chainId = `sprax-${network}-1`;

  const curlSnippet = `# Check Sprax Chain RPC Status
curl -X GET "${rpcUrl}/status"

# Fetch latest indexed blocks via Explorer REST API
curl -X GET "${apiBase}/blocks?limit=10"

# Fetch account balance
curl -X GET "${apiBase}/addresses/sprax1qpzry9x8gf2tvdw0s3jn54khce6mua7l4w3e2r"`;

  const tsSnippet = `import { SpraxClient } from "@sprax/wallet-core";

async function main() {
  const client = new SpraxClient({
    rpcUrl: "${rpcUrl}",
    chainId: "${chainId}",
  });

  // Query network status and block height
  const status = await client.getNetworkStatus();
  console.log("Current Height:", status.latest_block_height);

  // Query account balance
  const balance = await client.getBalance("sprax1qpzry9x8gf2tvdw0s3jn54khce6mua7l4w3e2r");
  console.log("Balance:", balance.formatted, "SPRX");
}

main();`;

  const rustSnippet = `use sprax_types::{Address, Amount};
use sprax_node::rpc::RpcClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = RpcClient::connect("${rpcUrl}").await?;
    let stats = client.get_network_stats().await?;
    println!("SPRX Height: #{}", stats.latest_height);
    Ok(())
}`;

  return (
    <div className="space-y-8 animate-fadeIn max-w-5xl mx-auto">
      <PageHeader
        title="Developer & RPC Portal"
        subtitle="Public RPC endpoints, Explorer REST API reference, and integration client SDKs for Sprax Chain."
      />

      {/* Network Connection Parameters */}
      <div className="rounded-2xl border border-border-subtle bg-bg-surface p-6 shadow-card space-y-4">
        <h3 className="text-sm font-bold uppercase tracking-wider text-text-secondary pb-3 border-b border-border-subtle">
          Network Connection Parameters ({network.toUpperCase()})
        </h3>

        <div className="grid grid-cols-1 md:grid-cols-3 gap-y-4 gap-x-6 text-xs sm:text-sm font-mono-num">
          <div className="text-text-secondary">Chain ID</div>
          <div className="md:col-span-2 font-mono font-bold text-sky-400 flex items-center justify-between">
            <span>{chainId}</span>
            <CopyButton text={chainId} />
          </div>

          <div className="text-text-secondary">Native Denomination</div>
          <div className="md:col-span-2 font-mono text-text-primary">
            SPRX (18 Decimals / 1 atto-SPRX = 10^-18 SPRX)
          </div>

          <div className="text-text-secondary">JSON-RPC Endpoint</div>
          <div className="md:col-span-2 font-mono text-emerald-400 flex items-center justify-between break-all">
            <span>{rpcUrl}</span>
            <CopyButton text={rpcUrl} />
          </div>

          <div className="text-text-secondary">Explorer REST API Base</div>
          <div className="md:col-span-2 font-mono text-sky-400 flex items-center justify-between break-all">
            <span>{apiBase}</span>
            <CopyButton text={apiBase} />
          </div>
        </div>
      </div>

      {/* Code Integration Playground */}
      <div className="rounded-2xl border border-border-subtle bg-bg-surface overflow-hidden shadow-card">
        <div className="flex items-center justify-between px-6 py-4 border-b border-border-subtle bg-bg-surface-elevated/40">
          <div className="flex items-center gap-2">
            <Terminal className="w-4 h-4 text-sky-400" />
            <h3 className="text-sm font-bold text-text-primary">Integration Code Samples</h3>
          </div>

          <div className="flex items-center gap-1 bg-bg-primary p-1 rounded-lg border border-border-subtle text-xs">
            {(["ts", "curl", "rust"] as const).map((mode) => (
              <button
                key={mode}
                type="button"
                onClick={() => setSelectedSnippet(mode)}
                className={`px-3 py-1 rounded font-mono uppercase transition-colors ${
                  selectedSnippet === mode
                    ? "bg-sky-600 text-white font-semibold"
                    : "text-text-secondary hover:text-text-primary"
                }`}
              >
                {mode}
              </button>
            ))}
          </div>
        </div>

        <div className="p-6">
          <div className="flex items-center justify-between pb-2 mb-2 border-b border-border-subtle">
            <span className="text-xs font-mono text-text-muted">
              {selectedSnippet === "ts"
                ? "TypeScript / Node.js SDK"
                : selectedSnippet === "curl"
                ? "cURL Shell Commands"
                : "Rust Tokio Client"}
            </span>
            <CopyButton
              text={
                selectedSnippet === "ts"
                  ? tsSnippet
                  : selectedSnippet === "curl"
                  ? curlSnippet
                  : rustSnippet
              }
            />
          </div>
          <pre className="p-4 rounded-xl bg-bg-primary border border-border-subtle font-mono text-xs text-sky-300 overflow-x-auto leading-relaxed">
            {selectedSnippet === "ts"
              ? tsSnippet
              : selectedSnippet === "curl"
              ? curlSnippet
              : rustSnippet}
          </pre>
        </div>
      </div>

      {/* REST API Endpoints Reference */}
      <div className="rounded-2xl border border-border-subtle bg-bg-surface p-6 shadow-card space-y-4">
        <h3 className="text-sm font-bold uppercase tracking-wider text-text-secondary pb-3 border-b border-border-subtle">
          Explorer REST API Endpoints (`/api/v1`)
        </h3>

        <div className="space-y-3 font-mono text-xs">
          <div className="p-3 rounded-xl bg-bg-surface-elevated border border-border-subtle flex flex-col sm:flex-row sm:items-center justify-between gap-2">
            <div className="flex items-center gap-2">
              <span className="px-2 py-0.5 rounded bg-sky-500/20 text-sky-400 font-bold">GET</span>
              <span className="text-text-primary">/api/v1/blocks?limit=20&offset=0</span>
            </div>
            <span className="text-text-muted text-[11px]">Paginated chronological blocks</span>
          </div>

          <div className="p-3 rounded-xl bg-bg-surface-elevated border border-border-subtle flex flex-col sm:flex-row sm:items-center justify-between gap-2">
            <div className="flex items-center gap-2">
              <span className="px-2 py-0.5 rounded bg-sky-500/20 text-sky-400 font-bold">GET</span>
              <span className="text-text-primary">/api/v1/blocks/{`{height_or_hash}`}</span>
            </div>
            <span className="text-text-muted text-[11px]">Fetch single block details</span>
          </div>

          <div className="p-3 rounded-xl bg-bg-surface-elevated border border-border-subtle flex flex-col sm:flex-row sm:items-center justify-between gap-2">
            <div className="flex items-center gap-2">
              <span className="px-2 py-0.5 rounded bg-emerald-500/20 text-emerald-400 font-bold">GET</span>
              <span className="text-text-primary">/api/v1/txs?limit=20&offset=0</span>
            </div>
            <span className="text-text-muted text-[11px]">Paginated transactions</span>
          </div>

          <div className="p-3 rounded-xl bg-bg-surface-elevated border border-border-subtle flex flex-col sm:flex-row sm:items-center justify-between gap-2">
            <div className="flex items-center gap-2">
              <span className="px-2 py-0.5 rounded bg-emerald-500/20 text-emerald-400 font-bold">GET</span>
              <span className="text-text-primary">/api/v1/txs/{`{hash}`}</span>
            </div>
            <span className="text-text-muted text-[11px]">Fetch cryptographic receipt & status</span>
          </div>

          <div className="p-3 rounded-xl bg-bg-surface-elevated border border-border-subtle flex flex-col sm:flex-row sm:items-center justify-between gap-2">
            <div className="flex items-center gap-2">
              <span className="px-2 py-0.5 rounded bg-amber-500/20 text-amber-400 font-bold">GET</span>
              <span className="text-text-primary">/api/v1/addresses/{`{address}`}</span>
            </div>
            <span className="text-text-muted text-[11px]">Account balance & nonce</span>
          </div>

          <div className="p-3 rounded-xl bg-bg-surface-elevated border border-border-subtle flex flex-col sm:flex-row sm:items-center justify-between gap-2">
            <div className="flex items-center gap-2">
              <span className="px-2 py-0.5 rounded bg-purple-500/20 text-purple-400 font-bold">GET</span>
              <span className="text-text-primary">/api/v1/stats</span>
            </div>
            <span className="text-text-muted text-[11px]">High-level network health & TPS</span>
          </div>
        </div>
      </div>
    </div>
  );
};
