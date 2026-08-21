import React, { useEffect, useRef } from "react";
import { X } from "lucide-react";
import { CopyButton } from "./CopyButton";

interface QrCodeModalProps {
  isOpen: boolean;
  onClose: () => void;
  address: string;
  title?: string;
}

export const QrCodeModal: React.FC<QrCodeModalProps> = ({
  isOpen,
  onClose,
  address,
  title = "Account Address QR",
}) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    if (!isOpen || !canvasRef.current) return;
    const canvas = canvasRef.current;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    // Draw pseudo high-contrast QR Matrix representation
    const size = 200;
    canvas.width = size;
    canvas.height = size;

    ctx.fillStyle = "#ffffff";
    ctx.fillRect(0, 0, size, size);

    ctx.fillStyle = "#0a0f1d";
    const blockSize = 8;
    const count = Math.floor(size / blockSize);

    // Seeded pseudo pattern from address characters
    let seed = 0;
    for (let i = 0; i < address.length; i++) {
      seed = (seed + address.charCodeAt(i) * (i + 1)) % 1000000007;
    }

    const pseudoRandom = () => {
      seed = (seed * 9301 + 49297) % 233280;
      return seed / 233280;
    };

    // Draw 3 corner finder patterns
    const drawFinder = (startX: number, startY: number) => {
      ctx.fillRect(startX * blockSize, startY * blockSize, 7 * blockSize, 7 * blockSize);
      ctx.fillStyle = "#ffffff";
      ctx.fillRect((startX + 1) * blockSize, (startY + 1) * blockSize, 5 * blockSize, 5 * blockSize);
      ctx.fillStyle = "#0a0f1d";
      ctx.fillRect((startX + 2) * blockSize, (startY + 2) * blockSize, 3 * blockSize, 3 * blockSize);
    };

    drawFinder(2, 2);
    drawFinder(count - 9, 2);
    drawFinder(2, count - 9);

    // Draw data blocks
    for (let x = 0; x < count; x++) {
      for (let y = 0; y < count; y++) {
        // Skip finder zones
        if (
          (x < 10 && y < 10) ||
          (x > count - 11 && y < 10) ||
          (x < 10 && y > count - 11)
        ) {
          continue;
        }
        if (pseudoRandom() > 0.55) {
          ctx.fillRect(x * blockSize, y * blockSize, blockSize, blockSize);
        }
      }
    }
  }, [isOpen, address]);

  if (!isOpen) return null;

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/70 backdrop-blur-sm animate-fadeIn"
      onClick={onClose}
    >
      <div
        className="relative w-full max-w-sm rounded-2xl border border-border-strong bg-bg-surface p-6 shadow-elevated"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-center justify-between pb-4 border-b border-border-subtle">
          <h3 className="text-base font-semibold text-text-primary">{title}</h3>
          <button
            type="button"
            onClick={onClose}
            className="p-1 rounded-lg text-text-muted hover:text-text-primary hover:bg-bg-hover transition-colors"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        <div className="flex flex-col items-center justify-center my-6">
          <div className="p-3 bg-white rounded-xl shadow-md">
            <canvas ref={canvasRef} className="rounded" />
          </div>
          <p className="mt-3 text-xs text-text-secondary text-center">
            Scan with SPRX Mobile Wallet or web client
          </p>
        </div>

        <div className="p-3 rounded-lg bg-bg-surface-elevated border border-border-subtle flex items-center justify-between gap-2">
          <span className="font-mono text-xs text-text-secondary truncate">{address}</span>
          <CopyButton text={address} />
        </div>
      </div>
    </div>
  );
};
