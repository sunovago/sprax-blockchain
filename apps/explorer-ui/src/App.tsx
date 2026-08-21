import React, { useEffect, useState, useCallback } from "react";
import { Currency, IndexedAccount, IndexedBlock, IndexedTx, IndexedValidator, Network, NetworkStats, SmartContract, ThemeMode } from "@/types";
import { apiService } from "@/services/api";
import { AppHeader } from "@/components/AppHeader";
import { AppFooter } from "@/components/AppFooter";
import { SearchModal } from "@/components/SearchModal";
import { ErrorBoundary } from "@/components/ErrorBoundary";

// Pages
import { HomePage } from "@/pages/HomePage";
import { BlocksPage } from "@/pages/BlocksPage";
import { BlockDetailsPage } from "@/pages/BlockDetailsPage";
import { TransactionsPage } from "@/pages/TransactionsPage";
import { TxDetailsPage } from "@/pages/TxDetailsPage";
import { AddressPage } from "@/pages/AddressPage";
import { ValidatorsPage } from "@/pages/ValidatorsPage";
import { ValidatorDetailsPage } from "@/pages/ValidatorDetailsPage";
import { StakingPage } from "@/pages/StakingPage";
import { ContractsPage } from "@/pages/ContractsPage";
import { ContractDetailsPage } from "@/pages/ContractDetailsPage";
import { DevelopersPage } from "@/pages/DevelopersPage";
import { AnalyticsPage } from "@/pages/AnalyticsPage";
import { FaucetPage } from "@/pages/FaucetPage";
import { NetworkStatusPage } from "@/pages/NetworkStatusPage";
import { NotFoundPage } from "@/pages/NotFoundPage";

export const App: React.FC = () => {
  // Theme state
  const [theme, setTheme] = useState<ThemeMode>(() => {
    return (localStorage.getItem("sprx_theme") as ThemeMode) || "dark";
  });

  // Currency state
  const [currency, setCurrency] = useState<Currency>(() => {
    return (localStorage.getItem("sprx_currency") as Currency) || "USD";
  });

  // Network state
  const [network, setNetwork] = useState<Network>(() => {
    return (localStorage.getItem("sprx_network") as Network) || "mainnet";
  });

  // Navigation / Route state
  const [currentRoute, setCurrentRoute] = useState<string>(() => {
    return window.location.pathname || "/";
  });

  // Search Modal state
  const [searchModalOpen, setSearchModalOpen] = useState(false);

  // Blockchain Data state
  const [stats, setStats] = useState<NetworkStats | null>(null);
  const [blocks, setBlocks] = useState<IndexedBlock[]>([]);
  const [transactions, setTransactions] = useState<IndexedTx[]>([]);
  const [validators, setValidators] = useState<IndexedValidator[]>([]);
  const [contracts, setContracts] = useState<SmartContract[]>([]);

  // Detailed selected records
  const [currentBlock, setCurrentBlock] = useState<IndexedBlock | null>(null);
  const [currentBlockTxs, setCurrentBlockTxs] = useState<IndexedTx[]>([]);
  const [currentTx, setCurrentTx] = useState<IndexedTx | null>(null);
  const [currentAccount, setCurrentAccount] = useState<IndexedAccount | null>(null);
  const [currentAccountTxs, setCurrentAccountTxs] = useState<IndexedTx[]>([]);
  const [currentValidator, setCurrentValidator] = useState<IndexedValidator | null>(null);
  const [currentContract, setCurrentContract] = useState<SmartContract | null>(null);

  // Pagination states
  const [blocksPage, setBlocksPage] = useState(1);
  const [txsPage, setTxsPage] = useState(1);
  const pageSize = 15;

  // Real-time polling
  const [isPolling, setIsPolling] = useState(true);

  // Sync theme with HTML class
  useEffect(() => {
    localStorage.setItem("sprx_theme", theme);
    if (theme === "dark") {
      document.documentElement.classList.add("dark");
      document.documentElement.classList.remove("light");
    } else if (theme === "light") {
      document.documentElement.classList.remove("dark");
      document.documentElement.classList.add("light");
    } else {
      const prefersDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
      document.documentElement.classList.toggle("dark", prefersDark);
      document.documentElement.classList.toggle("light", !prefersDark);
    }
  }, [theme]);

  // Sync currency and network
  const handleCurrencyChange = (c: Currency) => {
    setCurrency(c);
    localStorage.setItem("sprx_currency", c);
  };

  const handleNetworkChange = (n: Network) => {
    setNetwork(n);
    apiService.setNetwork(n);
    localStorage.setItem("sprx_network", n);
    loadAllData();
  };

  const toggleTheme = () => {
    setTheme((prev) => (prev === "dark" ? "light" : "dark"));
  };

  // Load all initial telemetry data
  const loadAllData = useCallback(async () => {
    try {
      const [st, blkRes, txRes, valList, ctList] = await Promise.all([
        apiService.getStats(),
        apiService.getBlocks(pageSize, (blocksPage - 1) * pageSize),
        apiService.getTransactions(pageSize, (txsPage - 1) * pageSize),
        apiService.getValidators(),
        apiService.getContracts(),
      ]);

      setStats(st);
      setBlocks(blkRes.items);
      setTransactions(txRes.items);
      setValidators(valList);
      setContracts(ctList);
    } catch (err) {
      console.error("Failed to load blockchain data:", err);
    }
  }, [blocksPage, txsPage]);

  useEffect(() => {
    loadAllData();
  }, [loadAllData]);

  // Periodic polling for real-time blocks & txs
  useEffect(() => {
    if (!isPolling) return;
    const interval = setInterval(async () => {
      try {
        const [st, blkRes, txRes] = await Promise.all([
          apiService.getStats(),
          apiService.getBlocks(pageSize, 0),
          apiService.getTransactions(pageSize, 0),
        ]);
        setStats(st);
        setBlocks(blkRes.items);
        setTransactions(txRes.items);
      } catch {
        // Suppress polling error
      }
    }, 4000);

    return () => clearInterval(interval);
  }, [isPolling]);

  // Router navigation helper
  const navigateTo = (route: string) => {
    setCurrentRoute(route);
    window.scrollTo({ top: 0, behavior: "smooth" });
  };

  // Select Block
  const handleSelectBlock = async (heightOrHash: string | number) => {
    const block = await apiService.getBlock(heightOrHash);
    if (block) {
      setCurrentBlock(block);
      const txs = transactions.filter((t) => t.block_height === block.height);
      setCurrentBlockTxs(txs.length > 0 ? txs : transactions.slice(0, 3));
      navigateTo(`/block/${block.height}`);
    } else {
      navigateTo("/404");
    }
  };

  // Select Transaction
  const handleSelectTx = async (hash: string) => {
    const tx = await apiService.getTransaction(hash);
    if (tx) {
      setCurrentTx(tx);
      navigateTo(`/tx/${tx.tx_hash}`);
    } else {
      navigateTo("/404");
    }
  };

  // Select Address
  const handleSelectAddress = async (address: string) => {
    const [acc, accTxs] = await Promise.all([
      apiService.getAddress(address),
      apiService.getAddressTransactions(address, 20, 0),
    ]);
    if (acc) {
      setCurrentAccount(acc);
      setCurrentAccountTxs(accTxs.items);
      navigateTo(`/address/${acc.address}`);
    } else {
      navigateTo("/404");
    }
  };

  // Select Validator
  const handleSelectValidator = async (operatorAddress: string) => {
    const val = validators.find(
      (v) => v.operator_address.toLowerCase() === operatorAddress.toLowerCase()
    );
    if (val) {
      setCurrentValidator(val);
      navigateTo(`/validator/${val.operator_address}`);
    } else {
      navigateTo("/404");
    }
  };

  // Select Contract
  const handleSelectContract = async (contractAddress: string) => {
    const ct = await apiService.getContract(contractAddress);
    if (ct) {
      setCurrentContract(ct);
      navigateTo(`/contract/${ct.address}`);
    } else {
      navigateTo("/404");
    }
  };

  // Universal Search Resolver
  const handleUniversalSearch = async (query: string) => {
    const result = await apiService.search(query);
    if (!result) {
      navigateTo("/404");
      return;
    }

    if (result.type === "Block") {
      setCurrentBlock(result.data);
      const txs = transactions.filter((t) => t.block_height === result.data.height);
      setCurrentBlockTxs(txs.length > 0 ? txs : transactions.slice(0, 3));
      navigateTo(`/block/${result.data.height}`);
    } else if (result.type === "Transaction") {
      setCurrentTx(result.data);
      navigateTo(`/tx/${result.data.tx_hash}`);
    } else if (result.type === "Address") {
      setCurrentAccount(result.data);
      const accTxs = await apiService.getAddressTransactions(result.data.address, 20, 0);
      setCurrentAccountTxs(accTxs.items);
      navigateTo(`/address/${result.data.address}`);
    } else if (result.type === "Validator") {
      setCurrentValidator(result.data);
      navigateTo(`/validator/${result.data.operator_address}`);
    } else if (result.type === "Contract") {
      setCurrentContract(result.data);
      navigateTo(`/contract/${result.data.address}`);
    }
  };

  // Keyboard shortcut listener (Cmd+K / Ctrl+K)
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "k") {
        e.preventDefault();
        setSearchModalOpen((prev) => !prev);
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, []);

  return (
    <ErrorBoundary>
      <div className="min-h-screen flex flex-col bg-bg-primary text-text-primary">
        <AppHeader
          currentRoute={currentRoute}
          onNavigate={navigateTo}
          network={network}
          onNetworkChange={handleNetworkChange}
          currency={currency}
          onCurrencyChange={handleCurrencyChange}
          theme={theme}
          onToggleTheme={toggleTheme}
          onOpenSearch={() => setSearchModalOpen(true)}
          latestBlockHeight={stats?.latest_height}
        />

        {apiService.isUsingOfflineFallback() && (
          <div className="bg-amber-500/10 border-b border-amber-500/30 px-4 py-2 text-center text-xs sm:text-sm text-amber-400 font-medium flex items-center justify-center gap-2">
            <span className="inline-block w-2 h-2 rounded-full bg-amber-400 animate-pulse" />
            <span>
              Demo / Offline Data: Live blockchain backend is currently unreachable on {network}. Showing simulated genesis data.
            </span>
          </div>
        )}

        <main className="flex-1 max-w-7xl w-full mx-auto px-4 sm:px-6 lg:px-8 py-6 sm:py-8">
          {/* Router View Switching */}
          {currentRoute === "/" && stats && (
            <HomePage
              stats={stats}
              recentBlocks={blocks}
              recentTxs={transactions}
              currency={currency}
              isPolling={isPolling}
              onTogglePolling={() => setIsPolling(!isPolling)}
              onSelectBlock={handleSelectBlock}
              onSelectTx={handleSelectTx}
              onSelectAddress={handleSelectAddress}
              onNavigate={navigateTo}
              onSearch={handleUniversalSearch}
            />
          )}

          {currentRoute === "/blocks" && (
            <BlocksPage
              blocks={blocks}
              totalBlocks={stats?.latest_height || 100}
              currentPage={blocksPage}
              pageSize={pageSize}
              onPageChange={setBlocksPage}
              onSelectBlock={handleSelectBlock}
              onSelectAddress={handleSelectAddress}
            />
          )}

          {currentRoute.startsWith("/block/") && currentBlock && (
            <BlockDetailsPage
              block={currentBlock}
              txs={currentBlockTxs}
              onBack={() => navigateTo("/blocks")}
              onSelectBlock={handleSelectBlock}
              onSelectTx={handleSelectTx}
              onSelectAddress={handleSelectAddress}
            />
          )}

          {currentRoute === "/transactions" && (
            <TransactionsPage
              transactions={transactions}
              totalTxs={stats?.total_transactions || 500}
              currentPage={txsPage}
              pageSize={pageSize}
              onPageChange={setTxsPage}
              onSelectTx={handleSelectTx}
              onSelectBlock={handleSelectBlock}
              onSelectAddress={handleSelectAddress}
            />
          )}

          {currentRoute.startsWith("/tx/") && currentTx && (
            <TxDetailsPage
              tx={currentTx}
              currency={currency}
              onBack={() => navigateTo("/transactions")}
              onSelectBlock={handleSelectBlock}
              onSelectAddress={handleSelectAddress}
            />
          )}

          {currentRoute.startsWith("/address/") && currentAccount && (
            <AddressPage
              account={currentAccount}
              txs={currentAccountTxs}
              currency={currency}
              onBack={() => navigateTo("/")}
              onSelectTx={handleSelectTx}
              onSelectBlock={handleSelectBlock}
              onSelectAddress={handleSelectAddress}
            />
          )}

          {currentRoute === "/validators" && (
            <ValidatorsPage
              validators={validators}
              onSelectValidator={handleSelectValidator}
            />
          )}

          {currentRoute.startsWith("/validator/") && currentValidator && (
            <ValidatorDetailsPage
              validator={currentValidator}
              onBack={() => navigateTo("/validators")}
              onSelectAddress={handleSelectAddress}
            />
          )}

          {currentRoute === "/staking" && (
            <StakingPage
              validators={validators}
              currency={currency}
              onSelectValidator={handleSelectValidator}
            />
          )}

          {currentRoute === "/contracts" && (
            <ContractsPage
              contracts={contracts}
              onSelectContract={handleSelectContract}
              onSelectAddress={handleSelectAddress}
              onSelectBlock={handleSelectBlock}
            />
          )}

          {currentRoute.startsWith("/contract/") && currentContract && (
            <ContractDetailsPage
              contract={currentContract}
              onBack={() => navigateTo("/contracts")}
              onSelectAddress={handleSelectAddress}
              onSelectBlock={handleSelectBlock}
            />
          )}

          {currentRoute === "/developers" && (
            <DevelopersPage network={network} />
          )}

          {currentRoute === "/analytics" && (
            <AnalyticsPage />
          )}

          {currentRoute === "/faucet" && (
            <FaucetPage
              network={network}
            />
          )}

          {currentRoute === "/network" && stats && (
            <NetworkStatusPage
              stats={stats}
            />
          )}

          {currentRoute === "/404" && (
            <NotFoundPage
              onGoHome={() => navigateTo("/")}
              onOpenSearch={() => setSearchModalOpen(true)}
            />
          )}
        </main>

        <AppFooter
          network={network}
          latestBlock={stats?.latest_height}
          onNavigate={navigateTo}
        />

        {/* Global Search Modal Palette */}
        <SearchModal
          isOpen={searchModalOpen}
          onClose={() => setSearchModalOpen(false)}
          onSelectResult={handleUniversalSearch}
        />
      </div>
    </ErrorBoundary>
  );
};
export default App;
