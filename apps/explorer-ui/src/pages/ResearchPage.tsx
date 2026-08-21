import React, { useState, useEffect } from "react";
import {
  BookOpen,
  Clock,
  ExternalLink,
} from "lucide-react";
import { ResearchPaper } from "@/types";
import { apiService } from "@/services/api";

interface ResearchPageProps {
  onNavigate: (route: string) => void;
}

export const ResearchPage: React.FC<ResearchPageProps> = ({ onNavigate }) => {
  const [papers, setPapers] = useState<ResearchPaper[]>([]);
  const [selectedCategory, setSelectedCategory] = useState<string>("All");

  useEffect(() => {
    const load = async () => {
      const list = await apiService.getResearchPapers();
      setPapers(list);
    };
    load();
  }, []);

  const categories = ["All", "Consensus", "Cryptography", "RWA Legal Rails", "State Storage"];

  const filtered = papers.filter(
    (p) => selectedCategory === "All" || p.category === selectedCategory
  );

  return (
    <div className="space-y-8 animate-fadeIn">
      {/* Header Banner */}
      <div className="rounded-3xl border border-border-prominent bg-gradient-to-r from-bg-surface via-bg-surface-elevated/40 to-bg-surface p-6 sm:p-10 space-y-4 shadow-card">
        <div className="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-cyan-500/10 border border-cyan-500/20 text-xs font-bold text-cyan-400">
          <BookOpen className="w-3.5 h-3.5" />
          <span>SPRX CRYPTOGRAPHIC & PROTOCOL RESEARCH</span>
        </div>
        <h1 className="text-2xl sm:text-4xl font-extrabold text-text-primary tracking-tight">
          Protocol Research Hub
        </h1>
        <p className="text-xs sm:text-sm text-text-secondary max-w-2xl leading-relaxed">
          Peer-reviewed mathematical analyses, consensus proofs, and legal rails governing sovereign real-world asset settlement.
        </p>

        {/* Category Filters */}
        <div className="pt-2 flex items-center gap-1.5 overflow-x-auto">
          {categories.map((cat) => (
            <button
              key={cat}
              onClick={() => setSelectedCategory(cat)}
              className={`px-3 py-1.5 rounded-xl text-xs font-bold transition-all shrink-0 ${
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

      {/* Papers List */}
      <div className="space-y-4">
        {filtered.map((paper) => (
          <div
            key={paper.id}
            className="rounded-2xl border border-border-subtle bg-bg-surface p-6 space-y-4 shadow-sm"
          >
            <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-2">
              <div className="flex items-center gap-2">
                <span className="text-xs font-bold px-2.5 py-0.5 rounded-full bg-cyan-500/10 text-cyan-400 border border-cyan-500/20">
                  {paper.category}
                </span>
                <span className="text-xs text-text-muted flex items-center gap-1">
                  <Clock className="w-3.5 h-3.5" />
                  <span>{paper.readingMinutes} min read</span>
                </span>
              </div>

              <span className="text-xs text-text-muted font-mono">{paper.date}</span>
            </div>

            <h3 className="text-base sm:text-xl font-bold text-text-primary">
              {paper.title}
            </h3>

            <p className="text-xs sm:text-sm text-text-secondary leading-relaxed">
              {paper.abstract}
            </p>

            <div className="pt-3 border-t border-border-subtle flex flex-wrap items-center justify-between gap-3 text-xs">
              <span className="text-text-muted">
                Authors: {paper.authors.join(", ")}
              </span>

              <button
                onClick={() => onNavigate("/whitepaper")}
                className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-cyan-500/10 text-cyan-400 border border-cyan-500/30 font-bold hover:bg-cyan-500/20 transition-all"
              >
                <span>Read Full Paper</span>
                <ExternalLink className="w-3.5 h-3.5" />
              </button>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
};
