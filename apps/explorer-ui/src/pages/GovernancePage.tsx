import React, { useState, useEffect } from "react";
import {
  Vote,
  CheckCircle2,
  ChevronRight,
} from "lucide-react";
import { GovernanceProposal, SIPItem } from "@/types";
import { apiService } from "@/services/api";

interface GovernancePageProps {
  initialTab?: string;
  onNavigate?: (route: string) => void;
}

export const GovernancePage: React.FC<GovernancePageProps> = ({
  initialTab = "voting",
}) => {
  const [activeTab, setActiveTab] = useState<string>(() => {
    if (initialTab.includes("sips")) return "sips";
    if (initialTab.includes("grants")) return "grants";
    if (initialTab.includes("bug-bounty")) return "bug-bounty";
    return "voting";
  });

  const [proposals, setProposals] = useState<GovernanceProposal[]>([]);
  const [sips, setSips] = useState<SIPItem[]>([]);
  const [selectedSip, setSelectedSip] = useState<SIPItem | null>(null);
  const [sipFilterCategory, setSipFilterCategory] = useState<string>("All");
  const [sipFilterStatus, setSipFilterStatus] = useState<string>("All");
  const [votedProposals, setVotedProposals] = useState<Record<number, string>>({});

  useEffect(() => {
    const loadData = async () => {
      const [govList, sipList] = await Promise.all([
        apiService.getGovernanceProposals(),
        apiService.getSIPs(),
      ]);
      setProposals(govList);
      setSips(sipList);
    };
    loadData();
  }, []);

  const handleVote = (proposalId: number, option: string) => {
    setVotedProposals((prev) => ({ ...prev, [proposalId]: option }));
  };

  const filteredSips = sips.filter((s) => {
    const matchesCat =
      sipFilterCategory === "All" || s.category === sipFilterCategory;
    const matchesStatus =
      sipFilterStatus === "All" || s.status === sipFilterStatus;
    return matchesCat && matchesStatus;
  });

  return (
    <div className="space-y-8 animate-fadeIn">
      {/* Governance Banner */}
      <div className="rounded-3xl border border-border-prominent bg-gradient-to-r from-bg-surface via-bg-surface-elevated/40 to-bg-surface p-6 sm:p-10 space-y-4 shadow-card">
        <div className="space-y-2">
          <div className="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-amber-500/10 border border-amber-500/20 text-xs font-bold text-amber-400">
            <Vote className="w-3.5 h-3.5" />
            <span>ON-CHAIN GOVERNANCE & PROTOCOL STANDARDS</span>
          </div>
          <h1 className="text-2xl sm:text-4xl font-extrabold text-text-primary tracking-tight">
            Governance & Improvement Proposals
          </h1>
          <p className="text-xs sm:text-sm text-text-secondary max-w-2xl leading-relaxed">
            Vote on parameter updates, review SPRX Improvement Proposals (SIPs), and participate in ecosystem foundation grants.
          </p>
        </div>

        {/* Tab Controls */}
        <div className="pt-2 flex items-center gap-2 overflow-x-auto">
          <button
            onClick={() => setActiveTab("voting")}
            className={`px-3.5 py-1.5 rounded-xl text-xs font-bold transition-all shrink-0 ${
              activeTab === "voting"
                ? "bg-amber-500/20 text-amber-400 border border-amber-500/40"
                : "bg-bg-surface border border-border-subtle text-text-secondary hover:text-text-primary"
            }`}
          >
            Voting Proposals
          </button>
          <button
            onClick={() => setActiveTab("sips")}
            className={`px-3.5 py-1.5 rounded-xl text-xs font-bold transition-all shrink-0 ${
              activeTab === "sips"
                ? "bg-amber-500/20 text-amber-400 border border-amber-500/40"
                : "bg-bg-surface border border-border-subtle text-text-secondary hover:text-text-primary"
            }`}
          >
            SIP Proposals Registry
          </button>
          <button
            onClick={() => setActiveTab("grants")}
            className={`px-3.5 py-1.5 rounded-xl text-xs font-bold transition-all shrink-0 ${
              activeTab === "grants"
                ? "bg-amber-500/20 text-amber-400 border border-amber-500/40"
                : "bg-bg-surface border border-border-subtle text-text-secondary hover:text-text-primary"
            }`}
          >
            Foundation Grants
          </button>
          <button
            onClick={() => setActiveTab("bug-bounty")}
            className={`px-3.5 py-1.5 rounded-xl text-xs font-bold transition-all shrink-0 ${
              activeTab === "bug-bounty"
                ? "bg-amber-500/20 text-amber-400 border border-amber-500/40"
                : "bg-bg-surface border border-border-subtle text-text-secondary hover:text-text-primary"
            }`}
          >
            Bug Bounty Program
          </button>
        </div>
      </div>

      {/* 1. Voting Proposals Tab */}
      {activeTab === "voting" && (
        <div className="space-y-6">
          <div className="grid grid-cols-1 gap-4">
            {proposals.map((prop) => {
              const userVote = votedProposals[prop.id];

              return (
                <div
                  key={prop.id}
                  className="rounded-2xl border border-border-subtle bg-bg-surface p-6 space-y-4 shadow-sm"
                >
                  <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-2">
                    <div className="flex items-center gap-2">
                      <span className="text-xs font-bold px-2.5 py-0.5 rounded-full bg-amber-500/10 text-amber-400 border border-amber-500/20">
                        #{prop.id}
                      </span>
                      <h3 className="text-base sm:text-lg font-bold text-text-primary">
                        {prop.title}
                      </h3>
                    </div>

                    <span
                      className={`text-xs font-bold px-2.5 py-0.5 rounded-full self-start sm:self-auto border ${
                        prop.status === "Voting"
                          ? "bg-emerald-500/10 text-emerald-400 border-emerald-500/30"
                          : "bg-bg-surface-elevated text-text-muted border-border-subtle"
                      }`}
                    >
                      {prop.status}
                    </span>
                  </div>

                  <p className="text-xs sm:text-sm text-text-secondary leading-relaxed">
                    {prop.description}
                  </p>

                  {/* Voting Progress Bar */}
                  <div className="space-y-2 pt-2">
                    <div className="flex items-center justify-between text-xs font-semibold text-text-muted">
                      <span>Yes: {prop.votes.yes}%</span>
                      <span>No: {prop.votes.no}%</span>
                      <span>Veto: {prop.votes.noWithVeto}%</span>
                      <span>Abstain: {prop.votes.abstain}%</span>
                    </div>

                    <div className="h-2.5 w-full bg-bg-surface-elevated rounded-full overflow-hidden flex">
                      <div
                        style={{ width: `${prop.votes.yes}%` }}
                        className="bg-emerald-400"
                      />
                      <div
                        style={{ width: `${prop.votes.no}%` }}
                        className="bg-coral-400"
                      />
                      <div
                        style={{ width: `${prop.votes.noWithVeto}%` }}
                        className="bg-purple-400"
                      />
                      <div
                        style={{ width: `${prop.votes.abstain}%` }}
                        className="bg-slate-500"
                      />
                    </div>
                  </div>

                  {/* Vote Action Buttons */}
                  <div className="pt-3 border-t border-border-subtle flex flex-wrap items-center justify-between gap-3">
                    <div className="text-xs text-text-muted">
                      Voting Closes: <span className="text-text-primary font-mono-num">{prop.votingEndTime}</span>
                    </div>

                    {userVote ? (
                      <div className="text-xs font-bold text-emerald-400 flex items-center gap-1.5">
                        <CheckCircle2 className="w-4 h-4" />
                        <span>You voted: {userVote}</span>
                      </div>
                    ) : (
                      <div className="flex items-center gap-2">
                        <button
                          onClick={() => handleVote(prop.id, "Yes")}
                          className="px-3 py-1.5 rounded-lg bg-emerald-500/20 text-emerald-400 border border-emerald-500/30 text-xs font-bold hover:bg-emerald-500/30 transition-all"
                        >
                          Vote Yes
                        </button>
                        <button
                          onClick={() => handleVote(prop.id, "No")}
                          className="px-3 py-1.5 rounded-lg bg-coral-500/20 text-coral-400 border border-coral-500/30 text-xs font-bold hover:bg-coral-500/30 transition-all"
                        >
                          Vote No
                        </button>
                        <button
                          onClick={() => handleVote(prop.id, "NoWithVeto")}
                          className="px-3 py-1.5 rounded-lg bg-purple-500/20 text-purple-400 border border-purple-500/30 text-xs font-bold hover:bg-purple-500/30 transition-all"
                        >
                          No With Veto
                        </button>
                        <button
                          onClick={() => handleVote(prop.id, "Abstain")}
                          className="px-3 py-1.5 rounded-lg bg-bg-surface-elevated text-text-secondary border border-border-subtle text-xs font-medium hover:text-text-primary transition-all"
                        >
                          Abstain
                        </button>
                      </div>
                    )}
                  </div>
                </div>
              );
            })}
          </div>
        </div>
      )}

      {/* 2. SIP Proposals Registry Tab */}
      {activeTab === "sips" && (
        <div className="space-y-6">
          {/* SIP Filter Controls */}
          <div className="flex flex-wrap items-center justify-between gap-3">
            <div className="flex items-center gap-2">
              <span className="text-xs font-bold text-text-muted">Category:</span>
              {["All", "Consensus", "Execution", "Standards", "Core"].map((cat) => (
                <button
                  key={cat}
                  onClick={() => setSipFilterCategory(cat)}
                  className={`px-2.5 py-1 rounded-lg text-xs font-semibold transition-all ${
                    sipFilterCategory === cat
                      ? "bg-amber-500/20 text-amber-400 border border-amber-500/40"
                      : "bg-bg-surface border border-border-subtle text-text-secondary hover:text-text-primary"
                  }`}
                >
                  {cat}
                </button>
              ))}
            </div>

            <div className="flex items-center gap-2">
              <span className="text-xs font-bold text-text-muted">Status:</span>
              {["All", "Final", "Accepted", "Review", "Draft"].map((st) => (
                <button
                  key={st}
                  onClick={() => setSipFilterStatus(st)}
                  className={`px-2.5 py-1 rounded-lg text-xs font-semibold transition-all ${
                    sipFilterStatus === st
                      ? "bg-cyan-500/20 text-cyan-400 border border-cyan-500/40"
                      : "bg-bg-surface border border-border-subtle text-text-secondary hover:text-text-primary"
                  }`}
                >
                  {st}
                </button>
              ))}
            </div>
          </div>

          {/* SIP Cards List */}
          <div className="space-y-4">
            {filteredSips.map((sip) => (
              <div
                key={sip.id}
                onClick={() => setSelectedSip(sip)}
                className="group cursor-pointer rounded-2xl border border-border-subtle bg-bg-surface hover:bg-bg-surface-elevated hover:border-amber-500/40 p-5 transition-all space-y-3"
              >
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2">
                    <span className="text-xs font-mono font-bold px-2 py-0.5 rounded bg-amber-500/10 text-amber-400 border border-amber-500/20">
                      {sip.sipNumber}
                    </span>
                    <span className="text-xs font-semibold text-text-muted">
                      {sip.category}
                    </span>
                  </div>

                  <span
                    className={`text-[10px] font-bold px-2 py-0.5 rounded-full border ${
                      sip.status === "Final"
                        ? "bg-emerald-500/10 text-emerald-400 border-emerald-500/30"
                        : "bg-sky-500/10 text-sky-400 border-sky-500/30"
                    }`}
                  >
                    {sip.status}
                  </span>
                </div>

                <h3 className="text-base font-bold text-text-primary group-hover:text-amber-400 transition-colors">
                  {sip.title}
                </h3>
                <p className="text-xs text-text-secondary leading-relaxed">
                  {sip.summary}
                </p>

                <div className="pt-2 flex items-center justify-between text-xs text-text-muted border-t border-border-subtle">
                  <span>Author: {sip.author}</span>
                  <span className="font-bold text-amber-400 group-hover:underline flex items-center gap-1">
                    <span>Inspect Specification</span>
                    <ChevronRight className="w-3.5 h-3.5" />
                  </span>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* 3. Foundation Grants Tab */}
      {activeTab === "grants" && (
        <div className="rounded-3xl border border-border-subtle bg-bg-surface p-6 sm:p-10 space-y-6 max-w-4xl mx-auto shadow-card">
          <div className="space-y-2 border-b border-border-subtle pb-6">
            <span className="text-xs font-bold text-emerald-400 uppercase tracking-wider">
              Ecosystem Funding
            </span>
            <h2 className="text-2xl sm:text-3xl font-extrabold text-text-primary">
              SPRX Ecosystem Foundation Grants
            </h2>
            <p className="text-xs sm:text-sm text-text-secondary">
              Non-dilutive capital and technical mentorship for developers building open-source tooling, SDKs, and real-world dApps.
            </p>
          </div>

          <div className="grid grid-cols-1 sm:grid-cols-3 gap-4">
            <div className="rounded-xl border border-border-subtle bg-bg-surface-elevated p-4 space-y-2">
              <span className="text-xs font-bold text-sky-400">Tier 1: Ignition</span>
              <div className="text-lg font-bold text-text-primary font-mono-num">$5K – $15K</div>
              <p className="text-xs text-text-muted">For prototypes, developer libraries, and documentation tutorials.</p>
            </div>
            <div className="rounded-xl border border-border-subtle bg-bg-surface-elevated p-4 space-y-2">
              <span className="text-xs font-bold text-emerald-400">Tier 2: Growth</span>
              <div className="text-lg font-bold text-text-primary font-mono-num">$20K – $50K</div>
              <p className="text-xs text-text-muted">For deployed testnet dApps, security audits, and indexer integrations.</p>
            </div>
            <div className="rounded-xl border border-border-subtle bg-bg-surface-elevated p-4 space-y-2">
              <span className="text-xs font-bold text-purple-400">Tier 3: Strategic</span>
              <div className="text-lg font-bold text-text-primary font-mono-num">$75K – $250K</div>
              <p className="text-xs text-text-muted">For core infrastructure, RWA tokenization pilots, and institutional rails.</p>
            </div>
          </div>
        </div>
      )}

      {/* 4. Bug Bounty Tab */}
      {activeTab === "bug-bounty" && (
        <div className="rounded-3xl border border-border-subtle bg-bg-surface p-6 sm:p-10 space-y-6 max-w-4xl mx-auto shadow-card">
          <div className="space-y-2 border-b border-border-subtle pb-6">
            <span className="text-xs font-bold text-purple-400 uppercase tracking-wider">
              Responsible Disclosure
            </span>
            <h2 className="text-2xl sm:text-3xl font-extrabold text-text-primary">
              Security Bug Bounty Program
            </h2>
            <p className="text-xs sm:text-sm text-text-secondary">
              Rewards for identifying consensus, cryptographic, or smart contract execution vulnerabilities.
            </p>
          </div>

          <div className="space-y-3 text-xs sm:text-sm text-text-secondary leading-relaxed">
            <p>
              Submissions can be sent directly to <code className="text-cyan-400">security@sprax.network</code> using our official PGP key.
            </p>
            <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 pt-2">
              <div className="p-3.5 rounded-xl bg-bg-surface-elevated border border-border-subtle">
                <span className="text-xs font-bold text-coral-400 block">Critical Severity</span>
                <span className="text-sm font-bold text-text-primary">Up to 50,000 SPRX</span>
                <p className="text-xs text-text-muted mt-1">Consensus halting, state corruption, or RCE vulnerabilities.</p>
              </div>
              <div className="p-3.5 rounded-xl bg-bg-surface-elevated border border-border-subtle">
                <span className="text-xs font-bold text-amber-400 block">High Severity</span>
                <span className="text-sm font-bold text-text-primary">Up to 15,000 SPRX</span>
                <p className="text-xs text-text-muted mt-1">Validator jailing bypass or smart contract state tampering.</p>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Selected SIP Detail Modal */}
      {selectedSip && (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/80 backdrop-blur-sm animate-fadeIn">
          <div className="relative w-full max-w-2xl max-h-[85vh] overflow-y-auto rounded-2xl border border-border-prominent bg-bg-surface p-6 sm:p-8 shadow-2xl space-y-6">
            <button
              onClick={() => setSelectedSip(null)}
              className="absolute top-4 right-4 p-2 rounded-lg text-text-muted hover:text-text-primary hover:bg-bg-surface-elevated"
            >
              ✕
            </button>

            <div className="space-y-1">
              <span className="text-xs font-mono font-bold text-amber-400">
                {selectedSip.sipNumber} • {selectedSip.category}
              </span>
              <h2 className="text-xl sm:text-2xl font-extrabold text-text-primary">
                {selectedSip.title}
              </h2>
              <p className="text-xs text-text-muted">
                Author: {selectedSip.author} | Created: {selectedSip.createdDate}
              </p>
            </div>

            <div className="space-y-4 text-xs sm:text-sm text-text-secondary leading-relaxed border-t border-border-subtle pt-4">
              <div>
                <h4 className="font-bold text-text-primary uppercase text-xs mb-1">Summary:</h4>
                <p>{selectedSip.summary}</p>
              </div>

              <div>
                <h4 className="font-bold text-text-primary uppercase text-xs mb-1">Motivation:</h4>
                <p>{selectedSip.motivation}</p>
              </div>

              <div>
                <h4 className="font-bold text-text-primary uppercase text-xs mb-1">Specification:</h4>
                <p className="p-3 rounded-xl bg-bg-surface-elevated font-mono text-xs text-cyan-400">
                  {selectedSip.specification}
                </p>
              </div>

              <div>
                <h4 className="font-bold text-text-primary uppercase text-xs mb-1">Rationale:</h4>
                <p>{selectedSip.rationale}</p>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
