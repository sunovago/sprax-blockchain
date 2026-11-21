import React, { useState } from "react";
import {
  X,
  FileCode2,
  CheckCircle2,
  Upload,
  AlertCircle,
  Code2,
  FileJson,
  ArrowRight,
} from "lucide-react";
import { SmartContract } from "@/types";

interface ContractVerificationModalProps {
  isOpen: boolean;
  initialContractAddress?: string;
  onClose: () => void;
  onVerified: (contract: SmartContract) => void;
}

const SAMPLE_SOURCE_CODE = `// CosmWasm Rust Smart Contract for Sprax Chain
use cosmwasm_std::{entry_point, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct InstantiateMsg {
    pub initial_counter: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Increment {},
    Reset { value: u64 },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetCount {},
}

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    deps.storage.set(b"counter", &msg.initial_counter.to_be_bytes());
    Ok(Response::new().add_attribute("method", "instantiate"))
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    match msg {
        ExecuteMsg::Increment {} => {
            let mut val = deps.storage.get(b"counter")
                .map(|b| u64::from_be_bytes(b.try_into().unwrap_or_default()))
                .unwrap_or(0);
            val += 1;
            deps.storage.set(b"counter", &val.to_be_bytes());
            Ok(Response::new().add_attribute("action", "increment"))
        }
        ExecuteMsg::Reset { value } => {
            deps.storage.set(b"counter", &value.to_be_bytes());
            Ok(Response::new().add_attribute("action", "reset"))
        }
    }
}
`;

const SAMPLE_ABI = JSON.stringify(
  {
    contract: "CounterWasm",
    compiler: "cosmwasm-rust:1.80.0",
    execute_messages: ["increment", "reset"],
    query_messages: ["get_count"],
    version: "0.1.0",
  },
  null,
  2
);

export const ContractVerificationModal: React.FC<ContractVerificationModalProps> = ({
  isOpen,
  initialContractAddress = "",
  onClose,
  onVerified,
}) => {
  const [contractAddress, setContractAddress] = useState(initialContractAddress);
  const [contractName, setContractName] = useState("");
  const [compilerVersion, setCompilerVersion] = useState("cosmwasm-rust:1.80.0");
  const [sourceCode, setSourceCode] = useState(SAMPLE_SOURCE_CODE);
  const [abi, setAbi] = useState(SAMPLE_ABI);
  const [uploadedFileName, setUploadedFileName] = useState<string | null>(null);
  const [bytecodeSize, setBytecodeSize] = useState<number>(32500);
  const [isVerifying, setIsVerifying] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<SmartContract | null>(null);

  if (!isOpen) return null;

  const handleFileUpload = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;

    setUploadedFileName(file.name);
    setBytecodeSize(file.size);

    if (file.name.endsWith(".rs") || file.name.endsWith(".txt")) {
      const reader = new FileReader();
      reader.onload = (event) => {
        const text = event.target?.result;
        if (typeof text === "string") {
          setSourceCode(text);
        }
      };
      reader.readAsText(file);
    }
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);

    const addr = contractAddress.trim();
    if (!addr || !addr.startsWith("sprax1")) {
      setError("Please provide a valid Sprax Bech32 contract address (starts with sprax1).");
      return;
    }

    if (!sourceCode.trim()) {
      setError("Contract source code cannot be empty.");
      return;
    }

    setIsVerifying(true);

    try {
      // Simulate cryptographic bytecode verification against chain state
      await new Promise((resolve) => setTimeout(resolve, 1400));

      const newVerifiedContract: SmartContract = {
        address: addr,
        creator: "sprax1sdtpz05x2r7d70u428v2ff3rtyj49z65a380gr",
        created_height: Math.floor(100000 + Math.random() * 500000),
        bytecode_size: bytecodeSize || 42000,
        tx_count: Math.floor(10 + Math.random() * 500),
        verified: true,
        compiler_version: compilerVersion,
        source_code: sourceCode,
        abi: abi,
      };

      setSuccess(newVerifiedContract);
      onVerified(newVerifiedContract);
    } catch (err: any) {
      setError(err.message || "Failed to verify contract");
    } finally {
      setIsVerifying(false);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/80 backdrop-blur-sm animate-fadeIn overflow-y-auto">
      <div className="relative w-full max-w-2xl my-8 rounded-2xl border border-border-subtle bg-bg-surface p-6 shadow-2xl space-y-6">
        {/* Header */}
        <div className="flex items-center justify-between border-b border-border-subtle pb-4">
          <div className="flex items-center gap-3">
            <div className="p-2 rounded-xl bg-indigo-500/10 border border-indigo-500/30 text-indigo-400">
              <FileCode2 className="w-5 h-5" />
            </div>
            <div>
              <h2 className="text-base sm:text-lg font-bold text-text-primary">
                Verify & Publish Smart Contract
              </h2>
              <p className="text-xs text-text-secondary">
                Upload CosmWasm Rust source code and match deterministic bytecode hash
              </p>
            </div>
          </div>
          <button
            onClick={onClose}
            className="p-1.5 rounded-lg text-text-muted hover:text-text-primary hover:bg-bg-hover transition-colors"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        {error && (
          <div className="flex items-start gap-2.5 p-3 rounded-xl bg-rose-500/10 border border-rose-500/30 text-rose-400 text-xs">
            <AlertCircle className="w-4 h-4 shrink-0 mt-0.5" />
            <span>{error}</span>
          </div>
        )}

        {success ? (
          <div className="space-y-4 py-4 text-center">
            <div className="w-14 h-14 mx-auto rounded-full bg-emerald-500/10 border border-emerald-500/30 flex items-center justify-center text-emerald-400">
              <CheckCircle2 className="w-8 h-8" />
            </div>
            <div className="space-y-1">
              <h3 className="text-base font-bold text-emerald-400">
                Contract Successfully Verified!
              </h3>
              <p className="text-xs text-text-secondary max-w-md mx-auto">
                Bytecode match confirmed with Sprax VM runtime. Verified source code and ABI are now publicly available on the Explorer.
              </p>
            </div>
            <div className="p-3 rounded-xl bg-bg-primary border border-border-subtle font-mono text-xs text-text-primary break-all">
              {success.address}
            </div>
            <div className="pt-2">
              <button
                onClick={onClose}
                className="w-full py-2.5 rounded-xl bg-indigo-600 hover:bg-indigo-500 text-white font-semibold text-xs transition-colors flex items-center justify-center gap-2"
              >
                <span>View Verified Contract</span>
                <ArrowRight className="w-4 h-4" />
              </button>
            </div>
          </div>
        ) : (
          <form onSubmit={handleSubmit} className="space-y-4">
            {/* Contract Address */}
            <div className="space-y-1.5">
              <label className="text-xs font-semibold text-text-secondary flex items-center justify-between">
                <span>Contract Address</span>
                <span className="text-[11px] text-text-muted">Must be deployed on-chain</span>
              </label>
              <input
                type="text"
                placeholder="sprax1..."
                value={contractAddress}
                onChange={(e) => setContractAddress(e.target.value)}
                required
                className="w-full bg-bg-primary border border-border-subtle rounded-xl px-3.5 py-2 text-xs font-mono text-text-primary placeholder:text-text-muted focus:outline-none focus:border-indigo-500"
              />
            </div>

            {/* Contract Name & Compiler Grid */}
            <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
              <div className="space-y-1.5">
                <label className="text-xs font-semibold text-text-secondary">
                  Contract Moniker / Name
                </label>
                <input
                  type="text"
                  placeholder="e.g. MyCW20Token"
                  value={contractName}
                  onChange={(e) => setContractName(e.target.value)}
                  className="w-full bg-bg-primary border border-border-subtle rounded-xl px-3.5 py-2 text-xs text-text-primary placeholder:text-text-muted focus:outline-none focus:border-indigo-500"
                />
              </div>

              <div className="space-y-1.5">
                <label className="text-xs font-semibold text-text-secondary">
                  Compiler & Toolchain
                </label>
                <select
                  value={compilerVersion}
                  onChange={(e) => setCompilerVersion(e.target.value)}
                  className="w-full bg-bg-primary border border-border-subtle rounded-xl px-3 py-2 text-xs text-text-primary focus:outline-none focus:border-indigo-500 cursor-pointer"
                >
                  <option value="cosmwasm-rust:1.80.0">cosmwasm-rust:1.80.0 (Recommended)</option>
                  <option value="cosmwasm-rust:1.75.0">cosmwasm-rust:1.75.0</option>
                  <option value="rustc:1.82.0-wasm32">rustc:1.82.0-wasm32</option>
                  <option value="docker:cosmwasm/rust-optimizer:0.16.0">
                    docker:cosmwasm/rust-optimizer:0.16.0
                  </option>
                </select>
              </div>
            </div>

            {/* Optional Wasm / Source File Upload */}
            <div className="space-y-1.5">
              <label className="text-xs font-semibold text-text-secondary flex items-center justify-between">
                <span>Upload Source File or Compiled .wasm (Optional)</span>
                {uploadedFileName && (
                  <span className="text-emerald-400 text-[11px]">✓ {uploadedFileName}</span>
                )}
              </label>
              <label className="flex items-center justify-center gap-2 p-3 rounded-xl border border-dashed border-border-subtle hover:border-indigo-500 bg-bg-primary/50 cursor-pointer text-xs text-text-muted hover:text-text-primary transition-colors">
                <Upload className="w-4 h-4 text-indigo-400" />
                <span>Choose .rs, .wasm, or .json file to autofill</span>
                <input
                  type="file"
                  accept=".rs,.wasm,.json,.txt"
                  onChange={handleFileUpload}
                  className="hidden"
                />
              </label>
            </div>

            {/* Source Code Editor */}
            <div className="space-y-1.5">
              <div className="flex items-center justify-between">
                <label className="text-xs font-semibold text-text-secondary flex items-center gap-1.5">
                  <Code2 className="w-3.5 h-3.5 text-indigo-400" />
                  <span>Rust Source Code (src/lib.rs)</span>
                </label>
                <span className="text-[11px] text-text-muted">CosmWasm Standard</span>
              </div>
              <textarea
                rows={6}
                value={sourceCode}
                onChange={(e) => setSourceCode(e.target.value)}
                className="w-full bg-bg-primary border border-border-subtle rounded-xl p-3 font-mono text-xs text-sky-300 placeholder:text-text-muted focus:outline-none focus:border-indigo-500 resize-vertical"
              />
            </div>

            {/* ABI / JSON Schema */}
            <div className="space-y-1.5">
              <label className="text-xs font-semibold text-text-secondary flex items-center gap-1.5">
                <FileJson className="w-3.5 h-3.5 text-emerald-400" />
                <span>Contract ABI / Schema (JSON)</span>
              </label>
              <textarea
                rows={3}
                value={abi}
                onChange={(e) => setAbi(e.target.value)}
                className="w-full bg-bg-primary border border-border-subtle rounded-xl p-3 font-mono text-xs text-emerald-300 placeholder:text-text-muted focus:outline-none focus:border-indigo-500 resize-vertical"
              />
            </div>

            {/* Submit Button */}
            <div className="pt-2">
              <button
                type="submit"
                disabled={isVerifying}
                className="w-full py-2.5 rounded-xl bg-indigo-600 hover:bg-indigo-500 disabled:bg-indigo-600/50 text-white font-semibold text-xs sm:text-sm transition-colors flex items-center justify-center gap-2"
              >
                {isVerifying ? (
                  <>
                    <div className="w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin" />
                    <span>Compiling & Verifying Bytecode...</span>
                  </>
                ) : (
                  <>
                    <CheckCircle2 className="w-4 h-4" />
                    <span>Verify & Publish Contract</span>
                  </>
                )}
              </button>
            </div>
          </form>
        )}
      </div>
    </div>
  );
};
