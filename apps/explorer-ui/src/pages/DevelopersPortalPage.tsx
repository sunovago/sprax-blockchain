import React, { useState } from "react";
import {
  Code2,
  Terminal,
  Cpu,
  Globe,
  Coins,
  Play,
  CheckCircle2,
} from "lucide-react";
import { Network } from "@/types";
import { CodePlayground } from "@/components/CodePlayground";

interface DevelopersPortalPageProps {
  network?: Network;
  initialTab?: string;
  onNavigate: (route: string) => void;
}

export const DevelopersPortalPage: React.FC<DevelopersPortalPageProps> = ({
  initialTab = "overview",
  onNavigate,
}) => {
  const [activeTab, setActiveTab] = useState<string>(() => {
    if (initialTab.includes("rpc")) return "rpc";
    if (initialTab.includes("smart-contracts")) return "smart-contracts";
    if (initialTab.includes("nodes")) return "nodes";
    if (initialTab.includes("standards")) return "standards";
    return "overview";
  });

  // RPC Playground State
  const [selectedMethod, setSelectedMethod] = useState<string>("sprax_getStatus");
  const [rpcParamInput, setRpcParamInput] = useState<string>("[]");
  const [rpcResponse, setRpcResponse] = useState<string | null>(null);
  const [isExecutingRpc, setIsExecutingRpc] = useState(false);

  const rpcMethods = [
    {
      method: "sprax_getStatus",
      category: "Chain & Consensus",
      description: "Returns active chain ID, latest block height, state root, and validator counts.",
      sampleParams: "[]",
      sampleResult: {
        chainId: "sprax-mainnet-1",
        latestBlockHeight: 8245920,
        latestBlockHash: "0x7f8a9b2c3d4e5f60718293a4b5c6d7e8f901a2b3c4d5e6f708192a3b4c5d6e70",
        stateRoot: "0xabcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789",
        connectedPeers: 48,
        validatorCount: 100,
        syncStatus: "synced",
        mempoolPending: 12,
      },
    },
    {
      method: "sprax_getBlock",
      category: "Block Ledger",
      description: "Fetches block header, state root commitments, and transactions list by height or hash.",
      sampleParams: '["latest"]',
      sampleResult: {
        hash: "0x7f8a9b2c3d4e5f60718293a4b5c6d7e8f901a2b3c4d5e6f708192a3b4c5d6e70",
        header: {
          height: 8245920,
          timestampUnixSecs: 1774000000,
          proposer: "spraxvaloper1q8x9y7w6e5r4t3y2u1i0o9p8a7s6d5f4g3h2j1",
          txsCount: 14,
          gasUsed: 145000,
        },
      },
    },
    {
      method: "sprax_getAccount",
      category: "State Query",
      description: "Retrieves 18-decimal balance in atto-SPRX and account sequence nonce.",
      sampleParams: '["sprax1sdtpz05x2r7d70u428v2ff3rtyj49z65a380gr"]',
      sampleResult: {
        address: "sprax1sdtpz05x2r7d70u428v2ff3rtyj49z65a380gr",
        balance: "154250000000000000000000",
        balanceSprx: "154250.00",
        nonce: 42,
      },
    },
    {
      method: "sprax_estimateFee",
      category: "Execution",
      description: "Estimates base gas fee and recommended gas limit for a proposed transaction payload.",
      sampleParams: "[]",
      sampleResult: {
        fee: "500000000000000",
        feeSprx: "0.0005",
        gasLimit: 200000,
      },
    },
    {
      method: "sprax_getValidators",
      category: "Staking",
      description: "Returns active validator set with voting power and consensus public keys.",
      sampleParams: "[]",
      sampleResult: [
        {
          address: "spraxvaloper1q8x9y7w6e5r4t3y2u1i0o9p8a7s6d5f4g3h2j1",
          votingPower: 3500000,
          moniker: "SPRX Core Genesis Alpha",
        },
        {
          address: "spraxvaloper1a2s3d4f5g6h7j8k9l0z1x2c3v4b5n6m7q8w9e0",
          votingPower: 2800000,
          moniker: "Global Real-World X Node",
        },
      ],
    },
  ];

  const handleSelectMethod = (m: string) => {
    setSelectedMethod(m);
    const item = rpcMethods.find((x) => x.method === m);
    if (item) {
      setRpcParamInput(item.sampleParams);
      setRpcResponse(null);
    }
  };

  const handleRunRpc = () => {
    setIsExecutingRpc(true);
    setTimeout(() => {
      const item = rpcMethods.find((x) => x.method === selectedMethod);
      setRpcResponse(
        JSON.stringify(
          {
            jsonrpc: "2.0",
            id: 1,
            result: item?.sampleResult || {},
          },
          null,
          2
        )
      );
      setIsExecutingRpc(false);
    }, 350);
  };

  return (
    <div className="space-y-8 animate-fadeIn">
      {/* Developer Header Banner */}
      <div className="rounded-3xl border border-border-prominent bg-gradient-to-r from-bg-surface via-bg-surface-elevated/40 to-bg-surface p-6 sm:p-10 space-y-4 shadow-card">
        <div className="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-cyan-500/10 border border-cyan-500/20 text-xs font-bold text-cyan-400">
          <Code2 className="w-3.5 h-3.5" />
          <span>DEVELOPER COMMAND CENTER</span>
        </div>
        <h1 className="text-2xl sm:text-4xl font-extrabold text-text-primary tracking-tight">
          Build on SPRX Protocol
        </h1>
        <p className="text-xs sm:text-sm text-text-secondary max-w-2xl leading-relaxed">
          Access high-performance JSON-RPC & REST endpoints, compile Rust CosmWasm smart contracts, integrate the TypeScript SDK, and run sovereign validator nodes.
        </p>

        {/* Tab Navigation */}
        <div className="pt-2 flex items-center gap-1.5 overflow-x-auto">
          <button
            onClick={() => setActiveTab("overview")}
            className={`px-3 py-1.5 rounded-xl text-xs font-bold transition-all shrink-0 ${
              activeTab === "overview"
                ? "bg-cyan-500/20 text-cyan-400 border border-cyan-500/40"
                : "bg-bg-surface border border-border-subtle text-text-secondary hover:text-text-primary"
            }`}
          >
            Developer Overview
          </button>
          <button
            onClick={() => setActiveTab("rpc")}
            className={`px-3 py-1.5 rounded-xl text-xs font-bold transition-all shrink-0 ${
              activeTab === "rpc"
                ? "bg-cyan-500/20 text-cyan-400 border border-cyan-500/40"
                : "bg-bg-surface border border-border-subtle text-text-secondary hover:text-text-primary"
            }`}
          >
            JSON-RPC & REST Playground
          </button>
          <button
            onClick={() => setActiveTab("smart-contracts")}
            className={`px-3 py-1.5 rounded-xl text-xs font-bold transition-all shrink-0 ${
              activeTab === "smart-contracts"
                ? "bg-cyan-500/20 text-cyan-400 border border-cyan-500/40"
                : "bg-bg-surface border border-border-subtle text-text-secondary hover:text-text-primary"
            }`}
          >
            CosmWasm Smart Contracts
          </button>
          <button
            onClick={() => setActiveTab("nodes")}
            className={`px-3 py-1.5 rounded-xl text-xs font-bold transition-all shrink-0 ${
              activeTab === "nodes"
                ? "bg-cyan-500/20 text-cyan-400 border border-cyan-500/40"
                : "bg-bg-surface border border-border-subtle text-text-secondary hover:text-text-primary"
            }`}
          >
            Run a Node & Sentry
          </button>
          <button
            onClick={() => setActiveTab("standards")}
            className={`px-3 py-1.5 rounded-xl text-xs font-bold transition-all shrink-0 ${
              activeTab === "standards"
                ? "bg-cyan-500/20 text-cyan-400 border border-cyan-500/40"
                : "bg-bg-surface border border-border-subtle text-text-secondary hover:text-text-primary"
            }`}
          >
            SXS Protocol Standards
          </button>
          <button
            onClick={() => onNavigate("/faucet")}
            className="px-3 py-1.5 rounded-xl text-xs font-bold bg-sky-500/10 border border-sky-500/30 text-sky-400 hover:bg-sky-500/20 transition-all shrink-0 flex items-center gap-1"
          >
            <Coins className="w-3.5 h-3.5" />
            <span>Testnet Faucet</span>
          </button>
        </div>
      </div>

      {/* 1. Overview Tab */}
      {activeTab === "overview" && (
        <div className="space-y-8">
          <CodePlayground />

          <div className="grid grid-cols-1 sm:grid-cols-3 gap-6">
            <div
              onClick={() => setActiveTab("rpc")}
              className="cursor-pointer group rounded-2xl border border-border-subtle bg-bg-surface p-6 hover:bg-bg-surface-elevated hover:border-cyan-500/40 transition-all space-y-3"
            >
              <div className="p-3 rounded-xl bg-cyan-500/10 text-cyan-400 border border-cyan-500/20 w-fit">
                <Terminal className="w-5 h-5" />
              </div>
              <h3 className="text-base font-bold text-text-primary group-hover:text-cyan-400 transition-colors">
                Interactive API Playground
              </h3>
              <p className="text-xs text-text-secondary leading-relaxed">
                Test JSON-RPC methods like <code className="text-cyan-400">sprax_getStatus</code> and <code className="text-cyan-400">sprax_broadcastTx</code> in real-time.
              </p>
            </div>

            <div
              onClick={() => setActiveTab("smart-contracts")}
              className="cursor-pointer group rounded-2xl border border-border-subtle bg-bg-surface p-6 hover:bg-bg-surface-elevated hover:border-cyan-500/40 transition-all space-y-3"
            >
              <div className="p-3 rounded-xl bg-purple-500/10 text-purple-400 border border-purple-500/20 w-fit">
                <Cpu className="w-5 h-5" />
              </div>
              <h3 className="text-base font-bold text-text-primary group-hover:text-cyan-400 transition-colors">
                CosmWasm Rust Contracts
              </h3>
              <p className="text-xs text-text-secondary leading-relaxed">
                Build CW20 fungible tokens and multi-party escrow contracts with native compile-time memory safety.
              </p>
            </div>

            <div
              onClick={() => setActiveTab("nodes")}
              className="cursor-pointer group rounded-2xl border border-border-subtle bg-bg-surface p-6 hover:bg-bg-surface-elevated hover:border-cyan-500/40 transition-all space-y-3"
            >
              <div className="p-3 rounded-xl bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 w-fit">
                <Globe className="w-5 h-5" />
              </div>
              <h3 className="text-base font-bold text-text-primary group-hover:text-cyan-400 transition-colors">
                Node Operations & CLI
              </h3>
              <p className="text-xs text-text-secondary leading-relaxed">
                Deploy full RPC nodes and isolated validator sentries using the official <code className="text-cyan-400">sprax</code> CLI binary.
              </p>
            </div>
          </div>
        </div>
      )}

      {/* 2. Interactive JSON-RPC Playground */}
      {activeTab === "rpc" && (
        <div className="grid grid-cols-1 lg:grid-cols-12 gap-6">
          {/* Methods List (Left 4 Cols) */}
          <div className="lg:col-span-4 rounded-2xl border border-border-subtle bg-bg-surface p-4 space-y-2">
            <h3 className="text-xs font-bold uppercase tracking-wider text-text-muted px-2 mb-2">
              JSON-RPC 2.0 Methods
            </h3>
            <div className="space-y-1">
              {rpcMethods.map((m) => (
                <div
                  key={m.method}
                  onClick={() => handleSelectMethod(m.method)}
                  className={`p-3 rounded-xl cursor-pointer transition-all ${
                    selectedMethod === m.method
                      ? "bg-cyan-500/20 border border-cyan-500/40 text-cyan-400"
                      : "hover:bg-bg-surface-elevated text-text-secondary"
                  }`}
                >
                  <div className="text-xs font-mono font-bold">{m.method}</div>
                  <div className="text-[11px] text-text-muted mt-0.5">{m.category}</div>
                </div>
              ))}
            </div>
          </div>

          {/* Interactive Request & Response Box (Right 8 Cols) */}
          <div className="lg:col-span-8 space-y-4">
            <div className="rounded-2xl border border-border-prominent bg-bg-surface p-5 space-y-4 shadow-card">
              <div className="flex items-center justify-between">
                <div>
                  <h3 className="text-base font-bold text-text-primary font-mono">
                    {selectedMethod}
                  </h3>
                  <p className="text-xs text-text-muted mt-0.5">
                    {rpcMethods.find((m) => m.method === selectedMethod)?.description}
                  </p>
                </div>
                <button
                  onClick={handleRunRpc}
                  disabled={isExecutingRpc}
                  className="px-4 py-2 rounded-xl bg-gradient-to-r from-sky-500 to-indigo-600 hover:from-sky-400 hover:to-indigo-500 text-white text-xs font-bold flex items-center gap-1.5 shadow-md disabled:opacity-50"
                >
                  <Play className={`w-3.5 h-3.5 ${isExecutingRpc ? "animate-spin" : ""}`} />
                  <span>{isExecutingRpc ? "Calling..." : "Execute Call"}</span>
                </button>
              </div>

              <div>
                <label className="text-xs font-bold text-text-muted uppercase block mb-1">
                  Parameters JSON Array:
                </label>
                <input
                  type="text"
                  value={rpcParamInput}
                  onChange={(e) => setRpcParamInput(e.target.value)}
                  className="w-full p-2.5 rounded-xl bg-[#080c14] border border-border-subtle text-cyan-400 font-mono text-xs focus:outline-none focus:border-cyan-400"
                />
              </div>

              {/* Response Inspector */}
              <div>
                <label className="text-xs font-bold text-text-muted uppercase block mb-1">
                  JSON-RPC 2.0 Response:
                </label>
                <div className="p-4 rounded-xl bg-[#05080c] border border-border-subtle font-mono text-xs text-emerald-400 overflow-x-auto min-h-[160px]">
                  <pre>
                    {rpcResponse ||
                      JSON.stringify(
                        {
                          jsonrpc: "2.0",
                          id: 1,
                          result:
                            rpcMethods.find((m) => m.method === selectedMethod)?.sampleResult || {},
                        },
                        null,
                        2
                      )}
                  </pre>
                </div>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* 3. CosmWasm Smart Contracts */}
      {activeTab === "smart-contracts" && (
        <div className="rounded-3xl border border-border-subtle bg-bg-surface p-6 sm:p-10 space-y-6 max-w-4xl mx-auto shadow-card">
          <div className="space-y-2 border-b border-border-subtle pb-6">
            <span className="text-xs font-bold text-purple-400 uppercase tracking-wider">
              Smart Contract Architecture
            </span>
            <h2 className="text-2xl sm:text-3xl font-extrabold text-text-primary">
              CosmWasm Rust Smart Contracts (sprax-wasm)
            </h2>
            <p className="text-xs sm:text-sm text-text-secondary">
              Write, compile, and deploy deterministic WASM smart contracts using the CosmWasm actor model.
            </p>
          </div>

          <div className="space-y-4 text-xs sm:text-sm text-text-secondary leading-relaxed">
            <h3 className="text-base font-bold text-text-primary">
              1. Compilation Target
            </h3>
            <p>
              SPRX smart contracts compile to the standalone <code className="text-cyan-400">wasm32-unknown-unknown</code> target:
            </p>
            <div className="p-3 rounded-xl bg-[#080c14] border border-border-subtle font-mono text-xs text-slate-100">
              cargo build --target wasm32-unknown-unknown --release
            </div>

            <h3 className="text-base font-bold text-text-primary pt-4">
              2. Contract Entry Points
            </h3>
            <ul className="space-y-2 text-xs">
              <li className="flex items-start gap-2 text-text-primary">
                <CheckCircle2 className="w-4 h-4 text-emerald-400 shrink-0 mt-0.5" />
                <span><code className="text-cyan-400 font-bold">instantiate()</code>: Initializes contract state and assigns administrative parameters.</span>
              </li>
              <li className="flex items-start gap-2 text-text-primary">
                <CheckCircle2 className="w-4 h-4 text-emerald-400 shrink-0 mt-0.5" />
                <span><code className="text-cyan-400 font-bold">execute()</code>: State mutating actions (Transfers, Escrow Releases, Minting).</span>
              </li>
              <li className="flex items-start gap-2 text-text-primary">
                <CheckCircle2 className="w-4 h-4 text-emerald-400 shrink-0 mt-0.5" />
                <span><code className="text-cyan-400 font-bold">query()</code>: Read-only gas-metered state inspection.</span>
              </li>
            </ul>
          </div>
        </div>
      )}

      {/* 4. Node Operations */}
      {activeTab === "nodes" && (
        <div className="rounded-3xl border border-border-subtle bg-bg-surface p-6 sm:p-10 space-y-6 max-w-4xl mx-auto shadow-card">
          <div className="space-y-2 border-b border-border-subtle pb-6">
            <span className="text-xs font-bold text-emerald-400 uppercase tracking-wider">
              Node Infrastructure
            </span>
            <h2 className="text-2xl sm:text-3xl font-extrabold text-text-primary">
              Running a Sovereign SPRX Node
            </h2>
            <p className="text-xs sm:text-sm text-text-secondary">
              Hardware requirements, CLI initialization, and P2P gossip seed connection.
            </p>
          </div>

          <div className="space-y-4 text-xs sm:text-sm text-text-secondary leading-relaxed">
            <h3 className="text-base font-bold text-text-primary">
              CLI Node Commands
            </h3>
            <div className="p-4 rounded-xl bg-[#080c14] border border-border-subtle font-mono text-xs text-slate-100 space-y-2">
              <p className="text-text-muted"># 1. Initialize node configuration</p>
              <p className="text-cyan-400">sprax node init --chain-id sprax-testnet-1 --moniker "MyValidatorNode"</p>
              <p className="text-text-muted pt-2"># 2. Start node connected to testnet seed</p>
              <p className="text-emerald-400">sprax node start --p2p.seeds "tcp://seed.testnet.sprax.network:26656"</p>
            </div>
          </div>
        </div>
      )}

      {/* 5. Standards */}
      {activeTab === "standards" && (
        <div className="rounded-3xl border border-border-subtle bg-bg-surface p-6 sm:p-10 space-y-6 max-w-4xl mx-auto shadow-card">
          <div className="space-y-2 border-b border-border-subtle pb-6">
            <span className="text-xs font-bold text-cyan-400 uppercase tracking-wider">
              Protocol Specifications
            </span>
            <h2 className="text-2xl sm:text-3xl font-extrabold text-text-primary">
              SXS — SPRX eXtensible Standards
            </h2>
            <p className="text-xs sm:text-sm text-text-secondary">
              Official smart contract interfaces for tokens, real-world assets, and escrow settlements.
            </p>
          </div>

          <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
            <div className="rounded-2xl border border-border-subtle bg-bg-surface-elevated p-5 space-y-2">
              <span className="text-xs font-bold text-cyan-400">SXS-20</span>
              <h3 className="text-base font-bold text-text-primary">Fungible Token Standard</h3>
              <p className="text-xs text-text-muted">
                Standard token interface implementing Transfer, TransferFrom, Approve, Allowance, and Mint.
              </p>
            </div>

            <div className="rounded-2xl border border-border-subtle bg-bg-surface-elevated p-5 space-y-2">
              <span className="text-xs font-bold text-emerald-400">SXS-721</span>
              <h3 className="text-base font-bold text-text-primary">Real-World Asset NFT</h3>
              <p className="text-xs text-text-muted">
                Asset standard binding on-chain cryptographic state with real-world legal custody certificates.
              </p>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
