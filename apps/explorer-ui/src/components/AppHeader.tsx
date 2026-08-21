import React, { useState, useRef } from "react";
import {
  Activity,
  Box,
  Code2,
  Cpu,
  Coins,
  ChevronDown,
  Layers,
  Menu,
  Moon,
  Search,
  ShieldCheck,
  Sun,
  X,
  FileCode2,
  Zap,
  Globe,
  Compass,
  BookOpen,
  Vote,
  FileText,
  Terminal,
  Sparkles,
  TrendingUp,
  Award,
  Wallet,
} from "lucide-react";
import { Currency, Network, ThemeMode } from "@/types";

interface AppHeaderProps {
  currentRoute: string;
  onNavigate: (route: string) => void;
  network: Network;
  onNetworkChange: (network: Network) => void;
  currency: Currency;
  onCurrencyChange: (currency: Currency) => void;
  theme: ThemeMode;
  onToggleTheme: () => void;
  onOpenSearch: () => void;
  onOpenGetStarted?: () => void;
  onOpenConnectWallet?: () => void;
  connectedWallet?: string | null;
  latestBlockHeight?: number;
}

export const AppHeader: React.FC<AppHeaderProps> = ({
  currentRoute,
  onNavigate,
  network,
  onNetworkChange,
  currency,
  onCurrencyChange,
  theme,
  onToggleTheme,
  onOpenSearch,
  onOpenGetStarted,
  onOpenConnectWallet,
  connectedWallet,
}) => {
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false);
  const [activeDropdown, setActiveDropdown] = useState<string | null>(null);
  const dropdownTimeoutRef = useRef<NodeJS.Timeout | null>(null);

  const isTestnet = network !== "mainnet";

  const handleNav = (route: string) => {
    onNavigate(route);
    setMobileMenuOpen(false);
    setActiveDropdown(null);
  };

  const handleMouseEnter = (menuKey: string) => {
    if (dropdownTimeoutRef.current) clearTimeout(dropdownTimeoutRef.current);
    setActiveDropdown(menuKey);
  };

  const handleMouseLeave = () => {
    dropdownTimeoutRef.current = setTimeout(() => {
      setActiveDropdown(null);
    }, 150);
  };

  return (
    <header className="sticky top-0 z-40 w-full border-b border-border-subtle bg-bg-surface/90 backdrop-blur-md">
      {/* Testnet Alert Strip */}
      {isTestnet && (
        <div className="bg-amber-500/10 border-b border-amber-500/20 px-4 py-1 text-center text-xs font-semibold text-amber-400 flex items-center justify-center gap-2">
          <span className="w-2 h-2 rounded-full bg-amber-400 animate-pulse" />
          <span>CONNECTED TO {network.toUpperCase()} ENVIRONMENT — TESTNET TOKENS HAVE NO MONETARY VALUE</span>
        </div>
      )}

      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div className="flex items-center justify-between h-16 gap-3">
          {/* Logo & Wordmark */}
          <div
            className="flex items-center gap-3 cursor-pointer select-none shrink-0"
            onClick={() => handleNav("/")}
          >
            <div className="relative flex items-center justify-center w-9 h-9 rounded-xl bg-gradient-to-br from-cyan-400 via-sky-500 to-indigo-600 shadow-md border border-cyan-400/30">
              <span className="font-black text-white text-base tracking-tighter font-sans">
                X
              </span>
              <span className="absolute -top-1 -right-1 w-2.5 h-2.5 rounded-full bg-emerald-400 ring-2 ring-bg-surface" />
            </div>

            <div>
              <div className="flex items-center gap-1.5">
                <span className="text-lg font-extrabold tracking-tight text-text-primary">
                  SPRX
                </span>
                <span className="text-[10px] font-bold px-1.5 py-0.2 rounded bg-cyan-500/10 text-cyan-400 border border-cyan-500/20 uppercase tracking-wide">
                  Protocol
                </span>
              </div>
              <p className="text-[10px] text-text-muted hidden sm:block tracking-wide">
                Scalable Protocol for Real-world X
              </p>
            </div>
          </div>

          {/* Desktop Primary Navigation Mega-Menus */}
          <nav className="hidden lg:flex items-center gap-1 text-xs sm:text-sm font-medium text-text-secondary">
            {/* 1. Explore */}
            <div
              className="relative"
              onMouseEnter={() => handleMouseEnter("explore")}
              onMouseLeave={handleMouseLeave}
            >
              <button
                className={`flex items-center gap-1 px-3 py-2 rounded-lg transition-colors ${
                  currentRoute.startsWith("/explorer") || currentRoute.startsWith("/block") || currentRoute.startsWith("/tx")
                    ? "text-cyan-400 bg-cyan-500/10 font-bold"
                    : "hover:text-text-primary hover:bg-bg-surface-elevated"
                }`}
              >
                <span>Explore</span>
                <ChevronDown className="w-3.5 h-3.5 opacity-70" />
              </button>

              {activeDropdown === "explore" && (
                <div className="absolute left-0 mt-1 w-72 rounded-2xl border border-border-prominent bg-bg-surface p-2 shadow-2xl animate-fadeIn space-y-1">
                  <div
                    onClick={() => handleNav("/explorer")}
                    className="p-2.5 rounded-xl hover:bg-bg-surface-elevated cursor-pointer transition-colors flex items-start gap-3"
                  >
                    <Box className="w-4 h-4 text-sky-400 mt-0.5" />
                    <div>
                      <div className="text-xs font-bold text-text-primary">Blockchain Explorer</div>
                      <div className="text-[11px] text-text-muted">Live block stream & search</div>
                    </div>
                  </div>
                  <div
                    onClick={() => handleNav("/blocks")}
                    className="p-2.5 rounded-xl hover:bg-bg-surface-elevated cursor-pointer transition-colors flex items-start gap-3"
                  >
                    <Layers className="w-4 h-4 text-indigo-400 mt-0.5" />
                    <div>
                      <div className="text-xs font-bold text-text-primary">Blocks & Finality</div>
                      <div className="text-[11px] text-text-muted">CometBFT verified blocks</div>
                    </div>
                  </div>
                  <div
                    onClick={() => handleNav("/transactions")}
                    className="p-2.5 rounded-xl hover:bg-bg-surface-elevated cursor-pointer transition-colors flex items-start gap-3"
                  >
                    <Activity className="w-4 h-4 text-teal-400 mt-0.5" />
                    <div>
                      <div className="text-xs font-bold text-text-primary">Transactions</div>
                      <div className="text-[11px] text-text-muted">Atomic state transfers</div>
                    </div>
                  </div>
                  <div
                    onClick={() => handleNav("/contracts")}
                    className="p-2.5 rounded-xl hover:bg-bg-surface-elevated cursor-pointer transition-colors flex items-start gap-3"
                  >
                    <FileCode2 className="w-4 h-4 text-purple-400 mt-0.5" />
                    <div>
                      <div className="text-xs font-bold text-text-primary">Smart Contracts</div>
                      <div className="text-[11px] text-text-muted">Verified WASM bytecode</div>
                    </div>
                  </div>
                  <div
                    onClick={() => handleNav("/analytics")}
                    className="p-2.5 rounded-xl hover:bg-bg-surface-elevated cursor-pointer transition-colors flex items-start gap-3"
                  >
                    <TrendingUp className="w-4 h-4 text-amber-400 mt-0.5" />
                    <div>
                      <div className="text-xs font-bold text-text-primary">Network Analytics</div>
                      <div className="text-[11px] text-text-muted">TPS & Gas throughput charts</div>
                    </div>
                  </div>
                </div>
              )}
            </div>

            {/* 2. Learn */}
            <div
              className="relative"
              onMouseEnter={() => handleMouseEnter("learn")}
              onMouseLeave={handleMouseLeave}
            >
              <button
                className={`flex items-center gap-1 px-3 py-2 rounded-lg transition-colors ${
                  currentRoute.startsWith("/learn")
                    ? "text-cyan-400 bg-cyan-500/10 font-bold"
                    : "hover:text-text-primary hover:bg-bg-surface-elevated"
                }`}
              >
                <span>Learn</span>
                <ChevronDown className="w-3.5 h-3.5 opacity-70" />
              </button>

              {activeDropdown === "learn" && (
                <div className="absolute left-0 mt-1 w-80 rounded-2xl border border-border-prominent bg-bg-surface p-2 shadow-2xl animate-fadeIn space-y-1">
                  <div
                    onClick={() => handleNav("/learn")}
                    className="p-2.5 rounded-xl hover:bg-bg-surface-elevated cursor-pointer transition-colors flex items-start gap-3"
                  >
                    <BookOpen className="w-4 h-4 text-sky-400 mt-0.5" />
                    <div>
                      <div className="text-xs font-bold text-text-primary">Learn Hub</div>
                      <div className="text-[11px] text-text-muted">Master guide to SPRX architecture</div>
                    </div>
                  </div>
                  <div
                    onClick={() => handleNav("/learn/what-is-sprx")}
                    className="p-2.5 rounded-xl hover:bg-bg-surface-elevated cursor-pointer transition-colors flex items-start gap-3"
                  >
                    <Sparkles className="w-4 h-4 text-cyan-400 mt-0.5" />
                    <div>
                      <div className="text-xs font-bold text-text-primary">What is SPRX?</div>
                      <div className="text-[11px] text-text-muted">Scalable Protocol for Real-world X</div>
                    </div>
                  </div>
                  <div
                    onClick={() => handleNav("/learn/real-world-x")}
                    className="p-2.5 rounded-xl hover:bg-bg-surface-elevated cursor-pointer transition-colors flex items-start gap-3"
                  >
                    <Zap className="w-4 h-4 text-amber-400 mt-0.5" />
                    <div>
                      <div className="text-xs font-bold text-text-primary">Real-World X Rails</div>
                      <div className="text-[11px] text-text-muted">Payments, RWA, Identity & DePIN</div>
                    </div>
                  </div>
                  <div
                    onClick={() => handleNav("/learn/tokenomics")}
                    className="p-2.5 rounded-xl hover:bg-bg-surface-elevated cursor-pointer transition-colors flex items-start gap-3"
                  >
                    <Coins className="w-4 h-4 text-indigo-400 mt-0.5" />
                    <div>
                      <div className="text-xs font-bold text-text-primary">Tokenomics & Precision</div>
                      <div className="text-[11px] text-text-muted">1B Supply, 18 decimals, Base burn</div>
                    </div>
                  </div>
                  <div
                    onClick={() => handleNav("/learn/wallets")}
                    className="p-2.5 rounded-xl hover:bg-bg-surface-elevated cursor-pointer transition-colors flex items-start gap-3"
                  >
                    <Wallet className="w-4 h-4 text-teal-400 mt-0.5" />
                    <div>
                      <div className="text-xs font-bold text-text-primary">Wallets & Self-Custody</div>
                      <div className="text-[11px] text-text-muted">Keys, seed phrases & signing</div>
                    </div>
                  </div>
                </div>
              )}
            </div>

            {/* 3. Developers */}
            <div
              className="relative"
              onMouseEnter={() => handleMouseEnter("devs")}
              onMouseLeave={handleMouseLeave}
            >
              <button
                className={`flex items-center gap-1 px-3 py-2 rounded-lg transition-colors ${
                  currentRoute.startsWith("/developers")
                    ? "text-cyan-400 bg-cyan-500/10 font-bold"
                    : "hover:text-text-primary hover:bg-bg-surface-elevated"
                }`}
              >
                <span>Developers</span>
                <ChevronDown className="w-3.5 h-3.5 opacity-70" />
              </button>

              {activeDropdown === "devs" && (
                <div className="absolute left-0 mt-1 w-80 rounded-2xl border border-border-prominent bg-bg-surface p-2 shadow-2xl animate-fadeIn space-y-1">
                  <div
                    onClick={() => handleNav("/developers")}
                    className="p-2.5 rounded-xl hover:bg-bg-surface-elevated cursor-pointer transition-colors flex items-start gap-3"
                  >
                    <Code2 className="w-4 h-4 text-cyan-400 mt-0.5" />
                    <div>
                      <div className="text-xs font-bold text-text-primary">Developer Command Center</div>
                      <div className="text-[11px] text-text-muted">Quickstarts, SDKs & APIs</div>
                    </div>
                  </div>
                  <div
                    onClick={() => handleNav("/developers/rpc")}
                    className="p-2.5 rounded-xl hover:bg-bg-surface-elevated cursor-pointer transition-colors flex items-start gap-3"
                  >
                    <Terminal className="w-4 h-4 text-emerald-400 mt-0.5" />
                    <div>
                      <div className="text-xs font-bold text-text-primary">JSON-RPC & REST Playground</div>
                      <div className="text-[11px] text-text-muted">Live query runner & schemas</div>
                    </div>
                  </div>
                  <div
                    onClick={() => handleNav("/developers/smart-contracts")}
                    className="p-2.5 rounded-xl hover:bg-bg-surface-elevated cursor-pointer transition-colors flex items-start gap-3"
                  >
                    <Cpu className="w-4 h-4 text-purple-400 mt-0.5" />
                    <div>
                      <div className="text-xs font-bold text-text-primary">CosmWasm Smart Contracts</div>
                      <div className="text-[11px] text-text-muted">Rust WASM contract templates</div>
                    </div>
                  </div>
                  <div
                    onClick={() => handleNav("/developers/nodes")}
                    className="p-2.5 rounded-xl hover:bg-bg-surface-elevated cursor-pointer transition-colors flex items-start gap-3"
                  >
                    <Globe className="w-4 h-4 text-amber-400 mt-0.5" />
                    <div>
                      <div className="text-xs font-bold text-text-primary">Run a Node & Sentry</div>
                      <div className="text-[11px] text-text-muted">CLI setup & sync guide</div>
                    </div>
                  </div>
                  <div
                    onClick={() => handleNav("/faucet")}
                    className="p-2.5 rounded-xl hover:bg-bg-surface-elevated cursor-pointer transition-colors flex items-start gap-3"
                  >
                    <Coins className="w-4 h-4 text-sky-400 mt-0.5" />
                    <div>
                      <div className="text-xs font-bold text-text-primary">Testnet Faucet</div>
                      <div className="text-[11px] text-text-muted">Get free tSPRX tokens for testing</div>
                    </div>
                  </div>
                </div>
              )}
            </div>

            {/* 4. Network */}
            <div
              className="relative"
              onMouseEnter={() => handleMouseEnter("network")}
              onMouseLeave={handleMouseLeave}
            >
              <button
                className={`flex items-center gap-1 px-3 py-2 rounded-lg transition-colors ${
                  currentRoute.startsWith("/network") || currentRoute.startsWith("/validator") || currentRoute === "/staking"
                    ? "text-cyan-400 bg-cyan-500/10 font-bold"
                    : "hover:text-text-primary hover:bg-bg-surface-elevated"
                }`}
              >
                <span>Network</span>
                <ChevronDown className="w-3.5 h-3.5 opacity-70" />
              </button>

              {activeDropdown === "network" && (
                <div className="absolute left-0 mt-1 w-72 rounded-2xl border border-border-prominent bg-bg-surface p-2 shadow-2xl animate-fadeIn space-y-1">
                  <div
                    onClick={() => handleNav("/network")}
                    className="p-2.5 rounded-xl hover:bg-bg-surface-elevated cursor-pointer transition-colors flex items-start gap-3"
                  >
                    <Activity className="w-4 h-4 text-emerald-400 mt-0.5" />
                    <div>
                      <div className="text-xs font-bold text-text-primary">Network Status</div>
                      <div className="text-[11px] text-text-muted">CometBFT health & peers</div>
                    </div>
                  </div>
                  <div
                    onClick={() => handleNav("/validators")}
                    className="p-2.5 rounded-xl hover:bg-bg-surface-elevated cursor-pointer transition-colors flex items-start gap-3"
                  >
                    <ShieldCheck className="w-4 h-4 text-teal-400 mt-0.5" />
                    <div>
                      <div className="text-xs font-bold text-text-primary">Validator Set (Top 100)</div>
                      <div className="text-[11px] text-text-muted">Voting power & commission</div>
                    </div>
                  </div>
                  <div
                    onClick={() => handleNav("/staking")}
                    className="p-2.5 rounded-xl hover:bg-bg-surface-elevated cursor-pointer transition-colors flex items-start gap-3"
                  >
                    <Coins className="w-4 h-4 text-indigo-400 mt-0.5" />
                    <div>
                      <div className="text-xs font-bold text-text-primary">Staking & Yield</div>
                      <div className="text-[11px] text-text-muted">Delegate SPRX & earn rewards</div>
                    </div>
                  </div>
                </div>
              )}
            </div>

            {/* 5. Ecosystem */}
            <div
              className="relative"
              onMouseEnter={() => handleMouseEnter("ecosystem")}
              onMouseLeave={handleMouseLeave}
            >
              <button
                className={`flex items-center gap-1 px-3 py-2 rounded-lg transition-colors ${
                  currentRoute.startsWith("/ecosystem") || currentRoute === "/discover" || currentRoute === "/markets"
                    ? "text-cyan-400 bg-cyan-500/10 font-bold"
                    : "hover:text-text-primary hover:bg-bg-surface-elevated"
                }`}
              >
                <span>Ecosystem</span>
                <ChevronDown className="w-3.5 h-3.5 opacity-70" />
              </button>

              {activeDropdown === "ecosystem" && (
                <div className="absolute left-0 mt-1 w-72 rounded-2xl border border-border-prominent bg-bg-surface p-2 shadow-2xl animate-fadeIn space-y-1">
                  <div
                    onClick={() => handleNav("/ecosystem")}
                    className="p-2.5 rounded-xl hover:bg-bg-surface-elevated cursor-pointer transition-colors flex items-start gap-3"
                  >
                    <Compass className="w-4 h-4 text-sky-400 mt-0.5" />
                    <div>
                      <div className="text-xs font-bold text-text-primary">dApp Directory</div>
                      <div className="text-[11px] text-text-muted">DeFi, RWA, Wallets & Infra</div>
                    </div>
                  </div>
                  <div
                    onClick={() => handleNav("/discover")}
                    className="p-2.5 rounded-xl hover:bg-bg-surface-elevated cursor-pointer transition-colors flex items-start gap-3"
                  >
                    <Sparkles className="w-4 h-4 text-purple-400 mt-0.5" />
                    <div>
                      <div className="text-xs font-bold text-text-primary">Discover Showcase</div>
                      <div className="text-[11px] text-text-muted">Trending protocols & utilities</div>
                    </div>
                  </div>
                  <div
                    onClick={() => handleNav("/markets")}
                    className="p-2.5 rounded-xl hover:bg-bg-surface-elevated cursor-pointer transition-colors flex items-start gap-3"
                  >
                    <TrendingUp className="w-4 h-4 text-emerald-400 mt-0.5" />
                    <div>
                      <div className="text-xs font-bold text-text-primary">Live Markets</div>
                      <div className="text-[11px] text-text-muted">SPRX asset price & tickers</div>
                    </div>
                  </div>
                </div>
              )}
            </div>

            {/* 6. Governance & SIPs */}
            <div
              className="relative"
              onMouseEnter={() => handleMouseEnter("gov")}
              onMouseLeave={handleMouseLeave}
            >
              <button
                className={`flex items-center gap-1 px-3 py-2 rounded-lg transition-colors ${
                  currentRoute.startsWith("/governance") || currentRoute.startsWith("/sips")
                    ? "text-cyan-400 bg-cyan-500/10 font-bold"
                    : "hover:text-text-primary hover:bg-bg-surface-elevated"
                }`}
              >
                <span>Governance</span>
                <ChevronDown className="w-3.5 h-3.5 opacity-70" />
              </button>

              {activeDropdown === "gov" && (
                <div className="absolute left-0 mt-1 w-72 rounded-2xl border border-border-prominent bg-bg-surface p-2 shadow-2xl animate-fadeIn space-y-1">
                  <div
                    onClick={() => handleNav("/governance")}
                    className="p-2.5 rounded-xl hover:bg-bg-surface-elevated cursor-pointer transition-colors flex items-start gap-3"
                  >
                    <Vote className="w-4 h-4 text-amber-400 mt-0.5" />
                    <div>
                      <div className="text-xs font-bold text-text-primary">Voting Portal</div>
                      <div className="text-[11px] text-text-muted">On-chain proposal voting</div>
                    </div>
                  </div>
                  <div
                    onClick={() => handleNav("/sips")}
                    className="p-2.5 rounded-xl hover:bg-bg-surface-elevated cursor-pointer transition-colors flex items-start gap-3"
                  >
                    <FileText className="w-4 h-4 text-indigo-400 mt-0.5" />
                    <div>
                      <div className="text-xs font-bold text-text-primary">SIP Proposals</div>
                      <div className="text-[11px] text-text-muted">SPRX Improvement Proposals</div>
                    </div>
                  </div>
                  <div
                    onClick={() => handleNav("/community/grants")}
                    className="p-2.5 rounded-xl hover:bg-bg-surface-elevated cursor-pointer transition-colors flex items-start gap-3"
                  >
                    <Award className="w-4 h-4 text-emerald-400 mt-0.5" />
                    <div>
                      <div className="text-xs font-bold text-text-primary">Ecosystem Grants</div>
                      <div className="text-[11px] text-text-muted">Funding for open source</div>
                    </div>
                  </div>
                </div>
              )}
            </div>

            {/* 7. Research */}
            <div
              onClick={() => handleNav("/research")}
              className={`px-3 py-2 rounded-lg cursor-pointer transition-colors ${
                currentRoute === "/research"
                  ? "text-cyan-400 bg-cyan-500/10 font-bold"
                  : "hover:text-text-primary hover:bg-bg-surface-elevated"
              }`}
            >
              Research
            </div>
          </nav>

          {/* Right Utility Controls */}
          <div className="flex items-center gap-2 sm:gap-2.5 shrink-0">
            {/* Search Trigger (⌘K) */}
            <button
              onClick={onOpenSearch}
              className="flex items-center gap-2 px-2.5 sm:px-3 py-1.5 rounded-xl border border-border-subtle bg-bg-surface hover:bg-bg-surface-elevated hover:border-border-prominent text-text-secondary text-xs transition-all"
              title="Search Blockchain (Cmd+K / Ctrl+K)"
            >
              <Search className="w-3.5 h-3.5 text-text-muted" />
              <span className="hidden sm:inline">Search</span>
              <kbd className="hidden sm:inline-block px-1.5 py-0.5 text-[10px] font-mono bg-bg-surface-elevated border border-border-subtle rounded text-text-muted">
                ⌘K
              </kbd>
            </button>

            {/* Network Selector */}
            <div className="relative">
              <select
                value={network}
                onChange={(e) => onNetworkChange(e.target.value as Network)}
                className="appearance-none bg-bg-surface border border-border-subtle text-text-primary text-xs font-semibold rounded-xl pl-2.5 pr-7 py-1.5 focus:outline-none focus:border-cyan-400 cursor-pointer"
              >
                <option value="mainnet">Mainnet</option>
                <option value="testnet">Testnet</option>
                <option value="local">Local Devnet</option>
              </select>
              <ChevronDown className="w-3 h-3 text-text-muted absolute right-2 top-1/2 -translate-y-1/2 pointer-events-none" />
            </div>

            {/* Currency Selector */}
            <div className="relative hidden sm:block">
              <select
                value={currency}
                onChange={(e) => onCurrencyChange(e.target.value as Currency)}
                className="appearance-none bg-bg-surface border border-border-subtle text-text-primary text-xs font-semibold rounded-xl pl-2 pr-6 py-1.5 focus:outline-none focus:border-cyan-400 cursor-pointer font-mono-num"
              >
                <option value="USD">$ USD</option>
                <option value="INR">₹ INR</option>
                <option value="EUR">€ EUR</option>
                <option value="GBP">£ GBP</option>
                <option value="JPY">¥ JPY</option>
              </select>
              <ChevronDown className="w-3 h-3 text-text-muted absolute right-2 top-1/2 -translate-y-1/2 pointer-events-none" />
            </div>

            {/* Theme Toggle */}
            <button
              onClick={onToggleTheme}
              className="p-2 rounded-xl border border-border-subtle bg-bg-surface hover:bg-bg-surface-elevated text-text-secondary hover:text-text-primary transition-colors"
              title="Toggle Dark/Light Mode"
            >
              {theme === "dark" ? <Sun className="w-4 h-4" /> : <Moon className="w-4 h-4" />}
            </button>

            {/* Primary Action Button */}
            {connectedWallet ? (
              <button
                onClick={onOpenConnectWallet}
                className="hidden sm:flex items-center gap-1.5 px-3 py-1.5 rounded-xl border border-emerald-500/40 bg-emerald-500/10 text-emerald-400 text-xs font-bold font-mono-num hover:bg-emerald-500/20 transition-all"
              >
                <span className="w-1.5 h-1.5 rounded-full bg-emerald-400" />
                <span>{connectedWallet.slice(0, 7)}...{connectedWallet.slice(-4)}</span>
              </button>
            ) : (
              <button
                onClick={onOpenGetStarted || onOpenConnectWallet}
                className="px-3.5 py-1.5 rounded-xl bg-gradient-to-r from-sky-500 to-indigo-600 hover:from-sky-400 hover:to-indigo-500 text-white text-xs font-bold shadow-md transition-all flex items-center gap-1.5"
              >
                <Sparkles className="w-3.5 h-3.5" />
                <span>Get Started</span>
              </button>
            )}

            {/* Mobile Menu Button */}
            <button
              onClick={() => setMobileMenuOpen(!mobileMenuOpen)}
              className="lg:hidden p-2 rounded-xl border border-border-subtle bg-bg-surface text-text-primary"
            >
              {mobileMenuOpen ? <X className="w-5 h-5" /> : <Menu className="w-5 h-5" />}
            </button>
          </div>
        </div>
      </div>

      {/* Mobile Drawer Menu */}
      {mobileMenuOpen && (
        <div className="lg:hidden border-t border-border-subtle bg-bg-surface p-4 space-y-3 max-h-[80vh] overflow-y-auto animate-fadeIn">
          <div className="grid grid-cols-2 gap-2 text-xs font-bold">
            <button
              onClick={() => handleNav("/")}
              className="p-2.5 rounded-xl bg-bg-surface-elevated border border-border-subtle text-left text-text-primary"
            >
              Home
            </button>
            <button
              onClick={() => handleNav("/explorer")}
              className="p-2.5 rounded-xl bg-bg-surface-elevated border border-border-subtle text-left text-text-primary"
            >
              Explorer
            </button>
            <button
              onClick={() => handleNav("/learn")}
              className="p-2.5 rounded-xl bg-bg-surface-elevated border border-border-subtle text-left text-text-primary"
            >
              Learn
            </button>
            <button
              onClick={() => handleNav("/developers")}
              className="p-2.5 rounded-xl bg-bg-surface-elevated border border-border-subtle text-left text-text-primary"
            >
              Developers
            </button>
            <button
              onClick={() => handleNav("/network")}
              className="p-2.5 rounded-xl bg-bg-surface-elevated border border-border-subtle text-left text-text-primary"
            >
              Network
            </button>
            <button
              onClick={() => handleNav("/ecosystem")}
              className="p-2.5 rounded-xl bg-bg-surface-elevated border border-border-subtle text-left text-text-primary"
            >
              Ecosystem
            </button>
            <button
              onClick={() => handleNav("/governance")}
              className="p-2.5 rounded-xl bg-bg-surface-elevated border border-border-subtle text-left text-text-primary"
            >
              Governance
            </button>
            <button
              onClick={() => handleNav("/sips")}
              className="p-2.5 rounded-xl bg-bg-surface-elevated border border-border-subtle text-left text-text-primary"
            >
              SIP Proposals
            </button>
            <button
              onClick={() => handleNav("/markets")}
              className="p-2.5 rounded-xl bg-bg-surface-elevated border border-border-subtle text-left text-text-primary"
            >
              Markets
            </button>
            <button
              onClick={() => handleNav("/research")}
              className="p-2.5 rounded-xl bg-bg-surface-elevated border border-border-subtle text-left text-text-primary"
            >
              Research
            </button>
            <button
              onClick={() => handleNav("/faucet")}
              className="p-2.5 rounded-xl bg-bg-surface-elevated border border-border-subtle text-left text-text-primary"
            >
              Faucet
            </button>
            <button
              onClick={() => handleNav("/whitepaper")}
              className="p-2.5 rounded-xl bg-bg-surface-elevated border border-border-subtle text-left text-text-primary"
            >
              Whitepaper
            </button>
          </div>
        </div>
      )}
    </header>
  );
};
