import React from "react";
import { truncateAddress, truncateHash } from "@/utils/formatters";
import { CopyButton } from "./CopyButton";

interface LinkBaseProps {
  onClick?: () => void;
  showCopy?: boolean;
  className?: string;
  truncate?: boolean;
}

export const HashLink: React.FC<
  LinkBaseProps & { hash: string; start?: number; end?: number }
> = ({ hash, onClick, showCopy = false, className = "", truncate = true, start = 6, end = 6 }) => {
  const display = truncate ? truncateHash(hash, start, end) : hash;

  return (
    <span className={`inline-flex items-center gap-1 font-mono text-xs sm:text-sm ${className}`}>
      <button
        type="button"
        onClick={onClick}
        className="text-sky-400 hover:text-sky-300 transition-colors font-medium hover:underline text-left"
        title={hash}
      >
        {display}
      </button>
      {showCopy && <CopyButton text={hash} />}
    </span>
  );
};

export const TxLink: React.FC<
  LinkBaseProps & { hash: string; start?: number; end?: number }
> = ({ hash, onClick, showCopy = false, className = "", truncate = true, start = 8, end = 6 }) => {
  return (
    <HashLink
      hash={hash}
      onClick={onClick}
      showCopy={showCopy}
      className={className}
      truncate={truncate}
      start={start}
      end={end}
    />
  );
};

export const AddressLink: React.FC<
  LinkBaseProps & { address: string; start?: number; end?: number; label?: string }
> = ({
  address,
  onClick,
  showCopy = false,
  className = "",
  truncate = true,
  start = 8,
  end = 6,
  label,
}) => {
  const display = label || (truncate ? truncateAddress(address, start, end) : address);

  return (
    <span className={`inline-flex items-center gap-1 font-mono text-xs sm:text-sm ${className}`}>
      <button
        type="button"
        onClick={onClick}
        className="text-sky-400 hover:text-sky-300 transition-colors font-medium hover:underline text-left"
        title={address}
      >
        {display}
      </button>
      {showCopy && <CopyButton text={address} />}
    </span>
  );
};

export const BlockLink: React.FC<LinkBaseProps & { height: number }> = ({
  height,
  onClick,
  className = "",
}) => {
  return (
    <button
      type="button"
      onClick={onClick}
      className={`font-mono text-sky-400 hover:text-sky-300 font-semibold hover:underline text-xs sm:text-sm ${className}`}
    >
      #{height.toLocaleString()}
    </button>
  );
};

export const ValidatorLink: React.FC<
  LinkBaseProps & { address: string; moniker?: string }
> = ({ address, moniker, onClick, showCopy = false, className = "" }) => {
  return (
    <span className={`inline-flex items-center gap-1.5 ${className}`}>
      <button
        type="button"
        onClick={onClick}
        className="text-left group"
        title={address}
      >
        {moniker && (
          <span className="font-medium text-text-primary group-hover:text-sky-400 transition-colors block text-sm">
            {moniker}
          </span>
        )}
        <span className="font-mono text-xs text-text-secondary group-hover:text-sky-300 block">
          {truncateAddress(address, 8, 6)}
        </span>
      </button>
      {showCopy && <CopyButton text={address} />}
    </span>
  );
};
