import { Component, ErrorInfo, ReactNode } from "react";
import { AlertTriangle, RefreshCw } from "lucide-react";

interface Props {
  children: ReactNode;
}

interface State {
  hasError: boolean;
  error?: Error;
}

export class ErrorBoundary extends Component<Props, State> {
  public state: State = {
    hasError: false,
  };

  public static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error };
  }

  public componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    console.error("Uncaught error:", error, errorInfo);
  }

  public render() {
    if (this.state.hasError) {
      return (
        <div className="flex flex-col items-center justify-center min-h-[60vh] p-6 text-center">
          <div className="flex items-center justify-center w-16 h-16 rounded-2xl bg-rose-500/10 text-rose-400 border border-rose-500/20 mb-6">
            <AlertTriangle className="w-8 h-8" />
          </div>
          <h2 className="text-xl sm:text-2xl font-bold text-text-primary mb-2">
            Something went wrong
          </h2>
          <p className="text-xs sm:text-sm text-text-secondary max-w-md mb-6 font-mono-num">
            {this.state.error?.message || "An unexpected error occurred in the application."}
          </p>
          <button
            type="button"
            onClick={() => window.location.reload()}
            className="inline-flex items-center gap-2 px-4 py-2.5 rounded-xl bg-sky-600 hover:bg-sky-500 text-white text-xs sm:text-sm font-semibold transition-all"
          >
            <RefreshCw className="w-4 h-4" />
            <span>Reload Application</span>
          </button>
        </div>
      );
    }

    return this.props.children;
  }
}
