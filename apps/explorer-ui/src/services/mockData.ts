import {
  IndexedBlock,
  IndexedTx,
  IndexedValidator,
  NetworkStats,
  SmartContract,
} from "@/types";

export const MOCK_GENESIS_VALIDATORS: IndexedValidator[] = [
  {
    operator_address: "spraxvaloper1q8x9y7w6e5r4t3y2u1i0o9p8a7s6d5f4g3h2j1",
    moniker: "SPRX Core Genesis Alpha",
    voting_power: 3500000,
    voting_power_percentage: 35.0,
    tokens: "3,500,000 SPRX",
    commission_rate: 0.05,
    status: "Active",
    is_tombstoned: false,
    missed_blocks_count: 0,
    uptime_percentage: 99.98,
    blocks_proposed_count: 48920,
  },
  {
    operator_address: "spraxvaloper1a2s3d4f5g6h7j8k9l0z1x2c3v4b5n6m7q8w9e0",
    moniker: "Global Real-World X Node",
    voting_power: 2800000,
    voting_power_percentage: 28.0,
    tokens: "2,800,000 SPRX",
    commission_rate: 0.04,
    status: "Active",
    is_tombstoned: false,
    missed_blocks_count: 1,
    uptime_percentage: 99.95,
    blocks_proposed_count: 39150,
  },
  {
    operator_address: "spraxvaloper1z9y8x7w6v5u4t3s2r1q0p9o8n7m6l5k4j3h2g1",
    moniker: "Nexus Staking Labs",
    voting_power: 1900000,
    voting_power_percentage: 19.0,
    tokens: "1,900,000 SPRX",
    commission_rate: 0.05,
    status: "Active",
    is_tombstoned: false,
    missed_blocks_count: 3,
    uptime_percentage: 99.89,
    blocks_proposed_count: 26400,
  },
  {
    operator_address: "spraxvaloper1m0n9b8v7c6x5z4l3k2j1h0g9f8d7s6a5q4w3e2",
    moniker: "Hyperion Validator",
    voting_power: 1200000,
    voting_power_percentage: 12.0,
    tokens: "1,200,000 SPRX",
    commission_rate: 0.08,
    status: "Active",
    is_tombstoned: false,
    missed_blocks_count: 6,
    uptime_percentage: 99.72,
    blocks_proposed_count: 16800,
  },
  {
    operator_address: "spraxvaloper1p1o2i3u4y5t6r7e8w9q0a1s2d3f4g5h6j7k8l9",
    moniker: "Antares Infrastructure",
    voting_power: 600000,
    voting_power_percentage: 6.0,
    tokens: "600,000 SPRX",
    commission_rate: 0.10,
    status: "Active",
    is_tombstoned: false,
    missed_blocks_count: 12,
    uptime_percentage: 99.45,
    blocks_proposed_count: 8400,
  },
];

export const MOCK_GENESIS_BLOCKS: IndexedBlock[] = Array.from({ length: 20 }, (_, i) => {
  const height = 8245920 - i;
  const now = Math.floor(Date.now() / 1000) - i * 2;
  const proposer = MOCK_GENESIS_VALIDATORS[i % MOCK_GENESIS_VALIDATORS.length].operator_address;
  return {
    height,
    hash: `0x7f8a9b2c3d4e5f60718293a4b5c6d7e8f901a2b3c4d5e6f708192a3b4c5d6e7${i % 10}`,
    parent_hash: `0x6e7d8c9b0a1f2e3d4c5b6a708192a3b4c5d6e7f8091a2b3c4d5e6f708192a3b${(i + 1) % 10}`,
    chain_id: "sprax-mainnet-1",
    timestamp_unix_secs: now,
    proposer,
    txs_count: (i * 7 + 3) % 18 + 1,
    txs_root: "0x11223344556677889900aabbccddeeff00112233445566778899aabbccddeeff",
    state_root: "0xabcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789",
    gas_used: 145000 + (i * 12500),
    block_size_bytes: 4200 + (i * 350),
  };
});

export const MOCK_GENESIS_TXS: IndexedTx[] = Array.from({ length: 30 }, (_, i) => {
  const height = 8245920 - Math.floor(i / 2);
  const now = Math.floor(Date.now() / 1000) - i * 3;
  const types = ["Transfer", "Delegate", "ContractCall", "Transfer", "Undelegate", "Transfer"];
  const msgType = types[i % types.length];
  const senders = [
    "sprax1qpzry9x8gf2tvdw0s3jn54khce6mua7l4w3e2r",
    "sprax1234567890abcdef1234567890abcdef12345678",
    "sprax1valoper1q8x9y7w6e5r4t3y2u1i0o9p8a7s6d5f4",
    "sprax1987654321fedcba987654321fedcba987654321",
  ];
  const recipients = [
    "sprax18888888888888888888888888888888888888888",
    "sprax17777777777777777777777777777777777777777",
    "sprax1contract99999999999999999999999999999999",
    "sprax15555555555555555555555555555555555555555",
  ];

  return {
    tx_hash: `0x${(i + 10).toString(16).padStart(2, "0")}e4b2c1d9f8a7e6b5c4d3e2f1a0b9c8d7e6f5a4b3c2d1e0f9a8b7c6d5e4f3a2b1`,
    block_height: height,
    block_hash: `0x7f8a9b2c3d4e5f60718293a4b5c6d7e8f901a2b3c4d5e6f708192a3b4c5d6e7${i % 5}`,
    sender: senders[i % senders.length],
    recipient: msgType === "ContractCall" ? "sprax1contract99999999999999999999999999999999" : recipients[i % recipients.length],
    message_type: msgType,
    amount: `${((i + 1) * 12.5).toFixed(2)} SPRX`,
    fee_amount: "0.0025 SPRX",
    nonce: 100 + i,
    memo: i % 3 === 0 ? "SPRX Staking rewards delegation" : i % 5 === 0 ? "DeFi liquidity provision" : "",
    success: i !== 7,
    gas_used: msgType === "ContractCall" ? 85000 : 21000,
    timestamp_unix_secs: now,
  };
});

export const MOCK_GENESIS_STATS: NetworkStats = {
  chain_id: "sprax-mainnet-1",
  latest_height: 8245920,
  latest_block_hash: "0x7f8a9b2c3d4e5f60718293a4b5c6d7e8f901a2b3c4d5e6f708192a3b4c5d6e70",
  total_transactions: 14892040,
  total_accounts: 342150,
  active_validators_count: 5,
  total_bonded_tokens: "10,000,000 SPRX",
  avg_block_time_seconds: 2.1,
  current_tps: 842.5,
  latest_state_root: "0xabcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789",
};

export const MOCK_GENESIS_CONTRACTS: SmartContract[] = [
  {
    address: "sprax1contract99999999999999999999999999999999",
    creator: "sprax1qpzry9x8gf2tvdw0s3jn54khce6mua7l4w3e2r",
    created_height: 10420,
    bytecode_size: 18450,
    tx_count: 4920,
    verified: true,
    compiler_version: "rustc 1.78.0 / sprax-wasm 0.4.2",
    abi: JSON.stringify([
      { name: "deposit", inputs: [{ name: "amount", type: "Amount" }], outputs: [] },
      { name: "withdraw", inputs: [{ name: "amount", type: "Amount" }], outputs: [] },
      { name: "get_balance", inputs: [{ name: "account", type: "Address" }], outputs: [{ type: "Amount" }] },
    ], null, 2),
    source_code: `// SPRX Decentralized Vault Contract
use sprax_wasm_sdk::prelude::*;

#[sprax_contract]
pub struct VaultContract {
    balances: Map<Address, Amount>,
}

#[sprax_contract_impl]
impl VaultContract {
    pub fn deposit(&mut self, amount: Amount) {
        let sender = msg::sender();
        let cur = self.balances.get(&sender).unwrap_or(0);
        self.balances.insert(sender, cur + amount);
    }
}`,
  },
  {
    address: "sprax1contract88888888888888888888888888888888",
    creator: "sprax1234567890abcdef1234567890abcdef12345678",
    created_height: 25900,
    bytecode_size: 24100,
    tx_count: 12840,
    verified: true,
    compiler_version: "rustc 1.78.0 / sprax-wasm 0.4.2",
    abi: JSON.stringify([
      { name: "swap", inputs: [{ name: "token_in", type: "Address" }, { name: "amount", type: "Amount" }], outputs: [{ type: "Amount" }] },
    ], null, 2),
  },
];
