import React, { useEffect, useState } from "react";
import { Search, Sparkles, ArrowRight } from "lucide-react";
import { parseSearchQuery } from "@/utils/formatters";

interface SearchBarProps {
  onSearch: (query: string) => void;
  placeholder?: string;
  size?: "md" | "lg";
  className?: string;
  autoFocus?: boolean;
}

export const SearchBar: React.FC<SearchBarProps> = ({
  onSearch,
  placeholder = "Search by address / transaction / block / validator",
  size = "md",
  className = "",
  autoFocus = false,
}) => {
  const [query, setQuery] = useState("");
  const [detectedType, setDetectedType] = useState<string | null>(null);

  useEffect(() => {
    if (!query.trim()) {
      setDetectedType(null);
      return;
    }
    const parsed = parseSearchQuery(query);
    if (parsed.type === "height") setDetectedType("Block Height");
    else if (parsed.type === "tx_hash") setDetectedType("Tx Hash");
    else if (parsed.type === "address") setDetectedType("Address");
    else if (parsed.type === "validator") setDetectedType("Validator");
    else setDetectedType(null);
  }, [query]);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (query.trim()) {
      onSearch(query.trim());
    }
  };

  const isLg = size === "lg";

  return (
    <form onSubmit={handleSubmit} className={`w-full ${className}`}>
      <div
        className={`relative flex items-center w-full rounded-xl border border-border-subtle bg-bg-surface transition-all duration-200 focus-within:border-sky-500/80 focus-within:ring-2 focus-within:ring-sky-500/20 shadow-card ${
          isLg ? "p-1.5 sm:p-2" : "p-1"
        }`}
      >
        <div className="flex items-center pl-3 pr-2 text-text-muted">
          <Search className={`${isLg ? "w-5 h-5" : "w-4 h-4"} text-sky-400`} />
        </div>

        <input
          type="text"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder={placeholder}
          autoFocus={autoFocus}
          className={`flex-1 bg-transparent text-text-primary placeholder:text-text-muted focus:outline-none font-sans text-xs sm:text-sm ${
            isLg ? "py-2 sm:py-3" : "py-1.5"
          }`}
        />

        {detectedType && (
          <span className="hidden sm:inline-flex items-center gap-1 px-2 py-0.5 mr-2 rounded text-[11px] font-medium font-mono bg-sky-500/10 text-sky-400 border border-sky-500/20">
            <Sparkles className="w-3 h-3" />
            {detectedType}
          </span>
        )}

        <button
          type="submit"
          className={`inline-flex items-center justify-center gap-1.5 rounded-lg bg-sky-600 hover:bg-sky-500 active:scale-[0.98] text-white font-medium text-xs sm:text-sm transition-all ${
            isLg ? "px-4 sm:px-5 py-2 sm:py-2.5" : "px-3 py-1.5"
          }`}
        >
          <span>Search</span>
          <ArrowRight className="w-3.5 h-3.5" />
        </button>
      </div>
    </form>
  );
};
