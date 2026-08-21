import React from "react";
import { AlertCircle, Home, Search } from "lucide-react";

interface NotFoundPageProps {
  onGoHome: () => void;
  onOpenSearch: () => void;
  message?: string;
}

export const NotFoundPage: React.FC<NotFoundPageProps> = ({
  onGoHome,
  onOpenSearch,
  message = "The requested block, transaction, address, or page could not be found on Sprax Chain.",
}) => {
  return (
    <div className="flex flex-col items-center justify-center min-h-[50vh] text-center p-6 animate-fadeIn">
      <div className="flex items-center justify-center w-16 h-16 rounded-2xl bg-sky-500/10 text-sky-400 border border-sky-500/20 mb-6">
        <AlertCircle className="w-8 h-8" />
      </div>

      <h1 className="text-2xl sm:text-3xl font-extrabold text-text-primary mb-2">
        Record Not Found
      </h1>

      <p className="text-xs sm:text-sm text-text-secondary max-w-md mb-8 leading-relaxed">
        {message}
      </p>

      <div className="flex flex-wrap items-center justify-center gap-3">
        <button
          type="button"
          onClick={onGoHome}
          className="inline-flex items-center gap-2 px-4 py-2.5 rounded-xl bg-sky-600 hover:bg-sky-500 text-white text-xs sm:text-sm font-semibold transition-all shadow-sm"
        >
          <Home className="w-4 h-4" />
          <span>Back to Home</span>
        </button>

        <button
          type="button"
          onClick={onOpenSearch}
          className="inline-flex items-center gap-2 px-4 py-2.5 rounded-xl border border-border-subtle bg-bg-surface hover:bg-bg-hover text-text-primary text-xs sm:text-sm font-semibold transition-colors"
        >
          <Search className="w-4 h-4 text-sky-400" />
          <span>Universal Search</span>
        </button>
      </div>
    </div>
  );
};
