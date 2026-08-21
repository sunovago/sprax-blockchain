import { Currency } from "@/types";

export const FIAT_RATES: Record<Currency, { symbol: string; prefix: string; rate: number }> = {
  USD: { symbol: "USD", prefix: "$", rate: 1.25 },
  INR: { symbol: "INR", prefix: "₹", rate: 104.50 },
  EUR: { symbol: "EUR", prefix: "€", rate: 1.15 },
  GBP: { symbol: "GBP", prefix: "£", rate: 0.98 },
  JPY: { symbol: "JPY", prefix: "¥", rate: 195.00 },
};

/**
 * Truncates 32-byte hex hash to 0x1234...5678 format
 */
export function truncateHash(hash: string | undefined | null, start = 6, end = 6): string {
  if (!hash) return "";
  if (hash.length <= start + end + 3) return hash;
  return `${hash.slice(0, start)}...${hash.slice(-end)}`;
}

/**
 * Truncates Bech32 address (sprax1...) or hex address
 */
export function truncateAddress(address: string | undefined | null, start = 8, end = 6): string {
  if (!address) return "";
  if (address.length <= start + end + 3) return address;
  return `${address.slice(0, start)}...${address.slice(-end)}`;
}

/**
 * Formats amount with thousand separators and decimal trim
 */
export function formatAmount(val: string | number | undefined | null, maxDecimals = 4): string {
  if (val === undefined || val === null || val === "") return "0";
  const num = typeof val === "string" ? parseFloat(val.replace(/,/g, "")) : val;
  if (isNaN(num)) return String(val);
  return num.toLocaleString("en-US", {
    minimumFractionDigits: 0,
    maximumFractionDigits: maxDecimals,
  });
}

/**
 * Formats SPRX amount to fiat estimate with currency prefix
 */
export function formatFiat(
  sprxAmount: string | number | undefined | null,
  currency: Currency = "USD"
): string {
  if (sprxAmount === undefined || sprxAmount === null || sprxAmount === "") return "-";
  const num = typeof sprxAmount === "string" ? parseFloat(sprxAmount.replace(/,/g, "").replace(/ SPRX/i, "")) : sprxAmount;
  if (isNaN(num)) return "-";
  const rateConfig = FIAT_RATES[currency] || FIAT_RATES.USD;
  const fiatVal = num * rateConfig.rate;
  return `${rateConfig.prefix}${fiatVal.toLocaleString("en-US", {
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  })}`;
}

/**
 * Formats timestamp to human readable UTC / local date string
 */
export function formatTimestamp(unixSecs: number): string {
  if (!unixSecs) return "-";
  const date = new Date(unixSecs * 1000);
  return date.toLocaleString("en-US", {
    year: "numeric",
    month: "short",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
    hour12: false,
    timeZoneName: "short",
  });
}

/**
 * Formats relative time elapsed (e.g. 5s ago, 2m ago, 1h ago)
 */
export function formatTimeAgo(unixSecs: number): string {
  if (!unixSecs) return "-";
  const nowSecs = Math.floor(Date.now() / 1000);
  const diffSecs = Math.max(0, nowSecs - unixSecs);

  if (diffSecs < 5) return "just now";
  if (diffSecs < 60) return `${diffSecs}s ago`;
  const mins = Math.floor(diffSecs / 60);
  if (mins < 60) return `${mins}m ago`;
  const hours = Math.floor(mins / 60);
  if (hours < 24) return `${hours}h ago`;
  const days = Math.floor(hours / 24);
  return `${days}d ago`;
}

/**
 * Formats byte size
 */
export function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(2))} ${sizes[i]}`;
}

/**
 * Formats number with commas
 */
export function formatNumber(num: number | undefined | null): string {
  if (num === undefined || num === null) return "0";
  return num.toLocaleString("en-US");
}

/**
 * Determines search query intent
 */
export function parseSearchQuery(query: string): {
  type: "height" | "tx_hash" | "block_hash" | "address" | "validator" | "unknown";
  value: string;
} {
  const clean = query.trim();
  if (!clean) return { type: "unknown", value: "" };

  // Integer block height
  if (/^\d+$/.test(clean)) {
    return { type: "height", value: clean };
  }

  // 64-hex char hash (0x... or plain 64 hex chars)
  const hexMatch = clean.startsWith("0x") ? clean.slice(2) : clean;
  if (/^[0-9a-fA-F]{64}$/.test(hexMatch)) {
    return { type: "tx_hash", value: clean.startsWith("0x") ? clean : `0x${clean}` };
  }

  // Bech32 Address (sprax1...)
  if (clean.toLowerCase().startsWith("sprax1") && clean.length >= 38) {
    return { type: "address", value: clean };
  }

  // 20-byte Hex address (0x...)
  if (/^0x[0-9a-fA-F]{40}$/.test(clean)) {
    return { type: "address", value: clean };
  }

  // Moniker / string
  if (clean.length > 2) {
    return { type: "validator", value: clean };
  }

  return { type: "unknown", value: clean };
}

/**
 * Clipboard copy helper
 */
export async function copyToClipboard(text: string): Promise<boolean> {
  try {
    if (navigator?.clipboard?.writeText) {
      await navigator.clipboard.writeText(text);
      return true;
    }
    const textArea = document.createElement("textarea");
    textArea.value = text;
    textArea.style.position = "fixed";
    textArea.style.left = "-999999px";
    document.body.appendChild(textArea);
    textArea.focus();
    textArea.select();
    const success = document.execCommand("copy");
    document.body.removeChild(textArea);
    return success;
  } catch {
    return false;
  }
}
