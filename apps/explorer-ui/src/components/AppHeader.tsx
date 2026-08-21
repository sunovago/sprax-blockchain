import React, { useState } from "react";
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
} from "lucide-react";
import { Currency, Network, ThemeMode } from "@/types";
import { FIAT_RATES } from "@/utils/formatters";

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
}) => {
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false);
  const [blockchainOpen, setBlockchainOpen] = useState(false);
  const [devsOpen, setDevsOpen] = useState(false);

  const isTestnet = network !== "mainnet";

  const handleNav = (route: string) => {
    onNavigate(route);
    setMobileMenuOpen(false);
    setBlockchainOpen(false);
    setDevsOpen(false);
  };

  return (
    <header className="sticky top-0 z-40 w-full border-b border-border-subtle glass-panel">
      {/* Testnet Alert Bar if in testnet or devnet */}
      {isTestnet && (
        <div className="bg-purple-950/60 border-b border-purple-800/40 px-4 py-1 text-center text-xs font-semibold text-purple-300 flex items-center justify-center gap-2">
          <span className="w-2 h-2 rounded-full bg-purple-400 animate-pulse" />
          <span>CONNECTED TO {network.toUpperCase()} ENVIRONMENT — NOT FOR MAINNET ASSETS</span>
        </div>
      )}

      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div className="flex items-center justify-between h-16 gap-2 sm:gap-4">
          {/* Logo & Branding */}
          <div
            className="flex items-center gap-3 cursor-pointer select-none shrink-0"
            onClick={() => handleNav("/")}
          >
            {/* SPRX Custom SVG Logo Mark */}
            <div className="relative flex items-center justify-center w-9 h-9 rounded-xl bg-gradient-to-br from-sky-500 via-sky-600 to-indigo-700 shadow-md border border-sky-400/30">
              <span className="font-black text-white text-base tracking-tighter font-sans">
                X
              </span>
              <span className="absolute -top-1 -right-1 w-2.5 h-2.5 rounded-full bg-emerald-400 ring-2 ring-bg-surface" />
            </div>

            <div>
              <div className="flex items-center gap-1.5">
                <span className="text-base font-extrabold tracking-tight text-text-primary">
                  SPRX
                </span>
                <span className="text-xs font-semibold px-1.5 py-0.2 rounded bg-sky-500/10 text-sky-400 border border-sky-500/20 uppercase">
                  Explorer
                </span>
              </div>
              <p className="text-[10px] text-text-muted hidden sm:block tracking-wide">
                Scalable Protocol for Real-world X
              </p>
            </div>
          </div>

          {/* Desktop Navigation */}
          <nav className="hidden lg:flex items-center gap-1 text-sm font-medium text-text-secondary">
            <button
              type="button"
              onClick={() => handleNav("/")}
              className={`px-3 py-1.5 rounded-lg transition-colors ${
                currentRoute === "/" ? "text-sky-400 bg-bg-surface-elevated font-semibold" : "hover:text-text-primary hover:bg-bg-hover"
              }`}
            >
              Home
            </button>

            {/* Blockchain Dropdown */}
            <div className="relative">
              <button
                type="button"
                onClick={() => {
                  setBlockchainOpen(!blockchainOpen);
                  setDevsOpen(false);
                }}
                className={`flex items-center gap-1 px-3 py-1.5 rounded-lg transition-colors ${
                  ["/blocks", "/transactions", "/network"].includes(currentRoute)
                    ? "text-sky-400 bg-bg-surface-elevated font-semibold"
                    : "hover:text-text-primary hover:bg-bg-hover"
                }`}
              >
                <span>Blockchain</span>
                <ChevronDown className="w-3.5 h-3.5" />
              </button>

              {blockchainOpen && (
                <div
                  className="absolute left-0 mt-2 w-48 rounded-xl border border-border-strong bg-bg-surface p-1.5 shadow-elevated z-50 animate-scaleUp"
                  onMouseLeave={() => setBlockchainOpen(false)}
                >
                  <button
                    type="button"
                    onClick={() => handleNav("/blocks")}
                    className="w-full flex items-center gap-2.5 px-3 py-2 rounded-lg text-xs font-medium text-text-secondary hover:text-text-primary hover:bg-bg-hover text-left"
                  >
                    <Box className="w-4 h-4 text-sky-400" />
                    <span>Blocks</span>
                  </button>
                  <button
                    type="button"
                    onClick={() => handleNav("/transactions")}
                    className="w-full flex items-center gap-2.5 px-3 py-2 rounded-lg text-xs font-medium text-text-secondary hover:text-text-primary hover:bg-bg-hover text-left"
                  >
                    <Layers className="w-4 h-4 text-emerald-400" />
                    <span>Transactions</span>
                  </button>
                  <button
                    type="button"
                    onClick={() => handleNav("/network")}
                    className="w-full flex items-center gap-2.5 px-3 py-2 rounded-lg text-xs font-medium text-text-secondary hover:text-text-primary hover:bg-bg-hover text-left"
                  >
                    <Activity className="w-4 h-4 text-purple-400" />
                    <span>Network Health</span>
                  </button>
                  <button
                    type="button"
                    onClick={() => handleNav("/analytics")}
                    className="w-full flex items-center gap-2.5 px-3 py-2 rounded-lg text-xs font-medium text-text-secondary hover:text-text-primary hover:bg-bg-hover text-left"
                  >
                    <Zap className="w-4 h-4 text-amber-400" />
                    <span>Analytics</span>
                  </button>
                </div>
              )}
            </div>

            <button
              type="button"
              onClick={() => handleNav("/validators")}
              className={`px-3 py-1.5 rounded-lg transition-colors ${
                currentRoute.startsWith("/validator")
                  ? "text-sky-400 bg-bg-surface-elevated font-semibold"
                  : "hover:text-text-primary hover:bg-bg-hover"
              }`}
            >
              Validators
            </button>

            <button
              type="button"
              onClick={() => handleNav("/staking")}
              className={`px-3 py-1.5 rounded-lg transition-colors ${
                currentRoute === "/staking"
                  ? "text-sky-400 bg-bg-surface-elevated font-semibold"
                  : "hover:text-text-primary hover:bg-bg-hover"
              }`}
            >
              Staking
            </button>

            <button
              type="button"
              onClick={() => handleNav("/contracts")}
              className={`px-3 py-1.5 rounded-lg transition-colors ${
                currentRoute.startsWith("/contract")
                  ? "text-sky-400 bg-bg-surface-elevated font-semibold"
                  : "hover:text-text-primary hover:bg-bg-hover"
              }`}
            >
              Contracts
            </button>

            {/* Developers Dropdown */}
            <div className="relative">
              <button
                type="button"
                onClick={() => {
                  setDevsOpen(!devsOpen);
                  setBlockchainOpen(false);
                }}
                className={`flex items-center gap-1 px-3 py-1.5 rounded-lg transition-colors ${
                  ["/developers", "/faucet"].includes(currentRoute)
                    ? "text-sky-400 bg-bg-surface-elevated font-semibold"
                    : "hover:text-text-primary hover:bg-bg-hover"
                }`}
              >
                <span>Developers</span>
                <ChevronDown className="w-3.5 h-3.5" />
              </button>

              {devsOpen && (
                <div
                  className="absolute left-0 mt-2 w-48 rounded-xl border border-border-strong bg-bg-surface p-1.5 shadow-elevated z-50 animate-scaleUp"
                  onMouseLeave={() => setDevsOpen(false)}
                >
                  <button
                    type="button"
                    onClick={() => handleNav("/developers")}
                    className="w-full flex items-center gap-2.5 px-3 py-2 rounded-lg text-xs font-medium text-text-secondary hover:text-text-primary hover:bg-bg-hover text-left"
                  >
                    <Code2 className="w-4 h-4 text-sky-400" />
                    <span>RPC & API Portal</span>
                  </button>
                  <button
                    type="button"
                    onClick={() => handleNav("/faucet")}
                    className="w-full flex items-center gap-2.5 px-3 py-2 rounded-lg text-xs font-medium text-text-secondary hover:text-text-primary hover:bg-bg-hover text-left"
                  >
                    <Coins className="w-4 h-4 text-amber-400" />
                    <span>Testnet Faucet</span>
                  </button>
                </div>
              )}
            </div>
          </nav>

          {/* Right Header Actions */}
          <div className="flex items-center gap-2">
            {/* Universal Search Trigger */}
            <button
              type="button"
              onClick={onOpenSearch}
              className="flex items-center gap-2 px-2.5 sm:px-3.5 py-1.5 rounded-xl border border-border-subtle bg-bg-surface-elevated text-xs text-text-muted hover:text-text-primary hover:border-border-strong transition-all shadow-sm"
              title="Search Sprax Chain (Ctrl+K)"
            >
              <Search className="w-3.5 h-3.5 text-sky-400" />
              <span className="hidden md:inline font-normal">Search block / tx / address...</span>
              <kbd className="hidden sm:inline-block px-1.5 py-0.5 rounded bg-bg-primary text-[10px] font-mono border border-border-subtle">
                ⌘K
              </kbd>
            </button>

            {/* Currency Selector */}
            <select
              value={currency}
              onChange={(e) => onCurrencyChange(e.target.value as Currency)}
              aria-label="Select fiat currency"
              className="bg-bg-surface-elevated border border-border-subtle text-text-secondary text-xs rounded-lg px-2 py-1.5 focus:outline-none focus:border-sky-500 font-mono"
            >
              {Object.keys(FIAT_RATES).map((c) => (
                <option key={c} value={c}>
                  {c} ({FIAT_RATES[c as Currency].prefix})
                </option>
              ))}
            </select>

            {/* Network Selector */}
            <select
              value={network}
              onChange={(e) => onNetworkChange(e.target.value as Network)}
              aria-label="Select blockchain network"
              className="hidden sm:block bg-bg-surface-elevated border border-border-subtle text-text-secondary text-xs rounded-lg px-2 py-1.5 focus:outline-none focus:border-sky-500 font-medium"
            >
              <option value="mainnet">● Mainnet</option>
              <option value="testnet">● Testnet</option>
              <option value="local">● Devnet</option>
            </select>

            {/* Theme Toggle */}
            <button
              type="button"
              onClick={onToggleTheme}
              className="p-1.5 rounded-lg border border-border-subtle bg-bg-surface-elevated text-text-secondary hover:text-text-primary hover:border-border-strong transition-colors"
              title={`Switch to ${theme === "dark" ? "light" : "dark"} theme`}
              aria-label="Toggle theme"
            >
              {theme === "dark" ? (
                <Sun className="w-4 h-4 text-amber-400" />
              ) : (
                <Moon className="w-4 h-4 text-indigo-500" />
              )}
            </button>

            {/* Mobile Menu Button */}
            <button
              type="button"
              onClick={() => setMobileMenuOpen(!mobileMenuOpen)}
              className="lg:hidden p-1.5 rounded-lg border border-border-subtle bg-bg-surface-elevated text-text-secondary hover:text-text-primary"
              aria-label="Open mobile menu"
            >
              {mobileMenuOpen ? <X className="w-5 h-5" /> : <Menu className="w-5 h-5" />}
            </button>
          </div>
        </div>
      </div>

      {/* Mobile Drawer Navigation */}
      {mobileMenuOpen && (
        <div className="lg:hidden border-t border-border-subtle bg-bg-surface px-4 py-4 space-y-2 animate-fadeIn">
          <button
            type="button"
            onClick={() => handleNav("/")}
            className="w-full flex items-center gap-3 px-3 py-2 rounded-lg text-sm text-text-primary hover:bg-bg-hover text-left"
          >
            <Cpu className="w-4 h-4 text-sky-400" />
            <span>Home</span>
          </button>
          <button
            type="button"
            onClick={() => handleNav("/blocks")}
            className="w-full flex items-center gap-3 px-3 py-2 rounded-lg text-sm text-text-primary hover:bg-bg-hover text-left"
          >
            <Box className="w-4 h-4 text-sky-400" />
            <span>Blocks</span>
          </button>
          <button
            type="button"
            onClick={() => handleNav("/transactions")}
            className="w-full flex items-center gap-3 px-3 py-2 rounded-lg text-sm text-text-primary hover:bg-bg-hover text-left"
          >
            <Layers className="w-4 h-4 text-emerald-400" />
            <span>Transactions</span>
          </button>
          <button
            type="button"
            onClick={() => handleNav("/validators")}
            className="w-full flex items-center gap-3 px-3 py-2 rounded-lg text-sm text-text-primary hover:bg-bg-hover text-left"
          >
            <ShieldCheck className="w-4 h-4 text-amber-400" />
            <span>Validators</span>
          </button>
          <button
            type="button"
            onClick={() => handleNav("/staking")}
            className="w-full flex items-center gap-3 px-3 py-2 rounded-lg text-sm text-text-primary hover:bg-bg-hover text-left"
          >
            <Coins className="w-4 h-4 text-purple-400" />
            <span>Staking</span>
          </button>
          <button
            type="button"
            onClick={() => handleNav("/contracts")}
            className="w-full flex items-center gap-3 px-3 py-2 rounded-lg text-sm text-text-primary hover:bg-bg-hover text-left"
          >
            <FileCode2 className="w-4 h-4 text-indigo-400" />
            <span>Smart Contracts</span>
          </button>
          <button
            type="button"
            onClick={() => handleNav("/analytics")}
            className="w-full flex items-center gap-3 px-3 py-2 rounded-lg text-sm text-text-primary hover:bg-bg-hover text-left"
          >
            <Activity className="w-4 h-4 text-cyan-400" />
            <span>Analytics</span>
          </button>
          <button
            type="button"
            onClick={() => handleNav("/developers")}
            className="w-full flex items-center gap-3 px-3 py-2 rounded-lg text-sm text-text-primary hover:bg-bg-hover text-left"
          >
            <Code2 className="w-4 h-4 text-rose-400" />
            <span>Developers & RPC</span>
          </button>
          <button
            type="button"
            onClick={() => handleNav("/faucet")}
            className="w-full flex items-center gap-3 px-3 py-2 rounded-lg text-sm text-text-primary hover:bg-bg-hover text-left"
          >
            <Coins className="w-4 h-4 text-amber-400" />
            <span>Testnet Faucet</span>
          </button>

          <div className="pt-2 border-t border-border-subtle flex items-center justify-between">
            <span className="text-xs text-text-muted">Network:</span>
            <select
              value={network}
              onChange={(e) => onNetworkChange(e.target.value as Network)}
              aria-label="Select mobile network"
              className="bg-bg-surface-elevated border border-border-subtle text-text-secondary text-xs rounded-lg px-2 py-1"
            >
              <option value="mainnet">SPRX Mainnet</option>
              <option value="testnet">SPRX Testnet</option>
              <option value="local">Local Devnet</option>
            </select>
          </div>
        </div>
      )}
    </header>
  );
};
