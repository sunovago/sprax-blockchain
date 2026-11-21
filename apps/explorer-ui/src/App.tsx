import React, { useEffect, useState, useCallback } from "react";
import {
  Currency,
  IndexedAccount,
  IndexedBlock,
  IndexedTx,
  IndexedValidator,
  Network,
  NetworkStats,
  SmartContract,
  ThemeMode,
} from "@/types";
import { apiService } from "@/services/api";
import { AppHeader } from "@/components/AppHeader";
import { AppFooter } from "@/components/AppFooter";
import { SearchModal } from "@/components/SearchModal";
import { GetStartedModal } from "@/components/GetStartedModal";
import { ConnectWalletModal } from "@/components/ConnectWalletModal";
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
import { AnalyticsPage } from "@/pages/AnalyticsPage";
import { FaucetPage } from "@/pages/FaucetPage";
import { NetworkStatusPage } from "@/pages/NetworkStatusPage";
import { NotFoundPage } from "@/pages/NotFoundPage";

// New Ecosystem Hub Pages
import { LearnPage } from "@/pages/LearnPage";
import { DevelopersPortalPage } from "@/pages/DevelopersPortalPage";
import { EcosystemPage } from "@/pages/EcosystemPage";
import { GovernancePage } from "@/pages/GovernancePage";
import { ResearchPage } from "@/pages/ResearchPage";
import { WhitepaperPage } from "@/pages/WhitepaperPage";
import { AboutPage } from "@/pages/AboutPage";
import { BrandKitPage } from "@/pages/BrandKitPage";

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

  // Modal states
  const [searchModalOpen, setSearchModalOpen] = useState(false);
  const [getStartedModalOpen, setGetStartedModalOpen] = useState(false);
  const [connectWalletModalOpen, setConnectWalletModalOpen] = useState(false);
  const [connectedWallet, setConnectedWallet] = useState<string | null>(() => {
    return localStorage.getItem("sprx_connected_wallet") || null;
  });

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
  const [isPolling] = useState(true);

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
    setBlocksPage(1);
    setTxsPage(1);
  };

  const toggleTheme = () => {
    setTheme((prev) => (prev === "dark" ? "light" : "dark"));
  };

  // Load all initial telemetry data
  const loadAllData = useCallback(async () => {
    apiService.setNetwork(network);
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
  }, [blocksPage, txsPage, network]);

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
          apiService.getBlocks(pageSize, (blocksPage - 1) * pageSize),
          apiService.getTransactions(pageSize, (txsPage - 1) * pageSize),
        ]);
        setStats(st);
        setBlocks(blkRes.items);
        setTransactions(txRes.items);
      } catch {
        // Suppress polling error
      }
    }, 4000);

    return () => clearInterval(interval);
  }, [isPolling, blocksPage, txsPage, network]);

  // Router navigation helper
  const navigateTo = (route: string) => {
    if (window.location.pathname !== route) window.history.pushState(null, "", route);
    setCurrentRoute(route);
    window.scrollTo({ top: 0, behavior: "smooth" });
  };

  useEffect(() => {
    const restoreRoute = () => setCurrentRoute(window.location.pathname || "/");
    window.addEventListener("popstate", restoreRoute);
    return () => window.removeEventListener("popstate", restoreRoute);
  }, []);

  const [routeLoading, setRouteLoading] = useState(false);
  const [routeMissing, setRouteMissing] = useState(false);

  // Resolve detail URLs on refresh and browser Back/Forward as well as clicks.
  useEffect(() => {
    let cancelled = false;
    const loadRoute = async () => {
      const [, kind, id] = currentRoute.split("/");
      setRouteMissing(false);
      if (!id || !["block", "tx", "address", "contract", "validator"].includes(kind)) { setRouteLoading(false); return; }
      setRouteLoading(true);
      if (kind === "block") {
        const block = await apiService.getBlock(id);
        if (!cancelled) { setCurrentBlock(block); setRouteMissing(!block); }
      } else if (kind === "tx") {
        const tx = await apiService.getTransaction(id);
        if (!cancelled) { setCurrentTx(tx); setRouteMissing(!tx); }
      } else if (kind === "address") {
        const [account, txs] = await Promise.all([apiService.getAddress(id), apiService.getAddressTransactions(id, 20, 0)]);
        if (!cancelled) { setCurrentAccount(account); setCurrentAccountTxs(txs.items); setRouteMissing(!account); }
      } else if (kind === "contract") {
        const contract = await apiService.getContract(id);
        if (!cancelled) { setCurrentContract(contract); setRouteMissing(!contract); }
      } else if (kind === "validator") {
        const list = await apiService.getValidators();
        if (!cancelled) { const validator = list.find(v => v.operator_address === id) ?? null; setCurrentValidator(validator); setRouteMissing(!validator); }
      }
    };
    void loadRoute().catch(() => { if (!cancelled) setRouteMissing(true); }).finally(() => { if (!cancelled) setRouteLoading(false); });
    return () => { cancelled = true; };
  }, [currentRoute, network]);

  // Select Block
  const handleSelectBlock = async (heightOrHash: string | number) => {
    const block = await apiService.getBlock(heightOrHash);
    if (block) {
      setCurrentBlock(block);
      const txs = transactions.filter((t) => t.block_height === block.height);
      setCurrentBlockTxs(txs);
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
      setCurrentBlockTxs(txs);
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
          onOpenGetStarted={() => setGetStartedModalOpen(true)}
          onOpenConnectWallet={() => setConnectWalletModalOpen(true)}
          connectedWallet={connectedWallet}
          latestBlockHeight={stats?.latest_height}
        />

        {apiService.isUsingOfflineFallback() && (
          <div className="bg-amber-500/10 border-b border-amber-500/30 px-4 py-2 text-center text-xs sm:text-sm text-amber-400 font-medium flex items-center justify-center gap-2">
            <span className="inline-block w-2 h-2 rounded-full bg-amber-400 animate-pulse" />
            <span>
              Showing sample {network.toUpperCase()} explorer data. Live chain connection is unavailable.
            </span>
          </div>
        )}

        <main className="app-container min-w-0 flex-1 py-6 sm:py-8">
          {routeLoading && <p role="status" className="py-8 text-text-secondary">Loading record...</p>}
          {routeMissing && <NotFoundPage onGoHome={() => navigateTo("/")} onOpenSearch={() => setSearchModalOpen(true)} />}
          {/* 1. Core Homepage */}
          {currentRoute === "/" && stats && (
            <HomePage
              stats={stats}
              recentBlocks={blocks}
              recentTxs={transactions}
              onSelectBlock={handleSelectBlock}
              onSelectTx={handleSelectTx}
              onNavigate={navigateTo}
              onSearch={handleUniversalSearch}
            />
          )}

          {/* 2. Explorer Hub */}
          {currentRoute === "/explorer" && stats && (
            <HomePage
              stats={stats}
              recentBlocks={blocks}
              recentTxs={transactions}
              onSelectBlock={handleSelectBlock}
              onSelectTx={handleSelectTx}
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

          {currentRoute.startsWith("/block/") && !routeLoading && !routeMissing && currentBlock && (
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

          {currentRoute.startsWith("/tx/") && !routeLoading && !routeMissing && currentTx && (
            <TxDetailsPage
              tx={currentTx}
              currency={currency}
              onBack={() => navigateTo("/transactions")}
              onSelectBlock={handleSelectBlock}
              onSelectAddress={handleSelectAddress}
            />
          )}

          {currentRoute.startsWith("/address/") && !routeLoading && !routeMissing && currentAccount && (
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

          {/* 3. Learn Hub */}
          {currentRoute.startsWith("/learn") && (
            <LearnPage
              key={currentRoute}
              initialSubpage={currentRoute}
              onNavigate={navigateTo}
            />
          )}

          {/* 4. Developer Command Center */}
          {currentRoute.startsWith("/developers") && (
            <DevelopersPortalPage
              key={currentRoute}
              network={network}
              initialTab={currentRoute.replace("/developers", "")}
              onNavigate={navigateTo}
            />
          )}

          {/* 5. Network & Validators */}
          {currentRoute === "/network" && stats && (
            <NetworkStatusPage stats={stats} />
          )}

          {currentRoute === "/validators" && (
            <ValidatorsPage
              validators={validators}
              onSelectValidator={handleSelectValidator}
            />
          )}

          {currentRoute.startsWith("/validator/") && !routeLoading && !routeMissing && currentValidator && (
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

          {/* 6. Smart Contracts */}
          {currentRoute === "/contracts" && (
            <ContractsPage
              contracts={contracts}
              onSelectContract={handleSelectContract}
              onSelectAddress={handleSelectAddress}
              onSelectBlock={handleSelectBlock}
              onVerifyContract={(newContract) => {
                apiService.saveContract(newContract);
                setContracts((prev) => [
                  newContract,
                  ...prev.filter((c) => c.address.toLowerCase() !== newContract.address.toLowerCase()),
                ]);
                setCurrentContract(newContract);
                navigateTo(`/contract/${newContract.address}`);
              }}
            />
          )}

          {currentRoute.startsWith("/contract/") && !routeLoading && !routeMissing && currentContract && (
            <ContractDetailsPage
              contract={currentContract}
              onBack={() => navigateTo("/contracts")}
              onSelectAddress={handleSelectAddress}
              onSelectBlock={handleSelectBlock}
              onContractUpdated={(updated) => {
                apiService.saveContract(updated);
                setContracts((prev) =>
                  prev.map((c) => (c.address.toLowerCase() === updated.address.toLowerCase() ? updated : c))
                );
                setCurrentContract(updated);
              }}
            />
          )}

          {/* 7. Ecosystem, Discover & Markets */}
          {(currentRoute.startsWith("/ecosystem") ||
            currentRoute === "/discover" ||
            currentRoute === "/markets") && (
            <EcosystemPage
              currency={currency}
              network={network}
              initialTab={currentRoute}
              onNavigate={navigateTo}
            />
          )}

          {/* 8. Governance & SIPs */}
          {(currentRoute.startsWith("/governance") ||
            currentRoute.startsWith("/sips") ||
            currentRoute.startsWith("/community/grants") ||
            currentRoute.startsWith("/security/bug-bounty")) && (
            <GovernancePage
              key={currentRoute}
              initialTab={currentRoute}
              onNavigate={navigateTo}
            />
          )}

          {/* 9. Research Hub */}
          {currentRoute.startsWith("/research") && (
            <ResearchPage onNavigate={navigateTo} />
          )}

          {/* 10. Whitepaper, About, Brand */}
          {currentRoute === "/whitepaper" && (
            <WhitepaperPage onBack={() => navigateTo("/")} />
          )}

          {currentRoute === "/about" && (
            <AboutPage />
          )}

          {currentRoute === "/brand" && (
            <BrandKitPage />
          )}

          {currentRoute === "/analytics" && (
            <AnalyticsPage />
          )}

          {currentRoute === "/faucet" && (
            <FaucetPage network={network} />
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

        {/* Global Search Command Palette */}
        <SearchModal
          isOpen={searchModalOpen}
          onClose={() => setSearchModalOpen(false)}
          onSelectResult={handleUniversalSearch}
        />

        {/* Onboarding & Persona Chooser Modal */}
        <GetStartedModal
          isOpen={getStartedModalOpen}
          onClose={() => setGetStartedModalOpen(false)}
          onNavigate={navigateTo}
        />

        {/* Non-custodial Wallet Connect Modal */}
        <ConnectWalletModal
          isOpen={connectWalletModalOpen}
          onClose={() => setConnectWalletModalOpen(false)}
          onConnected={(addr) => setConnectedWallet(addr)}
        />
      </div>
    </ErrorBoundary>
  );
};
export default App;
