export type Environment = 'LOCAL' | 'TESTNET' | 'MAINNET';

export type AdminRole = 'super_admin' | 'admin' | 'operations' | 'viewer';

export interface AdminUser {
  id: string;
  username: string;
  role: AdminRole;
  is_active: boolean;
  created_at: string | null;
  last_login: string | null;
}

export interface AdminProfile {
  id: string;
  username: string;
  role: AdminRole;
  permissions: string[];
  environment: Environment;
}

export interface DashboardKpis {
  current_block_height: number;
  total_transactions: number;
  active_validators: number;
  total_staked_sprx: string;
  indexer_height: number;
  indexer_lag: number;
  rpc_status: string;
  api_health: string;
  registered_users: number;
  active_sessions: number;
}

export interface SubsystemHealth {
  status: 'OPERATIONAL' | 'HEALTHY' | 'SYNCING' | 'DEGRADED' | 'OUTAGE' | 'TESTNET ONLY (PROD BLOCKED)';
  latency_ms?: number;
  lag?: number;
  connections?: number;
  queued_jobs?: number;
  uptime?: string;
}

export interface SystemService {
  name: string;
  type: string;
  status: string;
  uptime: string;
  latency?: string;
  lag?: number;
  active_clients?: number;
}

export interface AppUser {
  id: string;
  address: string;
  display_name: string | null;
  is_active: boolean;
  created_at: string | null;
}

export interface AppVersionConfig {
  platform: string;
  min_version: string;
  latest_version: string;
  recommended_version: string;
  force_update: boolean;
  maintenance_mode: boolean;
  maintenance_message: string;
  update_url_android: string;
}

export interface BlockchainStats {
  chain_id: string;
  environment: Environment;
  latest_height: number;
  latest_block_hash: string;
  total_transactions: number;
  validator_count: number;
  avg_block_time_ms: number;
  gas_limit_per_block: number;
  rpc_endpoint: string;
  consensus: string;
}

export interface BlockItem {
  height: number;
  hash: string;
  parent_hash: string | null;
  proposer: string | null;
  tx_count: number;
  gas_used: number;
  timestamp: string;
}

export interface TransactionItem {
  hash: string;
  block_height: number;
  sender: string;
  recipient: string | null;
  amount: string;
  fee: string;
  tx_type: string;
  status: string;
  timestamp: string;
}

export interface IndexerStatus {
  chain_height: number;
  indexed_height: number;
  lag_blocks: number;
  is_syncing: boolean;
  status: string;
  last_error: string | null;
  updated_at: string;
}

export interface MarketAsset {
  symbol: string;
  name: string;
  decimals: number;
  price_usd: string;
  change_24h: string;
  is_active: boolean;
}

export interface MarketPair {
  symbol: string;
  base: string;
  quote: string;
  last_price: string;
  is_active: boolean;
}

export interface DiscoverProject {
  id: string;
  name: string;
  category: string;
  description: string | null;
  website: string | null;
  is_active: boolean;
  is_featured: boolean;
  is_verified: boolean;
}

export interface ValidatorItem {
  id: string;
  address: string;
  name: string;
  total_stake: string;
  commission_rate: string;
  is_active: boolean;
  is_jailed: boolean;
  uptime_pct: string;
}

export interface PerpMarketItem {
  symbol: string;
  base: string;
  quote: string;
  mark_price: string;
  index_price: string;
  funding_rate: string;
  open_interest: string;
  max_leverage: number;
  is_active: boolean;
}

export interface FeatureFlagItem {
  name: string;
  is_enabled: boolean;
  description: string | null;
}

export interface AuditLogItem {
  id: string;
  actor: string;
  action: string;
  resource: string | null;
  result: string;
  metadata: Record<string, any> | null;
  created_at: string;
}

export interface TapMiningDashboard {
  total_players: number;
  total_points_issued: number;
  total_taps: number;
  banned_count: number;
  dau: number;
  total_sprx_claimed: number;
  total_claims_count: number;
  pending_claims_count: number;
  top_earners: { telegram_id: number; username: string | null; points: number }[];
}

export interface TapMiningPlayer {
  telegram_id: number;
  username: string | null;
  points: number;
  energy: number;
  level: string;
  is_banned: boolean;
  taps_total: number;
  checkin_streak: number;
  referral_code: string;
  payout_address: string | null;
  total_sprx_claimed: number;
  created_at: string;
  last_tap_at: string | null;
}

export interface TapMiningClaim {
  id: string;
  tap_miner_id: string;
  telegram_id: number;
  telegram_username: string | null;
  points_spent: number;
  sprx_amount: number;
  recipient_address: string;
  status: 'pending' | 'completed' | 'failed';
  tx_hash: string | null;
  error_message: string | null;
  created_at: string;
  completed_at: string | null;
}

export interface FaucetStatus {
  online: boolean;
  network?: string;
  faucet_address?: string;
  available_balance?: string;
  total_disbursed?: string;
  total_claims_count?: number;
  max_payout_per_request?: number;
  rate_limit_window_secs?: number;
  points_per_sprx?: number;
  min_claim_points?: number;
  claim_cooldown_seconds?: number;
}

export interface TapMiningActivityItem {
  event_type: string;
  points_delta: number;
  detail: string | null;
  created_at: string;
}

export type TapMiningTaskType = 'manual' | 'telegram_join' | 'milestone';

export interface TapMiningTask {
  id: string;
  title: string;
  description: string | null;
  task_type: TapMiningTaskType;
  reward_points: number;
  action_url: string | null;
  verify_target: string | null;
  verify_value: number | null;
  is_active: boolean;
  sort_order: number;
  completions: number;
  created_at: string;
}

export interface TapMiningPlayerDetail extends TapMiningPlayer {
  energy_cap_level: number;
  regen_speed_level: number;
  multitap_level: number;
  referral_earnings: number;
  referred_by_id: string | null;
  activity: TapMiningActivityItem[];
}

export interface SystemAnnouncementItem {
  id: string;
  title: string;
  body: string;
  type: string;
  is_active: boolean;
  created_at: string | null;
}
