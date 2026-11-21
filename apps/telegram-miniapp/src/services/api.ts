"use client";

const API_BASE = "/api/v1/tap-mining";
const TOKEN_KEY = "sprax_tap_mining_token";

export interface TapMiningProfile {
  telegram_id: number;
  username: string | null;
  first_name: string | null;
  last_name: string | null;
  photo_url: string | null;
  points: number;
  energy: number;
  energy_cap: number;
  energy_regen_seconds: number;
  points_per_tap: number;
  next_energy_in_seconds: number;
  taps_total: number;
  checkin_streak: number;
  longest_checkin_streak: number;
  checked_in_today: boolean;
  referral_code: string;
  payout_address: string | null;
  total_sprx_claimed: number;
  level: LevelInfo;
  created_at: string;
}

export interface LevelInfo {
  name: string;
  index: number;
  min_points: number;
  next_threshold: number | null;
  progress: number;
}

export interface ActivityEntry {
  event_type:
    | "checkin"
    | "referral_joined"
    | "referral_bonus"
    | "upgrade_purchased"
    | "level_up"
    | "task_completed"
    | "wallet_linked"
    | "tokens_claimed";
  points_delta: number;
  detail: string | null;
  created_at: string;
}

export interface ClaimInfo {
  payout_address: string | null;
  points: number;
  min_points_required: number;
  points_per_sprx: number;
  claimable_sprx: number;
  points_to_debit: number;
  can_claim: boolean;
  cooldown_remaining_seconds: number;
  last_claimed_at: string | null;
  total_sprx_claimed: number;
}

export interface ClaimTokensResult {
  claim_id: string;
  status: string;
  sprx_amount: number;
  points_debited: number;
  remaining_points: number;
  tx_hash: string | null;
  recipient_address: string;
}

export interface ClaimHistoryItem {
  id: string;
  points_spent: number;
  sprx_amount: number;
  recipient_address: string;
  status: "pending" | "completed" | "failed";
  tx_hash: string | null;
  error_message: string | null;
  created_at: string;
  completed_at: string | null;
}

export type TaskType = "manual" | "telegram_join" | "milestone";
export type TaskStatus = "available" | "locked" | "completed";

export interface TaskItem {
  id: string;
  title: string;
  description: string | null;
  task_type: TaskType;
  reward_points: number;
  action_url: string | null;
  status: TaskStatus;
}

export interface TapResult {
  points: number;
  energy: number;
  energy_cap: number;
  points_earned: number;
  taps_accepted: number;
  taps_rejected: number;
}

export interface CheckinResult {
  points: number;
  checkin_streak: number;
  longest_checkin_streak: number;
  reward_earned: number;
}

export interface LeaderboardEntry {
  rank: number;
  telegram_id: number;
  username: string | null;
  points: number;
}

export interface LeaderboardResult {
  leaderboard: LeaderboardEntry[];
  me: { rank: number; points: number };
}

export interface ReferralInfo {
  referral_code: string;
  referral_link: string | null;
  referred_count: number;
  referral_earnings: number;
}

export type UpgradeType = "energy_cap" | "regen_speed" | "multitap";

export interface UpgradeInfo {
  type: UpgradeType;
  level: number;
  max_level: number;
  effective_value: number;
  next_cost: number | null;
}

export interface UpgradesResult {
  upgrades: UpgradeInfo[];
  points: number;
}

export interface PurchaseUpgradeResult {
  type: UpgradeType;
  level: number;
  points: number;
}

class TapMiningApiClient {
  getToken(): string | null {
    if (typeof window === "undefined") return null;
    return sessionStorage.getItem(TOKEN_KEY);
  }

  private setToken(token: string): void {
    sessionStorage.setItem(TOKEN_KEY, token);
  }

  clearToken(): void {
    sessionStorage.removeItem(TOKEN_KEY);
  }

  private async request<T>(path: string, options: RequestInit = {}): Promise<T> {
    const token = this.getToken();
    const headers: Record<string, string> = {
      "Content-Type": "application/json",
      ...(options.headers as Record<string, string>),
    };
    if (token) {
      headers["Authorization"] = `Bearer ${token}`;
    }

    const res = await fetch(`${API_BASE}${path}`, { ...options, headers });
    const json = await res.json();
    if (!res.ok) {
      const message = json?.error?.message || "Request failed";
      throw new Error(message);
    }
    return json.data as T;
  }

  async authWithTelegram(
    initData: string,
    referralCode?: string | null,
  ): Promise<{ access_token: string; profile: TapMiningProfile }> {
    const data = await this.request<{ access_token: string; token_type: string; profile: TapMiningProfile }>(
      "/auth",
      { method: "POST", body: JSON.stringify({ init_data: initData, referral_code: referralCode || undefined }) },
    );
    this.setToken(data.access_token);
    return data;
  }

  async getProfile(): Promise<TapMiningProfile> {
    return this.request<TapMiningProfile>("/profile");
  }

  async sendTaps(tapCount: number): Promise<TapResult> {
    return this.request<TapResult>("/tap", {
      method: "POST",
      body: JSON.stringify({ tap_count: tapCount }),
    });
  }

  async checkin(): Promise<CheckinResult> {
    return this.request<CheckinResult>("/checkin", { method: "POST" });
  }

  async getLeaderboard(limit = 50): Promise<LeaderboardResult> {
    return this.request<LeaderboardResult>(`/leaderboard?limit=${limit}`);
  }

  async getReferral(): Promise<ReferralInfo> {
    return this.request<ReferralInfo>("/referral");
  }

  async getUpgrades(): Promise<UpgradesResult> {
    return this.request<UpgradesResult>("/upgrades");
  }

  async purchaseUpgrade(type: UpgradeType): Promise<PurchaseUpgradeResult> {
    return this.request<PurchaseUpgradeResult>(`/upgrades/${type}`, { method: "POST" });
  }

  async getActivity(limit = 20): Promise<ActivityEntry[]> {
    return this.request<ActivityEntry[]>(`/activity?limit=${limit}`);
  }

  async getTasks(): Promise<TaskItem[]> {
    return this.request<TaskItem[]>("/tasks");
  }

  async claimTask(taskId: string): Promise<{ points: number; reward_earned: number }> {
    return this.request(`/tasks/${taskId}/claim`, { method: "POST" });
  }

  async linkWallet(payoutAddress: string): Promise<{ payout_address: string; message: string }> {
    return this.request("/link-wallet", {
      method: "POST",
      body: JSON.stringify({ payout_address: payoutAddress }),
    });
  }

  async getClaimInfo(): Promise<ClaimInfo> {
    return this.request<ClaimInfo>("/claim-info");
  }

  async claimTokens(pointsToClaim?: number): Promise<ClaimTokensResult> {
    return this.request<ClaimTokensResult>("/claim-tokens", {
      method: "POST",
      body: JSON.stringify(pointsToClaim ? { points_to_claim: pointsToClaim } : {}),
    });
  }

  async getClaims(limit = 20): Promise<ClaimHistoryItem[]> {
    return this.request<ClaimHistoryItem[]>(`/claims?limit=${limit}`);
  }
}

export const tapMiningApi = new TapMiningApiClient();
