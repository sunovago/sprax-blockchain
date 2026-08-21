import React, { useState, useEffect } from "react";
import {
  Compass,
  Sparkles,
  TrendingUp,
  Search,
  ExternalLink,
  Github,
  CheckCircle2,
  PlusCircle,
  TrendingDown,
} from "lucide-react";
import { Currency, EcosystemProject, MarketAsset } from "@/types";
import { apiService } from "@/services/api";
import { formatFiat } from "@/utils/formatters";

interface EcosystemPageProps {
  currency: Currency;
  initialTab?: string;
  onNavigate?: (route: string) => void;
}

export const EcosystemPage: React.FC<EcosystemPageProps> = ({
  currency,
  initialTab = "directory",
}) => {
  const [activeTab, setActiveTab] = useState<string>(() => {
    if (initialTab.includes("discover")) return "discover";
    if (initialTab.includes("markets")) return "markets";
    return "directory";
  });

  const [projects, setProjects] = useState<EcosystemProject[]>([]);
  const [marketAssets, setMarketAssets] = useState<MarketAsset[]>([]);
  const [searchQuery, setSearchQuery] = useState("");
  const [selectedCategory, setSelectedCategory] = useState<string>("All");
  const [submitModalOpen, setSubmitModalOpen] = useState(false);

  useEffect(() => {
    const loadData = async () => {
      const [projList, assetList] = await Promise.all([
        apiService.getEcosystemProjects(),
        apiService.getMarketAssets(),
      ]);
      setProjects(projList);
      setMarketAssets(assetList);
    };
    loadData();
  }, []);

  const categories = [
    "All",
    "DeFi",
    "RWA & Payments",
    "Wallets",
    "Infrastructure",
    "Enterprise",
    "Gaming & Identity",
  ];

  const filteredProjects = projects.filter((p) => {
    const matchesCategory =
      selectedCategory === "All" || p.category === selectedCategory;
    const matchesSearch =
      p.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      p.tagline.toLowerCase().includes(searchQuery.toLowerCase()) ||
      p.description.toLowerCase().includes(searchQuery.toLowerCase());
    return matchesCategory && matchesSearch;
  });

  return (
    <div className="space-y-8 animate-fadeIn">
      {/* Ecosystem Header */}
      <div className="rounded-3xl border border-border-prominent bg-gradient-to-r from-bg-surface via-bg-surface-elevated/40 to-bg-surface p-6 sm:p-10 space-y-4 shadow-card">
        <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
          <div className="space-y-2">
            <div className="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-cyan-500/10 border border-cyan-500/20 text-xs font-bold text-cyan-400">
              <Compass className="w-3.5 h-3.5" />
              <span>SPRX ECOSYSTEM PORTAL</span>
            </div>
            <h1 className="text-2xl sm:text-4xl font-extrabold text-text-primary tracking-tight">
              Ecosystem & dApps
            </h1>
            <p className="text-xs sm:text-sm text-text-secondary max-w-2xl leading-relaxed">
              Discover verified decentralized applications, real-world asset settlement platforms, and developer tooling built on SPRX Protocol.
            </p>
          </div>

          <button
            onClick={() => setSubmitModalOpen(true)}
            className="px-4 py-2.5 rounded-xl bg-gradient-to-r from-sky-500 to-indigo-600 hover:from-sky-400 hover:to-indigo-500 text-white text-xs font-bold shadow-md flex items-center gap-2 transition-all self-start sm:self-auto shrink-0"
          >
            <PlusCircle className="w-4 h-4" />
            <span>Submit a Project</span>
          </button>
        </div>

        {/* Tab Navigation */}
        <div className="pt-2 flex items-center gap-2 overflow-x-auto">
          <button
            onClick={() => setActiveTab("directory")}
            className={`px-3.5 py-1.5 rounded-xl text-xs font-bold transition-all shrink-0 ${
              activeTab === "directory"
                ? "bg-cyan-500/20 text-cyan-400 border border-cyan-500/40"
                : "bg-bg-surface border border-border-subtle text-text-secondary hover:text-text-primary"
            }`}
          >
            dApp Directory
          </button>
          <button
            onClick={() => setActiveTab("discover")}
            className={`px-3.5 py-1.5 rounded-xl text-xs font-bold transition-all shrink-0 ${
              activeTab === "discover"
                ? "bg-cyan-500/20 text-cyan-400 border border-cyan-500/40"
                : "bg-bg-surface border border-border-subtle text-text-secondary hover:text-text-primary"
            }`}
          >
            Discover Showcase
          </button>
          <button
            onClick={() => setActiveTab("markets")}
            className={`px-3.5 py-1.5 rounded-xl text-xs font-bold transition-all shrink-0 ${
              activeTab === "markets"
                ? "bg-cyan-500/20 text-cyan-400 border border-cyan-500/40"
                : "bg-bg-surface border border-border-subtle text-text-secondary hover:text-text-primary"
            }`}
          >
            Live Markets & Tickers
          </button>
        </div>
      </div>

      {/* 1. dApp Directory Tab */}
      {activeTab === "directory" && (
        <div className="space-y-6">
          {/* Filter Bar */}
          <div className="flex flex-col sm:flex-row items-center justify-between gap-3">
            {/* Search Input */}
            <div className="relative w-full sm:w-80">
              <Search className="w-4 h-4 text-text-muted absolute left-3 top-1/2 -translate-y-1/2" />
              <input
                type="text"
                placeholder="Search projects..."
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                className="w-full pl-9 pr-4 py-2 rounded-xl bg-bg-surface border border-border-subtle text-xs text-text-primary placeholder:text-text-muted focus:outline-none focus:border-cyan-400"
              />
            </div>

            {/* Category Pills */}
            <div className="flex items-center gap-1.5 overflow-x-auto w-full sm:w-auto py-1">
              {categories.map((cat) => (
                <button
                  key={cat}
                  onClick={() => setSelectedCategory(cat)}
                  className={`px-3 py-1 rounded-lg text-xs font-semibold whitespace-nowrap transition-all ${
                    selectedCategory === cat
                      ? "bg-cyan-500/20 text-cyan-400 border border-cyan-500/40"
                      : "bg-bg-surface border border-border-subtle text-text-secondary hover:text-text-primary"
                  }`}
                >
                  {cat}
                </button>
              ))}
            </div>
          </div>

          {/* Projects Grid */}
          <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
            {filteredProjects.map((proj) => (
              <div
                key={proj.id}
                className="rounded-2xl border border-border-subtle bg-bg-surface hover:bg-bg-surface-elevated hover:border-cyan-500/40 p-5 transition-all flex flex-col justify-between space-y-4 shadow-sm"
              >
                <div className="space-y-3">
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-2">
                      <span className="text-xs font-bold px-2 py-0.5 rounded-full bg-cyan-500/10 text-cyan-400 border border-cyan-500/20">
                        {proj.category}
                      </span>
                      {proj.verified && (
                        <span className="inline-flex items-center gap-1 text-[10px] font-bold text-emerald-400">
                          <CheckCircle2 className="w-3.5 h-3.5" />
                          <span>Verified</span>
                        </span>
                      )}
                    </div>

                    <span className="text-[10px] font-semibold px-2 py-0.5 rounded bg-bg-surface-elevated text-text-muted border border-border-subtle">
                      {proj.stage}
                    </span>
                  </div>

                  <div>
                    <h3 className="text-base font-bold text-text-primary">
                      {proj.name}
                    </h3>
                    <p className="text-xs text-cyan-400/80 font-medium mt-0.5">
                      {proj.tagline}
                    </p>
                  </div>

                  <p className="text-xs text-text-secondary leading-relaxed line-clamp-3">
                    {proj.description}
                  </p>
                </div>

                <div className="pt-3 border-t border-border-subtle flex items-center justify-between">
                  {proj.metrics ? (
                    <div className="text-[11px]">
                      <span className="text-text-muted">{proj.metrics.label}: </span>
                      <span className="font-bold text-text-primary font-mono-num">
                        {proj.metrics.value}
                      </span>
                    </div>
                  ) : (
                    <div />
                  )}

                  <div className="flex items-center gap-2">
                    {proj.github && (
                      <a
                        href={proj.github}
                        target="_blank"
                        rel="noreferrer"
                        className="p-1.5 rounded-lg text-text-muted hover:text-text-primary hover:bg-bg-surface transition-colors"
                        title="GitHub Repository"
                      >
                        <Github className="w-4 h-4" />
                      </a>
                    )}
                    <a
                      href={proj.website}
                      target="_blank"
                      rel="noreferrer"
                      className="inline-flex items-center gap-1 px-2.5 py-1 rounded-lg bg-cyan-500/10 text-cyan-400 border border-cyan-500/30 text-xs font-bold hover:bg-cyan-500/20 transition-all"
                    >
                      <span>Launch</span>
                      <ExternalLink className="w-3 h-3" />
                    </a>
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* 2. Discover Showcase Tab */}
      {activeTab === "discover" && (
        <div className="space-y-6">
          <div className="rounded-2xl border border-sky-500/30 bg-gradient-to-r from-sky-950/40 via-bg-surface to-indigo-950/40 p-6 sm:p-8 space-y-4">
            <div className="inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full bg-sky-500/10 border border-sky-500/20 text-xs font-bold text-sky-400">
              <Sparkles className="w-3.5 h-3.5" />
              <span>FEATURED PROTOCOL SPOTLIGHT</span>
            </div>
            <h2 className="text-xl sm:text-2xl font-extrabold text-text-primary">
              SpraxSwap: Next-Generation Concentrated AMM
            </h2>
            <p className="text-xs sm:text-sm text-text-secondary max-w-2xl leading-relaxed">
              Ultra-low latency decentralized exchange powered by SPRX's 1.5s single-block finality and native multi-currency fee abstraction.
            </p>
            <a
              href="https://swap.sprax.network"
              target="_blank"
              rel="noreferrer"
              className="inline-flex items-center gap-1.5 px-4 py-2 rounded-xl bg-gradient-to-r from-sky-500 to-indigo-600 text-white text-xs font-bold shadow-md hover:from-sky-400 hover:to-indigo-500 transition-all"
            >
              <span>Explore SpraxSwap</span>
              <ExternalLink className="w-3.5 h-3.5" />
            </a>
          </div>
        </div>
      )}

      {/* 3. Live Markets Tab */}
      {activeTab === "markets" && (
        <div className="rounded-2xl border border-border-subtle bg-bg-surface p-5 space-y-4 shadow-card">
          <div className="flex items-center justify-between">
            <div>
              <h3 className="text-base font-bold text-text-primary">
                SPRX Ecosystem Asset Tickers
              </h3>
              <p className="text-xs text-text-muted mt-0.5">
                Real-time market valuation converted into {currency}
              </p>
            </div>
          </div>

          <div className="overflow-x-auto">
            <table className="w-full text-left text-xs border border-border-subtle rounded-xl overflow-hidden">
              <thead className="bg-bg-surface-elevated text-text-muted uppercase text-[10px] font-bold">
                <tr>
                  <th className="p-3">Asset</th>
                  <th className="p-3">Price ({currency})</th>
                  <th className="p-3">24h Change</th>
                  <th className="p-3">24h High / Low</th>
                  <th className="p-3">24h Volume</th>
                  <th className="p-3">Market Cap</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-border-subtle font-medium">
                {marketAssets.map((asset) => {
                  const isPositive = asset.change24h >= 0;

                  return (
                    <tr key={asset.id} className="hover:bg-bg-surface-elevated/40 transition-colors">
                      <td className="p-3">
                        <div className="flex items-center gap-2">
                          <div className="w-7 h-7 rounded-lg bg-cyan-500/10 text-cyan-400 font-bold flex items-center justify-center text-xs">
                            {asset.symbol.slice(0, 2)}
                          </div>
                          <div>
                            <span className="font-bold text-text-primary block">{asset.name}</span>
                            <span className="text-[10px] text-text-muted font-mono">{asset.symbol}</span>
                          </div>
                        </div>
                      </td>
                      <td className="p-3 font-bold font-mono-num text-text-primary">
                        {formatFiat(asset.priceUsd, currency)}
                      </td>
                      <td className="p-3">
                        <span
                          className={`inline-flex items-center gap-1 font-bold ${
                            isPositive ? "text-emerald-400" : "text-coral-400"
                          }`}
                        >
                          {isPositive ? (
                            <TrendingUp className="w-3.5 h-3.5" />
                          ) : (
                            <TrendingDown className="w-3.5 h-3.5" />
                          )}
                          <span>{isPositive ? `+${asset.change24h}%` : `${asset.change24h}%`}</span>
                        </span>
                      </td>
                      <td className="p-3 text-text-secondary font-mono-num">
                        {formatFiat(asset.high24h, currency)} / {formatFiat(asset.low24h, currency)}
                      </td>
                      <td className="p-3 text-text-secondary font-mono-num">
                        {formatFiat(asset.volume24hUsd, currency)}
                      </td>
                      <td className="p-3 text-text-primary font-bold font-mono-num">
                        {formatFiat(asset.marketCapUsd, currency)}
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        </div>
      )}

      {/* Project Submission Modal */}
      {submitModalOpen && (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/80 backdrop-blur-sm animate-fadeIn">
          <div className="relative w-full max-w-lg rounded-2xl border border-border-prominent bg-bg-surface p-6 space-y-4 shadow-2xl">
            <div className="flex items-center justify-between">
              <h3 className="text-lg font-bold text-text-primary">
                Submit Your Project to SPRX Ecosystem
              </h3>
              <button
                onClick={() => setSubmitModalOpen(false)}
                className="text-text-muted hover:text-text-primary p-1"
              >
                ✕
              </button>
            </div>
            <p className="text-xs text-text-secondary leading-relaxed">
              Submit your dApp, wallet, or developer tool for verification and listing on the official SPRX Ecosystem portal.
            </p>
            <div className="space-y-3 text-xs">
              <div>
                <label className="text-text-muted block mb-1">Project Name</label>
                <input
                  type="text"
                  placeholder="e.g. MyDEX Protocol"
                  className="w-full p-2.5 rounded-xl bg-bg-surface-elevated border border-border-subtle text-text-primary focus:outline-none focus:border-cyan-400"
                />
              </div>
              <div>
                <label className="text-text-muted block mb-1">Website URL</label>
                <input
                  type="url"
                  placeholder="https://mydex.io"
                  className="w-full p-2.5 rounded-xl bg-bg-surface-elevated border border-border-subtle text-text-primary focus:outline-none focus:border-cyan-400"
                />
              </div>
              <div>
                <label className="text-text-muted block mb-1">Category</label>
                <select className="w-full p-2.5 rounded-xl bg-bg-surface-elevated border border-border-subtle text-text-primary focus:outline-none focus:border-cyan-400">
                  <option>DeFi</option>
                  <option>RWA & Payments</option>
                  <option>Wallets</option>
                  <option>Infrastructure</option>
                  <option>Enterprise</option>
                  <option>Gaming & Identity</option>
                </select>
              </div>
            </div>
            <div className="pt-2 flex items-center justify-end gap-2">
              <button
                onClick={() => setSubmitModalOpen(false)}
                className="px-4 py-2 rounded-xl bg-bg-surface-elevated text-xs font-semibold text-text-secondary hover:text-text-primary"
              >
                Cancel
              </button>
              <button
                onClick={() => {
                  alert("Project application received. The SPRX Ecosystem Guild will review your submission.");
                  setSubmitModalOpen(false);
                }}
                className="px-4 py-2 rounded-xl bg-gradient-to-r from-sky-500 to-indigo-600 text-white text-xs font-bold shadow-md hover:from-sky-400 hover:to-indigo-500"
              >
                Submit Application
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
