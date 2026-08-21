import React, { useState } from "react";
import { Check, Copy } from "lucide-react";
import { copyToClipboard } from "@/utils/formatters";

interface CopyButtonProps {
  text: string;
  className?: string;
  title?: string;
}

export const CopyButton: React.FC<CopyButtonProps> = ({
  text,
  className = "",
  title = "Copy to clipboard",
}) => {
  const [copied, setCopied] = useState(false);

  const handleCopy = async (e: React.MouseEvent) => {
    e.stopPropagation();
    const ok = await copyToClipboard(text);
    if (ok) {
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  };

  return (
    <button
      type="button"
      onClick={handleCopy}
      title={copied ? "Copied!" : title}
      aria-label={copied ? "Copied!" : title}
      className={`inline-flex items-center justify-center p-1 rounded-md text-text-muted hover:text-text-primary hover:bg-bg-hover transition-colors ${className}`}
    >
      {copied ? (
        <Check className="w-3.5 h-3.5 text-emerald-400" />
      ) : (
        <Copy className="w-3.5 h-3.5" />
      )}
    </button>
  );
};
