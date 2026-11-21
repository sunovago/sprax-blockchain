import {
  AdminProfile,
  DashboardKpis,
  SystemService,
  AppUser,
  AppVersionConfig,
  BlockchainStats,
  BlockItem,
  TransactionItem,
  IndexerStatus,
  MarketAsset,
  MarketPair,
  DiscoverProject,
  ValidatorItem,
  PerpMarketItem,
  FeatureFlagItem,
  AuditLogItem,
  SystemAnnouncementItem,
  AdminUser,
  TapMiningDashboard,
  TapMiningPlayer,
  TapMiningPlayerDetail,
  TapMiningTask,
  TapMiningClaim,
  FaucetStatus,
} from '../types';

const API_BASE = (import.meta as any).env?.VITE_API_URL || '/api/v1';

class ApiClient {
  private getToken(): string | null {
    return localStorage.getItem('sprax_admin_token');
  }

  private async request<T>(path: string, options: RequestInit = {}): Promise<T> {
    const token = this.getToken();
    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
      ...(options.headers as Record<string, string>),
    };

    if (token) {
      headers['Authorization'] = `Bearer ${token}`;
    }

    const response = await fetch(`${API_BASE}${path}`, {
      ...options,
      headers,
    });

    if (response.status === 401) {
      localStorage.removeItem('sprax_admin_token');
      localStorage.removeItem('sprax_admin_user');
      window.dispatchEvent(new Event('sprax_auth_changed'));
      throw new Error('Unauthorized. Please log in.');
    }

    const json = await response.json();
    if (!response.ok) {
      const errMsg = json.error?.message || json.detail || 'Request failed';
      throw new Error(errMsg);
    }

    return json.data !== undefined ? json.data : json;
  }

  // Auth
  async login(username: string, password: string): Promise<{ access_token: string; role: string; username: string }> {
    const res = await fetch(`${API_BASE}/admin/login`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ username, password }),
    });
    const json = await res.json();
    if (!res.ok) {
      throw new Error(json.detail || 'Login failed');
    }
    const data = json.data;
    localStorage.setItem('sprax_admin_token', data.access_token);
    localStorage.setItem('sprax_admin_user', JSON.stringify({ username: data.username, role: data.role }));
    window.dispatchEvent(new Event('sprax_auth_changed'));
    return data;
  }

  logout(): void {
    localStorage.removeItem('sprax_admin_token');
    localStorage.removeItem('sprax_admin_user');
    window.dispatchEvent(new Event('sprax_auth_changed'));
  }

  async getMe(): Promise<AdminProfile> {
    return this.request<AdminProfile>('/admin/me');
  }

  // Dashboard
  async getDashboard(): Promise<{
    environment: string;
    chain_id: string;
    kpis: DashboardKpis;
    charts: { tx_volume_24h: { time: string; txs: number; blocks: number }[] };
    subsystems: Record<string, any>;
  }> {
    return this.request('/admin/dashboard');
  }

  async getSystemHealth(): Promise<{ status: string; environment: string; services: SystemService[] }> {
    return this.request('/admin/system/health');
  }

  // App Management
  async getAppUsers(limit = 50, offset = 0): Promise<{ total: number; users: AppUser[] }> {
    return this.request(`/admin/app/users?limit=${limit}&offset=${offset}`);
  }

  async getAppVersions(): Promise<AppVersionConfig> {
    return this.request('/admin/app/versions');
  }

  async updateAppVersions(config: Partial<AppVersionConfig>): Promise<{ status: string; config: AppVersionConfig }> {
    return this.request('/admin/app/versions', {
      method: 'PATCH',
      body: JSON.stringify(config),
    });
  }

  // Blockchain
  async getBlockchainStats(): Promise<BlockchainStats> {
    return this.request('/admin/blockchain/stats');
  }

  async getBlocks(limit = 20): Promise<BlockItem[]> {
    const res = await this.request<{ blocks: BlockItem[] }>(`/explorer/blocks?limit=${limit}`);
    return res.blocks || [];
  }

  async getTransactions(limit = 20): Promise<TransactionItem[]> {
    const res = await this.request<{ transactions: TransactionItem[] }>(`/explorer/transactions?limit=${limit}`);
    return res.transactions || [];
  }

  // Indexer
  async getIndexerStatus(): Promise<IndexerStatus> {
    return this.request('/admin/indexer/status');
  }

  async retryIndexer(): Promise<{ message: string }> {
    return this.request('/admin/indexer/retry', { method: 'POST' });
  }

  // Markets
  async getMarkets(): Promise<{ assets: MarketAsset[]; pairs: MarketPair[]; providers: any[] }> {
    return this.request('/admin/markets');
  }

  // Discover
  async getDiscover(): Promise<DiscoverProject[]> {
    return this.request('/admin/discover');
  }

  async updateDiscoverProject(id: string, updates: Partial<DiscoverProject>): Promise<{ status: string }> {
    return this.request(`/admin/discover/${id}`, {
      method: 'PATCH',
      body: JSON.stringify(updates),
    });
  }

  // Validators
  async getValidators(): Promise<ValidatorItem[]> {
    return this.request('/admin/validators');
  }

  // Perps
  async getPerpsStatus(): Promise<{
    environment_mode: string;
    is_production_active: boolean;
    active_markets_count: number;
    open_positions_count: number;
    open_orders_count: number;
    markets: PerpMarketItem[];
    risk_alerts: any[];
  }> {
    return this.request('/admin/perps/status');
  }

  // Notifications
  async getNotifications(): Promise<SystemAnnouncementItem[]> {
    return this.request('/admin/notifications');
  }

  async broadcastNotification(data: { title: string; body: string; type: string; action_url?: string }): Promise<{ message: string; id: string }> {
    return this.request('/admin/notifications/broadcast', {
      method: 'POST',
      body: JSON.stringify(data),
    });
  }

  // Feature Flags
  async getFeatureFlags(): Promise<FeatureFlagItem[]> {
    return this.request('/admin/feature-flags');
  }

  async updateFeatureFlag(name: string, is_enabled: boolean): Promise<{ name: string; is_enabled: boolean }> {
    return this.request(`/admin/feature-flags/${name}`, {
      method: 'PATCH',
      body: JSON.stringify({ is_enabled }),
    });
  }

  // Security & Audit
  async getAuditLogs(limit = 50): Promise<{ total: number; logs: AuditLogItem[] }> {
    return this.request(`/admin/audit-logs?limit=${limit}`);
  }

  async getAdmins(): Promise<AdminUser[]> {
    return this.request('/admin/admins');
  }

  async createAdmin(data: { username: string; password: string; role: string }): Promise<any> {
    return this.request('/admin/admins', {
      method: 'POST',
      body: JSON.stringify(data),
    });
  }

  // Tap Mining
  async getTapMiningDashboard(): Promise<TapMiningDashboard> {
    return this.request('/admin/tap-mining/dashboard');
  }

  async getTapMiningPlayers(search?: string): Promise<{ total: number; players: TapMiningPlayer[] }> {
    const query = search ? `?search=${encodeURIComponent(search)}` : '';
    return this.request(`/admin/tap-mining/players${query}`);
  }

  async getTapMiningPlayer(telegramId: number): Promise<TapMiningPlayerDetail> {
    return this.request(`/admin/tap-mining/players/${telegramId}`);
  }

  async banTapMiningPlayer(telegramId: number): Promise<{ telegram_id: number; is_banned: boolean }> {
    return this.request(`/admin/tap-mining/players/${telegramId}/ban`, { method: 'POST' });
  }

  async unbanTapMiningPlayer(telegramId: number): Promise<{ telegram_id: number; is_banned: boolean }> {
    return this.request(`/admin/tap-mining/players/${telegramId}/unban`, { method: 'POST' });
  }

  async adjustTapMiningPlayer(
    telegramId: number,
    data: { points_delta: number; energy_delta: number; reason: string },
  ): Promise<{ telegram_id: number; points: number; energy: number }> {
    return this.request(`/admin/tap-mining/players/${telegramId}/adjust`, {
      method: 'POST',
      body: JSON.stringify(data),
    });
  }

  async getTapMiningTasks(): Promise<TapMiningTask[]> {
    return this.request('/admin/tap-mining/tasks');
  }

  async createTapMiningTask(data: {
    title: string;
    description?: string;
    task_type: string;
    reward_points: number;
    action_url?: string;
    verify_target?: string;
    verify_value?: number;
    is_active?: boolean;
    sort_order?: number;
  }): Promise<TapMiningTask> {
    return this.request('/admin/tap-mining/tasks', {
      method: 'POST',
      body: JSON.stringify(data),
    });
  }

  async updateTapMiningTask(id: string, data: Partial<TapMiningTask>): Promise<TapMiningTask> {
    return this.request(`/admin/tap-mining/tasks/${id}`, {
      method: 'PATCH',
      body: JSON.stringify(data),
    });
  }

  async getTapMiningClaims(params?: {
    status?: string;
    search?: string;
    limit?: number;
    offset?: number;
  }): Promise<{ total: number; claims: TapMiningClaim[] }> {
    const q = new URLSearchParams();
    if (params?.status) q.set('status', params.status);
    if (params?.search) q.set('search', params.search);
    if (params?.limit) q.set('limit', String(params.limit));
    if (params?.offset) q.set('offset', String(params.offset));
    const qs = q.toString() ? `?${q.toString()}` : '';
    return this.request(`/admin/tap-mining/claims${qs}`);
  }

  async getFaucetStatus(): Promise<FaucetStatus> {
    return this.request('/admin/tap-mining/faucet/status');
  }

  async retryTapMiningClaim(claimId: string): Promise<{ status: string; tx_hash: string | null }> {
    return this.request(`/admin/tap-mining/claims/${claimId}/retry`, {
      method: 'POST',
    });
  }
}

export const api = new ApiClient();
