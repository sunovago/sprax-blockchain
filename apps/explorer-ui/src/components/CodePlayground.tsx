import React, { useState } from "react";
import { Check, Copy, Play, Terminal } from "lucide-react";

interface CodeSnippet {
  language: string;
  name: string;
  code: string;
  description: string;
  output: string;
}

const SNIPPETS: Record<string, CodeSnippet> = {
  typescript: {
    language: "typescript",
    name: "TypeScript SDK",
    description: "Connect to SPRX Node and query account balance using @sprax/wallet-core",
    code: `import { SpraxClient, HDWallet } from "@sprax/wallet-core";

// 1. Initialize high-throughput JSON-RPC client
const client = new SpraxClient("https://rpc.sprax.network:26657");

// 2. Query account state & balance (18 decimals precision)
const address = "sprax1sdtpz05x2r7d70u428v2ff3rtyj49z65a380gr";
const account = await client.getAccount(address);

console.log("SPRX Balance:", account.balanceSprx, "SPRX");
console.log("Atomic Atto:", account.balanceAtto, "atto-SPRX");
console.log("Account Nonce:", account.nonce);`,
    output: `[SPRX Client Connected: sprax-mainnet-1]
> Querying address: sprax1sdtpz05x2r7d70u428v2ff3rtyj49z65a380gr
> SPRX Balance: 154,250.00 SPRX
> Atomic Atto: 154250000000000000000000 atto-SPRX
> Account Nonce: 42
> Finality Verification: Deterministic (Block #8245920)`,
  },
  rust: {
    language: "rust",
    name: "Rust / CosmWasm",
    description: "Deterministic CosmWasm WASM contract execution handler in sprax-wasm",
    code: `use cosmwasm_std::{entry_point, DepsMut, Env, MessageInfo, Response, StdResult};
use cw20_base::msg::ExecuteMsg;

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    match msg {
        ExecuteMsg::Transfer { recipient, amount } => {
            // Sandboxed non-reentrant state transition
            cw20_base::contract::execute_transfer(deps, env, info, recipient, amount)
        }
        ExecuteMsg::Burn { amount } => {
            cw20_base::contract::execute_burn(deps, env, info, amount)
        }
        _ => Ok(Response::default()),
    }
}`,
    output: `Compiling sprax_wasm_contract v0.1.0 (wasm32-unknown-unknown)
   Compiling cosmwasm-std v1.5.0
   Compiling cw20-base v0.16.0
    Finished release [optimized] target(s) in 1.42s
Bytecode Blake3 Hash: 0x4a8f9c1b3e2d5a7f8091a2b3c4d5e6f708192a3b4c5d6e7f8091a2b3c4d5e6f7
Gas Metering Target: 21,000 base + 500/storage write`,
  },
  curl: {
    language: "bash",
    name: "cURL / JSON-RPC",
    description: "Raw JSON-RPC 2.0 sprax_getStatus query to consensus node",
    code: `curl -X POST https://rpc.sprax.network:26657 \\
  -H "Content-Type: application/json" \\
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "sprax_getStatus",
    "params": []
  }'`,
    output: `{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "chainId": "sprax-mainnet-1",
    "latestBlockHeight": 8245920,
    "latestBlockHash": "0x7f8a9b2c3d4e5f60718293a4b5c6d7e8f901a2b3c4d5e6f708192a3b4c5d6e70",
    "connectedPeers": 48,
    "validatorCount": 100,
    "syncStatus": "synced",
    "mempoolPending": 12
  }
}`,
  },
  python: {
    language: "python",
    name: "Python",
    description: "Broadcast signed transaction using Python JSON-RPC adapter",
    code: `import requests
import json

RPC_URL = "https://rpc.sprax.network:26657"

payload = {
    "jsonrpc": "2.0",
    "id": 1,
    "method": "sprax_getBlock",
    "params": ["latest"]
}

response = requests.post(RPC_URL, json=payload).json()
block = response["result"]
print(f"Latest Block #{block['header']['height']} | Proposer: {block['header']['proposer']}")`,
    output: `Connected to SPRX Python Client
Latest Block #8245920 | Proposer: spraxvaloper1q8x9y7w6e5r4t3y2u1i0o9p8a7s6d5f4g3h2j1
State Root: 0xabcdef0123456789abcdef0123456789...
Transactions in Block: 14`,
  },
  go: {
    language: "go",
    name: "Go (Golang)",
    description: "Query CometBFT consensus engine validator set in Go",
    code: `package main

import (
	"context"
	"fmt"
	"github.com/sprax-chain/sprax-sdk-go/client"
)

func main() {
	cli := client.NewClient("https://rpc.sprax.network:26657")
	validators, err := cli.GetValidators(context.Background())
	if err != nil {
		panic(err)
	}

	fmt.Printf("Active Validators Count: %d\\n", len(validators))
	for _, val := range validators[:3] {
		fmt.Printf("Moniker: %s | Voting Power: %d\\n", val.Moniker, val.VotingPower)
	}
}`,
    output: `Active Validators Count: 100
Moniker: SPRX Core Genesis Alpha | Voting Power: 3500000
Moniker: Global Real-World X Node | Voting Power: 2800000
Moniker: Nexus Staking Labs | Voting Power: 1900000`,
  },
};

export const CodePlayground: React.FC = () => {
  const [activeTab, setActiveTab] = useState<string>("typescript");
  const [copied, setCopied] = useState(false);
  const [isExecuting, setIsExecuting] = useState(false);
  const [showOutput, setShowOutput] = useState(true);

  const currentSnippet = SNIPPETS[activeTab] || SNIPPETS.typescript;

  const handleCopy = () => {
    navigator.clipboard.writeText(currentSnippet.code);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const handleRun = () => {
    setIsExecuting(true);
    setShowOutput(true);
    setTimeout(() => {
      setIsExecuting(false);
    }, 450);
  };

  return (
    <div className="rounded-2xl border border-border-prominent bg-bg-surface overflow-hidden shadow-card">
      {/* Top Header Bar */}
      <div className="flex flex-wrap items-center justify-between border-b border-border-subtle bg-bg-surface-elevated/80 px-4 py-2.5 gap-2">
        {/* Language Tabs */}
        <div className="flex items-center gap-1 overflow-x-auto py-1">
          {Object.entries(SNIPPETS).map(([key, item]) => (
            <button
              key={key}
              onClick={() => setActiveTab(key)}
              className={`px-3 py-1 rounded-lg text-xs font-semibold transition-all ${
                activeTab === key
                  ? "bg-sky-500/20 text-sky-400 border border-sky-500/40 shadow-sm"
                  : "text-text-secondary hover:text-text-primary hover:bg-bg-surface"
              }`}
            >
              {item.name}
            </button>
          ))}
        </div>

        {/* Action Controls */}
        <div className="flex items-center gap-2">
          <button
            onClick={handleRun}
            disabled={isExecuting}
            className="inline-flex items-center gap-1.5 px-3 py-1 rounded-lg bg-emerald-500/20 border border-emerald-500/30 text-emerald-400 text-xs font-bold hover:bg-emerald-500/30 transition-all disabled:opacity-50"
          >
            <Play className={`w-3 h-3 ${isExecuting ? "animate-spin" : ""}`} />
            <span>{isExecuting ? "Executing..." : "Run Snippet"}</span>
          </button>

          <button
            onClick={handleCopy}
            className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-lg bg-bg-surface border border-border-subtle text-text-secondary text-xs font-medium hover:text-text-primary hover:border-border-prominent transition-all"
            title="Copy Code"
          >
            {copied ? (
              <>
                <Check className="w-3 h-3 text-emerald-400" />
                <span className="text-emerald-400">Copied!</span>
              </>
            ) : (
              <>
                <Copy className="w-3 h-3" />
                <span>Copy</span>
              </>
            )}
          </button>
        </div>
      </div>

      {/* Code Description Bar */}
      <div className="px-4 py-2 bg-bg-surface/50 border-b border-border-subtle/50 text-xs text-text-muted flex items-center justify-between">
        <span>{currentSnippet.description}</span>
        <span className="text-[10px] font-mono-num uppercase">{currentSnippet.language}</span>
      </div>

      {/* Code Editor Body */}
      <div className="p-4 overflow-x-auto bg-[#070a0f] text-slate-100 font-mono text-xs sm:text-sm leading-relaxed">
        <pre className="selection:bg-sky-500/30">
          <code>{currentSnippet.code}</code>
        </pre>
      </div>

      {/* Execution Output Console */}
      {showOutput && (
        <div className="border-t border-border-subtle bg-[#05080c] p-4 text-xs font-mono">
          <div className="flex items-center gap-2 text-text-muted mb-2 font-sans font-semibold">
            <Terminal className="w-3.5 h-3.5 text-sky-400" />
            <span>Simulated Console Output:</span>
            {isExecuting && (
              <span className="inline-block w-1.5 h-1.5 rounded-full bg-sky-400 animate-ping" />
            )}
          </div>
          <pre className="text-emerald-400/90 whitespace-pre-wrap leading-relaxed overflow-x-auto">
            {isExecuting ? "Executing transaction request on SPRX consensus plane..." : currentSnippet.output}
          </pre>
        </div>
      )}
    </div>
  );
};
