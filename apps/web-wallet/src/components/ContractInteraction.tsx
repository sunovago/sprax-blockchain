import React, { useState } from "react";
import {
  Account,
  AddressUtil,
  NetworkConfig,
  SpraxClient,
  TransactionBuilder,
} from "@sprax/wallet-core";

interface ContractInteractionProps {
  account: Account;
  network: NetworkConfig;
  privateKey: Uint8Array;
}

const QUERY_TEMPLATES: Record<string, string> = {
  custom: `{\n  \n}`,
  cw20_token_info: `{\n  "token_info": {}\n}`,
  cw20_balance: `{\n  "balance": {\n    "address": "{SENDER}"\n  }\n}`,
  cw20_allowance: `{\n  "allowance": {\n    "owner": "{SENDER}",\n    "spender": ""\n  }\n}`,
  cw20_minter: `{\n  "minter": {}\n}`,
  cw20_all_accounts: `{\n  "all_accounts": {\n    "limit": 10\n  }\n}`,
  cw721_contract_info: `{\n  "contract_info": {}\n}`,
  cw721_num_tokens: `{\n  "num_tokens": {}\n}`,
  cw721_owner_of: `{\n  "owner_of": {\n    "token_id": "1"\n  }\n}`,
};

const EXECUTE_TEMPLATES: Record<string, string> = {
  custom: `{\n  \n}`,
  cw20_transfer: `{\n  "transfer": {\n    "recipient": "",\n    "amount": 1000000\n  }\n}`,
  cw20_mint: `{\n  "mint": {\n    "recipient": "{SENDER}",\n    "amount": 1000000\n  }\n}`,
  cw20_burn: `{\n  "burn": {\n    "amount": 500000\n  }\n}`,
  cw20_increase_allowance: `{\n  "increase_allowance": {\n    "spender": "",\n    "amount": 1000000\n  }\n}`,
  cw20_send: `{\n  "send": {\n    "contract": "",\n    "amount": 1000000,\n    "msg": ""\n  }\n}`,
};

export const ContractInteraction: React.FC<ContractInteractionProps> = ({
  account,
  network,
  privateKey,
}) => {
  const [activeTab, setActiveTab] = useState<"query" | "execute">("query");

  // Query tab state
  const [queryContractAddress, setQueryContractAddress] = useState("");
  const [selectedQueryTemplate, setSelectedQueryTemplate] = useState("cw20_token_info");
  const [queryMsg, setQueryMsg] = useState(
    QUERY_TEMPLATES.cw20_token_info.replace("{SENDER}", account.addressBech32)
  );
  const [queryGas, setQueryGas] = useState(200000);
  const [isQuerying, setIsQuerying] = useState(false);
  const [queryResult, setQueryResult] = useState<any>(null);
  const [queryError, setQueryError] = useState("");
  const [queryExecutionMs, setQueryExecutionMs] = useState<number | null>(null);

  // Execute tab state
  const [execContractAddress, setExecContractAddress] = useState("");
  const [selectedExecTemplate, setSelectedExecTemplate] = useState("cw20_transfer");
  const [execMsg, setExecMsg] = useState(
    EXECUTE_TEMPLATES.cw20_transfer.replace("{SENDER}", account.addressBech32)
  );
  const [fundsSprx, setFundsSprx] = useState("");
  const [gasLimit, setGasLimit] = useState(500000);
  const [memo, setMemo] = useState("");
  const [isExecuting, setIsExecuting] = useState(false);
  const [execSuccessHash, setExecSuccessHash] = useState<string | null>(null);
  const [execError, setExecError] = useState("");
  const [copiedResult, setCopiedResult] = useState(false);

  const client = new SpraxClient(network);

  const handleQueryTemplateChange = (tmplKey: string) => {
    setSelectedQueryTemplate(tmplKey);
    const tmpl = QUERY_TEMPLATES[tmplKey] || "{\n  \n}";
    setQueryMsg(tmpl.replace("{SENDER}", account.addressBech32));
  };

  const handleExecTemplateChange = (tmplKey: string) => {
    setSelectedExecTemplate(tmplKey);
    const tmpl = EXECUTE_TEMPLATES[tmplKey] || "{\n  \n}";
    setExecMsg(tmpl.replace("{SENDER}", account.addressBech32));
  };

  const handleRunQuery = async (e: React.FormEvent) => {
    e.preventDefault();
    setQueryError("");
    setQueryResult(null);
    setQueryExecutionMs(null);

    const addr = queryContractAddress.trim();
    if (!addr || !AddressUtil.isValidAddress(addr)) {
      setQueryError("Please enter a valid contract address (sprax1... or 0x...).");
      return;
    }

    let parsedPayload: any;
    try {
      parsedPayload = JSON.parse(queryMsg);
    } catch {
      setQueryError("Invalid JSON in Query Message. Please ensure valid JSON format.");
      return;
    }

    setIsQuerying(true);
    const startTime = performance.now();
    try {
      const res = await client.queryContract(addr, parsedPayload, queryGas);
      setQueryResult(res);
      setQueryExecutionMs(Math.round(performance.now() - startTime));
    } catch (err: any) {
      setQueryError(err.message || "Failed to query contract");
    } finally {
      setIsQuerying(false);
    }
  };

  const handleRunExecute = async (e: React.FormEvent) => {
    e.preventDefault();
    setExecError("");
    setExecSuccessHash(null);

    const addr = execContractAddress.trim();
    if (!addr || !AddressUtil.isValidAddress(addr)) {
      setExecError("Please enter a valid contract address (sprax1... or 0x...).");
      return;
    }

    let parsedPayload: any;
    try {
      parsedPayload = JSON.parse(execMsg);
    } catch {
      setExecError("Invalid JSON in Execute Message. Please ensure valid JSON format.");
      return;
    }

    let fundsAtto = "0";
    if (fundsSprx.trim()) {
      try {
        fundsAtto = TransactionBuilder.sprxToAtto(fundsSprx.trim()).toString();
      } catch {
        setExecError("Invalid attached SPRX funds amount.");
        return;
      }
    }

    setIsExecuting(true);
    try {
      const nonce = await client.getAccountNonce(account.addressBech32);

      const signedTx = TransactionBuilder.signContractCall(
        {
          fromAddress: account.addressBech32,
          contractAddress: addr,
          msg: parsedPayload,
          fundsAtto,
          nonce,
          fee: {
            amountAtto: "1000000000000000", // 0.001 SPRX fee
            gasLimit,
          },
          memo,
        },
        network.chainId,
        privateKey,
        account.algorithm
      );

      const txHash = await client.broadcastTransaction(signedTx);
      setExecSuccessHash(txHash);
    } catch (err: any) {
      setExecError(err.message || "Failed to execute contract transaction");
    } finally {
      setIsExecuting(false);
    }
  };

  const copyResultToClipboard = () => {
    if (!queryResult) return;
    navigator.clipboard.writeText(JSON.stringify(queryResult, null, 2));
    setCopiedResult(true);
    setTimeout(() => setCopiedResult(false), 2000);
  };

  return (
    <div
      style={{
        maxWidth: 580,
        margin: "0 auto",
        padding: 24,
        borderRadius: 16,
        background: "#131722",
        color: "#fff",
        fontFamily: "sans-serif",
        border: "1px solid #1f293d",
      }}
    >
      {/* Title & Network Status */}
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 20 }}>
        <div>
          <h2 style={{ margin: "0 0 4px 0", fontSize: 20 }}>CosmWasm Smart Contracts</h2>
          <p style={{ margin: 0, fontSize: 13, color: "#9ca3af" }}>
            Interact with WebAssembly dApps & CW20/CW721 tokens
          </p>
        </div>
        <span
          style={{
            fontSize: 12,
            padding: "4px 8px",
            background: network.isTestnet ? "#ff980033" : "#4caf5033",
            color: network.isTestnet ? "#ff9800" : "#4caf50",
            borderRadius: 6,
            fontWeight: "bold",
          }}
        >
          {network.name}
        </span>
      </div>

      {/* Mode Navigation Tabs */}
      <div
        style={{
          display: "grid",
          gridTemplateColumns: "1fr 1fr",
          gap: 8,
          marginBottom: 20,
          background: "#0b0e14",
          padding: 4,
          borderRadius: 10,
        }}
      >
        <button
          type="button"
          onClick={() => setActiveTab("query")}
          style={{
            padding: "10px 14px",
            borderRadius: 8,
            border: "none",
            fontWeight: "bold",
            fontSize: 14,
            cursor: "pointer",
            background: activeTab === "query" ? "#2563eb" : "transparent",
            color: activeTab === "query" ? "#fff" : "#94a3b8",
            transition: "all 0.2s ease",
          }}
        >
          🔍 Query (Read Only)
        </button>
        <button
          type="button"
          onClick={() => setActiveTab("execute")}
          style={{
            padding: "10px 14px",
            borderRadius: 8,
            border: "none",
            fontWeight: "bold",
            fontSize: 14,
            cursor: "pointer",
            background: activeTab === "execute" ? "#8b5cf6" : "transparent",
            color: activeTab === "execute" ? "#fff" : "#94a3b8",
            transition: "all 0.2s ease",
          }}
        >
          ⚡ Execute (State Change)
        </button>
      </div>

      {/* TAB 1: QUERY CONTRACT */}
      {activeTab === "query" && (
        <form onSubmit={handleRunQuery}>
          {queryError && (
            <div
              style={{
                background: "#ef444422",
                border: "1px solid #ef4444",
                color: "#fca5a5",
                padding: 12,
                borderRadius: 8,
                fontSize: 13,
                marginBottom: 16,
              }}
            >
              {queryError}
            </div>
          )}

          {/* Contract Address */}
          <div style={{ marginBottom: 14 }}>
            <label style={{ display: "block", fontSize: 13, color: "#9ca3af", marginBottom: 6 }}>
              Contract Address
            </label>
            <input
              type="text"
              placeholder="sprax1... or 0x..."
              value={queryContractAddress}
              onChange={(e) => setQueryContractAddress(e.target.value)}
              style={{
                width: "100%",
                padding: 12,
                background: "#0b0e14",
                border: `1px solid ${
                  queryContractAddress && !AddressUtil.isValidAddress(queryContractAddress)
                    ? "#ef4444"
                    : "#2d3748"
                }`,
                borderRadius: 8,
                color: "#fff",
                boxSizing: "border-box",
                fontSize: 13,
              }}
              required
            />
          </div>

          {/* Template Selection */}
          <div style={{ marginBottom: 14 }}>
            <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 6 }}>
              <label style={{ fontSize: 13, color: "#9ca3af" }}>Query Template</label>
              <span style={{ fontSize: 11, color: "#60a5fa" }}>Pre-configured presets</span>
            </div>
            <select
              value={selectedQueryTemplate}
              onChange={(e) => handleQueryTemplateChange(e.target.value)}
              style={{
                width: "100%",
                padding: 10,
                background: "#0b0e14",
                border: "1px solid #2d3748",
                borderRadius: 8,
                color: "#fff",
                fontSize: 13,
                cursor: "pointer",
              }}
            >
              <option value="cw20_token_info">CW20: Token Info</option>
              <option value="cw20_balance">CW20: Balance of Connected Wallet</option>
              <option value="cw20_allowance">CW20: Allowance</option>
              <option value="cw20_minter">CW20: Minter Info</option>
              <option value="cw20_all_accounts">CW20: All Token Holders</option>
              <option value="cw721_contract_info">CW721: Contract Info</option>
              <option value="cw721_num_tokens">CW721: Total Minted Tokens</option>
              <option value="cw721_owner_of">CW721: Owner Of Token</option>
              <option value="custom">Custom JSON Query</option>
            </select>
          </div>

          {/* Query Message Editor */}
          <div style={{ marginBottom: 14 }}>
            <label style={{ display: "block", fontSize: 13, color: "#9ca3af", marginBottom: 6 }}>
              Query Message (JSON)
            </label>
            <textarea
              rows={5}
              value={queryMsg}
              onChange={(e) => setQueryMsg(e.target.value)}
              style={{
                width: "100%",
                padding: 12,
                background: "#0b0e14",
                border: "1px solid #2d3748",
                borderRadius: 8,
                color: "#38bdf8",
                fontFamily: "monospace",
                fontSize: 13,
                boxSizing: "border-box",
                resize: "vertical",
              }}
              required
            />
          </div>

          {/* Gas Limit */}
          <div style={{ marginBottom: 18 }}>
            <label style={{ display: "block", fontSize: 13, color: "#9ca3af", marginBottom: 6 }}>
              Query Gas Limit (Max 200,000)
            </label>
            <input
              type="number"
              min={1}
              max={200000}
              value={queryGas}
              onChange={(e) => setQueryGas(parseInt(e.target.value) || 200000)}
              style={{
                width: "100%",
                padding: 10,
                background: "#0b0e14",
                border: "1px solid #2d3748",
                borderRadius: 8,
                color: "#fff",
                boxSizing: "border-box",
                fontSize: 13,
              }}
            />
          </div>

          <button
            type="submit"
            disabled={isQuerying}
            style={{
              width: "100%",
              padding: 14,
              borderRadius: 10,
              background: isQuerying ? "#4b5563" : "#2563eb",
              color: "#fff",
              border: "none",
              fontWeight: "bold",
              fontSize: 15,
              cursor: isQuerying ? "not-allowed" : "pointer",
            }}
          >
            {isQuerying ? "Querying State..." : "Run Query"}
          </button>

          {/* Results Box */}
          {queryResult !== null && (
            <div
              style={{
                marginTop: 20,
                background: "#0b0e14",
                borderRadius: 10,
                padding: 16,
                border: "1px solid #10b981",
              }}
            >
              <div
                style={{
                  display: "flex",
                  justifyContent: "space-between",
                  alignItems: "center",
                  marginBottom: 10,
                }}
              >
                <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
                  <span style={{ fontSize: 13, fontWeight: "bold", color: "#10b981" }}>
                    ✓ Query Succeeded
                  </span>
                  {queryExecutionMs !== null && (
                    <span style={{ fontSize: 11, color: "#9ca3af" }}>({queryExecutionMs} ms)</span>
                  )}
                </div>
                <button
                  type="button"
                  onClick={copyResultToClipboard}
                  style={{
                    background: "#1f293d",
                    border: "none",
                    color: copiedResult ? "#10b981" : "#60a5fa",
                    padding: "4px 8px",
                    borderRadius: 6,
                    fontSize: 12,
                    cursor: "pointer",
                  }}
                >
                  {copiedResult ? "✓ Copied" : "Copy JSON"}
                </button>
              </div>
              <pre
                style={{
                  margin: 0,
                  fontSize: 12,
                  fontFamily: "monospace",
                  color: "#e2e8f0",
                  whiteSpace: "pre-wrap",
                  wordBreak: "break-all",
                  maxHeight: 260,
                  overflowY: "auto",
                }}
              >
                {JSON.stringify(queryResult, null, 2)}
              </pre>
            </div>
          )}
        </form>
      )}

      {/* TAB 2: EXECUTE CONTRACT */}
      {activeTab === "execute" && (
        <form onSubmit={handleRunExecute}>
          {execError && (
            <div
              style={{
                background: "#ef444422",
                border: "1px solid #ef4444",
                color: "#fca5a5",
                padding: 12,
                borderRadius: 8,
                fontSize: 13,
                marginBottom: 16,
              }}
            >
              {execError}
            </div>
          )}

          {execSuccessHash && (
            <div
              style={{
                background: "#064e3b",
                border: "1px solid #10b981",
                color: "#6ee7b7",
                padding: 14,
                borderRadius: 8,
                fontSize: 13,
                marginBottom: 16,
              }}
            >
              <div style={{ fontWeight: "bold", marginBottom: 4 }}>✓ Transaction Submitted!</div>
              <div style={{ fontFamily: "monospace", fontSize: 12, wordBreak: "break-all" }}>
                Hash: {execSuccessHash}
              </div>
            </div>
          )}

          {/* Contract Address */}
          <div style={{ marginBottom: 14 }}>
            <label style={{ display: "block", fontSize: 13, color: "#9ca3af", marginBottom: 6 }}>
              Contract Address
            </label>
            <input
              type="text"
              placeholder="sprax1... or 0x..."
              value={execContractAddress}
              onChange={(e) => setExecContractAddress(e.target.value)}
              style={{
                width: "100%",
                padding: 12,
                background: "#0b0e14",
                border: `1px solid ${
                  execContractAddress && !AddressUtil.isValidAddress(execContractAddress)
                    ? "#ef4444"
                    : "#2d3748"
                }`,
                borderRadius: 8,
                color: "#fff",
                boxSizing: "border-box",
                fontSize: 13,
              }}
              required
            />
          </div>

          {/* Template Selection */}
          <div style={{ marginBottom: 14 }}>
            <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 6 }}>
              <label style={{ fontSize: 13, color: "#9ca3af" }}>Execute Template</label>
              <span style={{ fontSize: 11, color: "#a855f7" }}>Quick Action</span>
            </div>
            <select
              value={selectedExecTemplate}
              onChange={(e) => handleExecTemplateChange(e.target.value)}
              style={{
                width: "100%",
                padding: 10,
                background: "#0b0e14",
                border: "1px solid #2d3748",
                borderRadius: 8,
                color: "#fff",
                fontSize: 13,
                cursor: "pointer",
              }}
            >
              <option value="cw20_transfer">CW20: Transfer Tokens</option>
              <option value="cw20_mint">CW20: Mint Tokens</option>
              <option value="cw20_burn">CW20: Burn Tokens</option>
              <option value="cw20_increase_allowance">CW20: Increase Allowance</option>
              <option value="cw20_send">CW20: Send with Contract Payload</option>
              <option value="custom">Custom Execute Message</option>
            </select>
          </div>

          {/* Execute Message Editor */}
          <div style={{ marginBottom: 14 }}>
            <label style={{ display: "block", fontSize: 13, color: "#9ca3af", marginBottom: 6 }}>
              Execute Message (JSON)
            </label>
            <textarea
              rows={5}
              value={execMsg}
              onChange={(e) => setExecMsg(e.target.value)}
              style={{
                width: "100%",
                padding: 12,
                background: "#0b0e14",
                border: "1px solid #2d3748",
                borderRadius: 8,
                color: "#c084fc",
                fontFamily: "monospace",
                fontSize: 13,
                boxSizing: "border-box",
                resize: "vertical",
              }}
              required
            />
          </div>

          {/* Optional Native Deposit & Gas Limit */}
          <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 12, marginBottom: 14 }}>
            <div>
              <label style={{ display: "block", fontSize: 13, color: "#9ca3af", marginBottom: 6 }}>
                Attach SPRX (Optional)
              </label>
              <input
                type="number"
                step="any"
                placeholder="0.0"
                value={fundsSprx}
                onChange={(e) => setFundsSprx(e.target.value)}
                style={{
                  width: "100%",
                  padding: 10,
                  background: "#0b0e14",
                  border: "1px solid #2d3748",
                  borderRadius: 8,
                  color: "#fff",
                  boxSizing: "border-box",
                  fontSize: 13,
                }}
              />
            </div>
            <div>
              <label style={{ display: "block", fontSize: 13, color: "#9ca3af", marginBottom: 6 }}>
                Gas Limit
              </label>
              <input
                type="number"
                min={100000}
                max={2000000}
                value={gasLimit}
                onChange={(e) => setGasLimit(parseInt(e.target.value) || 500000)}
                style={{
                  width: "100%",
                  padding: 10,
                  background: "#0b0e14",
                  border: "1px solid #2d3748",
                  borderRadius: 8,
                  color: "#fff",
                  boxSizing: "border-box",
                  fontSize: 13,
                }}
              />
            </div>
          </div>

          {/* Memo */}
          <div style={{ marginBottom: 18 }}>
            <label style={{ display: "block", fontSize: 13, color: "#9ca3af", marginBottom: 6 }}>
              Memo (Optional)
            </label>
            <input
              type="text"
              placeholder="Contract call memo"
              value={memo}
              onChange={(e) => setMemo(e.target.value)}
              style={{
                width: "100%",
                padding: 10,
                background: "#0b0e14",
                border: "1px solid #2d3748",
                borderRadius: 8,
                color: "#fff",
                boxSizing: "border-box",
                fontSize: 13,
              }}
            />
          </div>

          {/* Fee estimate */}
          <div
            style={{
              display: "flex",
              justifyContent: "space-between",
              fontSize: 12,
              color: "#9ca3af",
              marginBottom: 20,
              padding: "8px 12px",
              background: "#0b0e14",
              borderRadius: 8,
            }}
          >
            <span>Network Execution Fee:</span>
            <span style={{ color: "#fff", fontWeight: "bold" }}>0.001 SPRX</span>
          </div>

          <button
            type="submit"
            disabled={isExecuting}
            style={{
              width: "100%",
              padding: 14,
              borderRadius: 10,
              background: isExecuting ? "#4b5563" : "#8b5cf6",
              color: "#fff",
              border: "none",
              fontWeight: "bold",
              fontSize: 15,
              cursor: isExecuting ? "not-allowed" : "pointer",
            }}
          >
            {isExecuting ? "Signing & Broadcasting Tx..." : "Sign & Execute Contract Call"}
          </button>
        </form>
      )}
    </div>
  );
};
