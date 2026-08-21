/**
 * Sprax Blockchain Explorer Type Definitions
 * Scalable Protocol for Real-world X (SPRX)
 */

export type Network = "mainnet" | "testnet" | "local";
export type Currency = "USD" | "INR" | "EUR" | "GBP" | "JPY";
export type ThemeMode = "dark" | "light" | "system";

export interface PaginatedResponse<T> {
  items: T[];
  total: number;
  limit: number;
  offset: number;
  has_more: boolean;
}

export interface IndexedBlock {
  height: number;
  hash: string;
  parent_hash: string;
  chain_id: string;
  timestamp_unix_secs: number;
  proposer: string;
  txs_count: number;
  txs_root: string;
  state_root: string;
  gas_used: number;
  block_size_bytes: number;
}

export interface TxMessage {
  type: string;
  sender: string;
  recipient?: string;
  amount?: string;
  data?: string;
}

export interface IndexedTx {
  tx_hash: string;
  block_height: number;
  block_hash: string;
  sender: string;
  recipient?: string | null;
  message_type: string;
  amount: string;
  fee_amount: string;
  nonce: number;
  memo: string;
  success: boolean;
  gas_used: number;
  timestamp_unix_secs: number;
  raw_messages?: TxMessage[];
}

export interface IndexedAccount {
  address: string;
  address_hex: string;
  balance: string;
  balance_sprx: string;
  nonce: number;
  tx_count: number;
  first_seen_height: number;
  last_active_height: number;
}

export interface IndexedValidator {
  operator_address: string;
  moniker: string;
  voting_power: number;
  voting_power_percentage: number;
  tokens: string;
  commission_rate: number;
  status: "Active" | "Inactive" | "Jailed";
  is_tombstoned: boolean;
  missed_blocks_count: number;
  uptime_percentage: number;
  blocks_proposed_count: number;
}

export interface NetworkStats {
  chain_id: string;
  latest_height: number;
  latest_block_hash: string;
  total_transactions: number;
  total_accounts: number;
  active_validators_count: number;
  total_bonded_tokens: string;
  avg_block_time_seconds: number;
  current_tps: number;
  latest_state_root: string;
}

export interface SmartContract {
  address: string;
  creator: string;
  created_height: number;
  bytecode_size: number;
  tx_count: number;
  verified: boolean;
  compiler_version?: string;
  source_code?: string;
  abi?: string;
}

export type SearchResultType = "Block" | "Transaction" | "Address" | "Validator" | "Contract";

export type SearchResult =
  | { type: "Block"; data: IndexedBlock }
  | { type: "Transaction"; data: IndexedTx }
  | { type: "Address"; data: IndexedAccount }
  | { type: "Validator"; data: IndexedValidator }
  | { type: "Contract"; data: SmartContract };

export interface DelegationInfo {
  validator_address: string;
  validator_moniker: string;
  delegator_address: string;
  shares: string;
  amount_sprx: string;
  rewards_sprx: string;
}

export interface StakingOverview {
  total_staked_sprx: string;
  staking_ratio_percentage: number;
  active_validators_count: number;
  unbonding_period_days: number;
  estimated_annual_metric_rate: number;
}

export interface CurrencyRate {
  symbol: string;
  prefix: string;
  ratePerSprx: number;
}
