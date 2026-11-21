import React, { useState, useEffect, useRef } from "react";
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
import { Currency, EcosystemProject, MarketAsset, Network } from "@/types";
import { apiService } from "@/services/api";
import { formatFiat } from "@/utils/formatters";

interface EcosystemPageProps {
  currency: Currency;
  network?: Network;
  initialTab?: string;
  onNavigate?: (route: string) => void;
}

export const EcosystemPage: React.FC<EcosystemPageProps> = ({
  currency,
  initialTab = "directory",
  network = "local",
  onNavigate,
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

  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");
  const [reload, setReload] = useState(0);
  const [previewProjects, setPreviewProjects] = useState(false);
  const [previewMarkets, setPreviewMarkets] = useState(false);
  const [selectedProject, setSelectedProject] = useState<EcosystemProject | null>(null);
  const [draftMessage, setDraftMessage] = useState("");

  useEffect(() => {
    setActiveTab(initialTab.includes("discover") ? "discover" : initialTab.includes("markets") ? "markets" : "directory");
    setSelectedProject(null);
    setSubmitModalOpen(false);
  }, [initialTab]);

  const changeTab = (tab: string) => {
    setActiveTab(tab);
    onNavigate?.(tab === "directory" ? "/ecosystem" : `/${tab}`);
  };

  useEffect(() => {
    let cancelled = false;
    let pending = false;
    const loadData = async () => {
      if (pending) return;
      pending = true;
      try {
        const [projList, assetList] = await Promise.all([
          apiService.getEcosystemProjects(), apiService.getMarketAssets(),
        ]);
        if (!cancelled) {
          setProjects(projList); setMarketAssets(assetList);
          setPreviewProjects(apiService.ecosystemIsPreview);
          setPreviewMarkets(apiService.marketsArePreview);
          setError("");
        }
      } catch {
        if (!cancelled) setError("Unable to load ecosystem data. Please try again.");
      } finally {
        pending = false;
        if (!cancelled) setLoading(false);
      }
    };
    setLoading(true);
    void loadData();
    const timer = window.setInterval(loadData, 30000);
    return () => { cancelled = true; window.clearInterval(timer); };
  }, [network, reload]);

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
    <div className="min-w-0 space-y-6 animate-fadeIn">
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
            onClick={() => { setDraftMessage(""); setSubmitModalOpen(true); }}
            className="px-4 py-2.5 rounded-xl bg-gradient-to-r from-sky-500 to-indigo-600 hover:from-sky-400 hover:to-indigo-500 text-white text-xs font-bold shadow-md flex items-center gap-2 transition-all self-start sm:self-auto shrink-0"
          >
            <PlusCircle className="w-4 h-4" />
            <span>Submit a Project</span>
          </button>
        </div>

        {/* Tab Navigation */}
        <div className="pt-2 flex flex-wrap items-center gap-2">
          <button
            onClick={() => changeTab("directory")}
            className={`px-3.5 py-1.5 rounded-xl text-xs font-bold transition-all shrink-0 ${
              activeTab === "directory"
                ? "bg-cyan-500/20 text-cyan-400 border border-cyan-500/40"
                : "bg-bg-surface border border-border-subtle text-text-secondary hover:text-text-primary"
            }`}
          >
            dApp Directory
          </button>
          <button
            onClick={() => changeTab("discover")}
            className={`px-3.5 py-1.5 rounded-xl text-xs font-bold transition-all shrink-0 ${
              activeTab === "discover"
                ? "bg-cyan-500/20 text-cyan-400 border border-cyan-500/40"
                : "bg-bg-surface border border-border-subtle text-text-secondary hover:text-text-primary"
            }`}
          >
            Discover Showcase
          </button>
          <button
            onClick={() => changeTab("markets")}
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

      {loading && <p role="status" className="text-sm text-text-secondary">Loading ecosystem data...</p>}
      {error && <div role="alert" className="rounded-xl border border-amber-500/30 p-4 text-sm">{error} <button className="text-cyan-400 underline" onClick={() => setReload(n => n + 1)}>Retry</button></div>}
      {!loading && (activeTab === "markets" ? previewMarkets : previewProjects) && (
        <div role="status" className="rounded-xl border border-amber-500/30 bg-amber-500/10 p-4 text-sm text-amber-400">
          Backend unavailable - showing sample {activeTab === "markets" ? "market data, not live prices" : "projects, not verified listings"}.
          <button className="ml-2 underline" onClick={() => setReload(n => n + 1)}>Retry connection</button>
        </div>
      )}
      {draftMessage && <p role="status" className="text-sm text-cyan-400">{draftMessage}</p>}
      {/* 1. dApp Directory Tab */}
      {activeTab === "directory" && (
        <div className="space-y-6">
          {/* Filter Bar */}
          <div className="flex flex-col xl:flex-row xl:items-center justify-between gap-3">
            {/* Search Input */}
            <div className="relative w-full xl:w-72 xl:shrink-0">
              <Search className="w-4 h-4 text-text-muted absolute left-3 top-1/2 -translate-y-1/2" />
              <input
                type="text"
                aria-label="Search projects"
                placeholder="Search projects..."
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                className="w-full pl-9 pr-4 py-2 rounded-xl bg-bg-surface border border-border-subtle text-xs text-text-primary placeholder:text-text-muted focus:outline-none focus:border-cyan-400"
              />
            </div>

            {/* Category Pills */}
            <div className="flex flex-wrap items-center gap-1.5 w-full xl:w-auto py-1">
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

          {!loading && !filteredProjects.length && <div className="rounded-xl border border-border-subtle p-8 text-center text-text-secondary">No projects match your search. <button onClick={() => { setSearchQuery(""); setSelectedCategory("All"); }} className="text-cyan-400 underline">Clear filters</button></div>}
          {/* Projects Grid */}
          <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
            {filteredProjects.map((proj) => (
              <div
                key={proj.id}
                className="rounded-2xl border border-border-subtle bg-bg-surface hover:bg-bg-surface-elevated hover:border-cyan-500/40 p-5 transition-all flex flex-col justify-between space-y-4 shadow-sm"
              >
                <div className="space-y-3">
                  <div className="flex flex-wrap items-center justify-between gap-2">
                    <div className="flex flex-wrap items-center gap-2">
                      <span className="text-xs font-bold px-2 py-0.5 rounded-full bg-cyan-500/10 text-cyan-400 border border-cyan-500/20">
                        {proj.category}
                      </span>
                      {proj.verified && !previewProjects && (
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
                    {!previewProjects && safeWebsite(proj.github) && (
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
                    <button onClick={() => setSelectedProject(proj)} className="inline-flex items-center gap-1 px-2.5 py-1 rounded-lg bg-cyan-500/10 text-cyan-400 border border-cyan-500/30 text-xs font-bold hover:bg-cyan-500/20">
                      View details
                    </button>
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Discover uses the same directory data, rather than a hardcoded dead link. */}
      {activeTab === "discover" && (
        <section className="space-y-4" aria-label="Discover Showcase">
          <h2 className="text-xl font-bold flex items-center gap-2"><Sparkles className="w-5 h-5 text-cyan-400" /> Discover Showcase</h2>
          {!loading && !projects.length && <p className="text-text-secondary">No projects are available yet.</p>}
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            {projects.map(project => (
              <article key={project.id} className="rounded-2xl border border-border-subtle bg-bg-surface p-6 space-y-3">
                <span className="text-xs font-semibold text-cyan-400">{project.category}</span>
                <h3 className="text-lg font-bold">{project.name}</h3>
                <p className="text-sm text-text-secondary">{project.description}</p>
                <button onClick={() => setSelectedProject(project)} className="text-sm font-semibold text-cyan-400">Explore project</button>
              </article>
            ))}
          </div>
        </section>
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
                {previewMarkets ? "Sample prices" : "Market data refreshed every 30 seconds"} - Displayed in {currency}
              </p>
            </div>
          </div>

          {!loading && !marketAssets.length && <p className="text-sm text-text-secondary">No market data is available yet.</p>}
          <div className="max-w-full overflow-x-auto" tabIndex={0} role="region" aria-label="Market prices">
            <table className="w-full min-w-[680px] text-left text-xs border border-border-subtle rounded-xl overflow-hidden">
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

      {selectedProject && (
        <EcosystemDialog title={selectedProject.name} onClose={() => setSelectedProject(null)}>
          <p className="text-cyan-400 text-sm">{selectedProject.category} - {previewProjects ? "Sample project" : selectedProject.stage}</p>
          <p className="text-sm text-text-secondary leading-relaxed">{selectedProject.description}</p>
          {previewProjects ? <p className="text-sm text-amber-400">This is a sample listing. A live project website has not been confirmed.</p> : safeWebsite(selectedProject.website) ? (
            <a href={safeWebsite(selectedProject.website)} target="_blank" rel="noopener noreferrer" className="inline-flex items-center gap-2 rounded-xl bg-cyan-500/10 p-3 text-cyan-400">Launch project <ExternalLink className="w-4 h-4" /></a>
          ) : <p className="text-sm text-text-muted">No project website has been provided.</p>}
        </EcosystemDialog>
      )}
      {submitModalOpen && (
        <EcosystemDialog title="Project application" onClose={() => setSubmitModalOpen(false)}>
          <p className="text-sm text-text-secondary">Prepare a project application. Online submissions are not available yet; you can save a draft on this device.</p>
          {draftMessage && <p role="alert" className="text-sm text-amber-400">{draftMessage}</p>}
          <form className="space-y-4" onSubmit={event => {
            event.preventDefault();
            const values = new FormData(event.currentTarget);
            const draft = { name: String(values.get("name")).trim(), website: String(values.get("website")).trim(), category: String(values.get("category")) };
            if (!draft.name || !safeWebsite(draft.website)) { setDraftMessage("Enter a project name and a valid HTTP or HTTPS website."); return; }
            try {
              localStorage.setItem("sprx_project_application_draft", JSON.stringify(draft));
              setDraftMessage("Application draft saved on this device. It has not been submitted for review.");
              setSubmitModalOpen(false);
            } catch { setDraftMessage("Unable to save the draft. Browser storage may be full or disabled."); }
          }}>
            <label className="block text-sm">Project Name<input name="name" required maxLength={120} defaultValue={readDraft().name} className="mt-1 w-full p-3 rounded-xl bg-bg-elevated border border-border-subtle" /></label>
            <label className="block text-sm">Website URL<input name="website" type="url" required defaultValue={readDraft().website} placeholder="https://example.com" className="mt-1 w-full p-3 rounded-xl bg-bg-elevated border border-border-subtle" /></label>
            <label className="block text-sm">Category<select name="category" defaultValue={readDraft().category ?? "DeFi"} className="mt-1 w-full p-3 rounded-xl bg-bg-elevated border border-border-subtle">{categories.filter(c => c !== "All").map(c => <option key={c}>{c}</option>)}</select></label>
            <div className="flex justify-end gap-3"><button type="button" onClick={() => setSubmitModalOpen(false)} className="px-4 py-2 text-text-secondary">Cancel</button><button type="submit" className="rounded-xl px-4 py-2 bg-cyan-500 text-black font-semibold">Save draft</button></div>
          </form>
        </EcosystemDialog>
      )}
    </div>
  );
};

function safeWebsite(value?: string): string | undefined {
  try { const url = new URL(value ?? ""); return ["http:", "https:"].includes(url.protocol) ? url.href : undefined; } catch { return undefined; }
}

function readDraft(): { name?: string; website?: string; category?: string } {
  try { return JSON.parse(localStorage.getItem("sprx_project_application_draft") ?? "{}") ?? {}; } catch { return {}; }
}

function EcosystemDialog({ title, onClose, children }: { title: string; onClose: () => void; children: React.ReactNode }) {
  const ref = useRef<HTMLDialogElement>(null);
  useEffect(() => {
    const previousFocus = document.activeElement as HTMLElement | null;
    const previousOverflow = document.body.style.overflow;
    document.body.style.overflow = "hidden";
    ref.current?.showModal();
    return () => { document.body.style.overflow = previousOverflow; previousFocus?.focus(); };
  }, []);
  return <dialog ref={ref} aria-labelledby="ecosystem-dialog-title" onCancel={onClose} onClick={event => { if (event.target === event.currentTarget) onClose(); }} className="m-auto w-[calc(100%-2rem)] max-w-lg max-h-[85dvh] overflow-y-auto rounded-2xl border border-border-strong bg-bg-surface text-text-primary p-0 backdrop:bg-black/80">
    <div className="p-5 sm:p-6 space-y-4">
      <div className="flex items-start justify-between gap-4"><h2 id="ecosystem-dialog-title" className="text-lg font-bold">{title}</h2><button aria-label="Close dialog" onClick={onClose} className="p-2 text-text-secondary">Close</button></div>
      {children}
    </div>
  </dialog>;
}
