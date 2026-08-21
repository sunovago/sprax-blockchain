import React, { useState } from 'react';
import { useAuth } from './context/AuthContext';
import { AdminShell } from './components/layout/AdminShell';
import { LoginPage } from './pages/LoginPage';
import { DashboardPage } from './pages/DashboardPage';
import { AppUsersPage } from './pages/AppUsersPage';
import { AppVersionsPage } from './pages/AppVersionsPage';
import { BlockchainOverviewPage } from './pages/BlockchainOverviewPage';
import { BlocksPage } from './pages/BlocksPage';
import { TransactionsPage } from './pages/TransactionsPage';
import { IndexerMonitorPage } from './pages/IndexerMonitorPage';
import { MarketsPage } from './pages/MarketsPage';
import { DiscoverPage } from './pages/DiscoverPage';
import { SearchIndexPage } from './pages/SearchIndexPage';
import { ValidatorsPage } from './pages/ValidatorsPage';
import { StakingPage } from './pages/StakingPage';
import { PerpsRiskPage } from './pages/PerpsRiskPage';
import { NotificationsPage } from './pages/NotificationsPage';
import { OperationsPage } from './pages/OperationsPage';
import { SecurityAdminsPage } from './pages/SecurityAdminsPage';
import { AuditLogsPage } from './pages/AuditLogsPage';
import { FeatureFlagsPage } from './pages/FeatureFlagsPage';

export const App: React.FC = () => {
  const { isAuthenticated, isLoading } = useAuth();
  const [activeTab, setActiveTab] = useState('dashboard');

  if (isLoading) {
    return (
      <div className="min-h-screen w-screen bg-[#0B0F19] flex items-center justify-center text-primary-400 font-mono text-xs">
        <div className="flex items-center gap-2">
          <div className="w-4 h-4 border-2 border-primary-500 border-t-transparent rounded-full animate-spin" />
          Initializing SPRX Admin Center...
        </div>
      </div>
    );
  }

  if (!isAuthenticated) {
    return <LoginPage />;
  }

  const renderContent = () => {
    switch (activeTab) {
      case 'dashboard':
        return <DashboardPage />;
      case 'app_users':
        return <AppUsersPage />;
      case 'app_versions':
        return <AppVersionsPage />;
      case 'blockchain':
        return <BlockchainOverviewPage />;
      case 'blocks':
        return <BlocksPage />;
      case 'transactions':
        return <TransactionsPage />;
      case 'indexer':
        return <IndexerMonitorPage />;
      case 'markets':
        return <MarketsPage />;
      case 'discover':
        return <DiscoverPage />;
      case 'search_index':
        return <SearchIndexPage />;
      case 'validators':
        return <ValidatorsPage />;
      case 'staking':
        return <StakingPage />;
      case 'perps_risk':
        return <PerpsRiskPage />;
      case 'notifications':
        return <NotificationsPage />;
      case 'operations':
        return <OperationsPage />;
      case 'security_admins':
        return <SecurityAdminsPage />;
      case 'audit_logs':
        return <AuditLogsPage />;
      case 'feature_flags':
        return <FeatureFlagsPage />;
      default:
        return <DashboardPage />;
    }
  };

  return (
    <AdminShell activeTab={activeTab} onSelectTab={setActiveTab}>
      {renderContent()}
    </AdminShell>
  );
};
