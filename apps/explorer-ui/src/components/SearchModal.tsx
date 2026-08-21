import React, { useEffect, useState } from "react";
import { Search, X, History, ArrowRight, CornerDownLeft, Box, ArrowLeftRight, UserCheck, ShieldCheck } from "lucide-react";
import { parseSearchQuery } from "@/utils/formatters";

interface SearchModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSelectResult: (query: string) => void;
}

export const SearchModal: React.FC<SearchModalProps> = ({
  isOpen,
  onClose,
  onSelectResult,
}) => {
  const [query, setQuery] = useState("");
  const [recents, setRecents] = useState<string[]>([]);

  useEffect(() => {
    try {
      const stored = localStorage.getItem("sprx_recent_searches");
      if (stored) {
        setRecents(JSON.parse(stored).slice(0, 5));
      }
    } catch {
      // Ignore
    }
  }, [isOpen]);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "k") {
        e.preventDefault();
        if (isOpen) onClose();
        else {
          // Open handled outside or via trigger
        }
      }
      if (e.key === "Escape" && isOpen) {
        onClose();
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [isOpen, onClose]);

  if (!isOpen) return null;

  const handleExecute = (targetQuery: string) => {
    const clean = targetQuery.trim();
    if (!clean) return;

    try {
      const updated = [clean, ...recents.filter((r) => r !== clean)].slice(0, 5);
      localStorage.setItem("sprx_recent_searches", JSON.stringify(updated));
      setRecents(updated);
    } catch {
      // Ignore
    }

    onSelectResult(clean);
    onClose();
  };

  const parsed = parseSearchQuery(query);

  return (
    <div
      className="fixed inset-0 z-50 flex items-start justify-center pt-16 sm:pt-24 p-4 bg-black/75 backdrop-blur-md animate-fadeIn"
      onClick={onClose}
    >
      <div
        className="relative w-full max-w-xl rounded-2xl border border-border-strong bg-bg-surface shadow-2xl overflow-hidden animate-scaleUp"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Search header */}
        <div className="flex items-center px-4 py-3.5 border-b border-border-subtle bg-bg-surface-elevated/50">
          <Search className="w-5 h-5 text-sky-400 mr-3 shrink-0" />
          <input
            type="text"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter" && query.trim()) {
                handleExecute(query);
              }
            }}
            autoFocus
            placeholder="Search by address, tx hash, block number, or validator..."
            className="flex-1 bg-transparent text-sm sm:text-base text-text-primary placeholder:text-text-muted focus:outline-none"
          />
          {query ? (
            <button
              type="button"
              onClick={() => setQuery("")}
              className="p-1 rounded text-text-muted hover:text-text-primary"
            >
              <X className="w-4 h-4" />
            </button>
          ) : (
            <kbd className="hidden sm:inline-block px-2 py-0.5 text-[11px] font-mono font-semibold text-text-muted bg-bg-primary rounded border border-border-subtle">
              ESC
            </kbd>
          )}
        </div>

        {/* Live Detected Preview */}
        {query.trim() && (
          <div className="p-3 border-b border-border-subtle bg-sky-500/5">
            <button
              type="button"
              onClick={() => handleExecute(query)}
              className="w-full flex items-center justify-between p-2.5 rounded-lg hover:bg-sky-500/10 text-left transition-colors group"
            >
              <div className="flex items-center gap-3">
                <div className="p-2 rounded-lg bg-sky-500/20 text-sky-400">
                  {parsed.type === "height" ? (
                    <Box className="w-4 h-4" />
                  ) : parsed.type === "tx_hash" ? (
                    <ArrowLeftRight className="w-4 h-4" />
                  ) : parsed.type === "address" ? (
                    <UserCheck className="w-4 h-4" />
                  ) : (
                    <ShieldCheck className="w-4 h-4" />
                  )}
                </div>
                <div>
                  <div className="text-xs font-semibold text-sky-400 uppercase tracking-wider">
                    {parsed.type === "height"
                      ? "Search Block Height"
                      : parsed.type === "tx_hash"
                      ? "Search Transaction Hash"
                      : parsed.type === "address"
                      ? "Search Account Address"
                      : parsed.type === "validator"
                      ? "Search Validator Moniker"
                      : "Universal Search"}
                  </div>
                  <div className="text-sm font-mono text-text-primary truncate max-w-md">
                    {parsed.value}
                  </div>
                </div>
              </div>
              <CornerDownLeft className="w-4 h-4 text-text-muted group-hover:text-sky-400" />
            </button>
          </div>
        )}

        {/* Recent Searches */}
        {recents.length > 0 && !query.trim() && (
          <div className="p-4">
            <div className="flex items-center justify-between text-xs font-semibold uppercase tracking-wider text-text-secondary mb-2">
              <span className="flex items-center gap-1.5">
                <History className="w-3.5 h-3.5 text-text-muted" />
                Recent Searches
              </span>
              <button
                type="button"
                onClick={() => {
                  localStorage.removeItem("sprx_recent_searches");
                  setRecents([]);
                }}
                className="text-[11px] text-text-muted hover:text-rose-400 font-normal lowercase"
              >
                Clear
              </button>
            </div>
            <div className="space-y-1">
              {recents.map((item, idx) => (
                <button
                  key={idx}
                  type="button"
                  onClick={() => handleExecute(item)}
                  className="w-full flex items-center justify-between px-3 py-2 rounded-lg hover:bg-bg-surface-elevated text-left text-xs sm:text-sm font-mono text-text-secondary hover:text-text-primary transition-colors"
                >
                  <span className="truncate">{item}</span>
                  <ArrowRight className="w-3.5 h-3.5 opacity-40 shrink-0 ml-2" />
                </button>
              ))}
            </div>
          </div>
        )}

        {/* Search quick hints */}
        <div className="px-4 py-3 bg-bg-surface-elevated/40 border-t border-border-subtle flex flex-wrap items-center justify-between gap-2 text-[11px] text-text-muted">
          <div className="flex items-center gap-3">
            <span>Examples:</span>
            <span
              onClick={() => handleExecute("8245920")}
              className="cursor-pointer hover:text-sky-400 font-mono"
            >
              #8245920
            </span>
            <span
              onClick={() => handleExecute("sprax1qpzry9x8gf2tvdw0s3jn54khce6mua7l4w3e2r")}
              className="cursor-pointer hover:text-sky-400 font-mono"
            >
              sprax1qpz...
            </span>
          </div>
          <div className="hidden sm:flex items-center gap-1.5 font-mono">
            <span>Press</span>
            <kbd className="px-1.5 py-0.5 rounded bg-bg-primary border border-border-subtle text-text-secondary">
              Enter
            </kbd>
            <span>to select</span>
          </div>
        </div>
      </div>
    </div>
  );
};
