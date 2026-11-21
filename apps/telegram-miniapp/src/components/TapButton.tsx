"use client";

import { useState, useRef, useCallback, MouseEvent, TouchEvent } from "react";
import { Zap } from "lucide-react";
import { hapticTap } from "@/services/telegram";

interface FloatingPoint {
  id: number;
  x: number;
  y: number;
  value: number;
}

interface TapButtonProps {
  disabled: boolean;
  onTap: () => void;
  pointsPerTap?: number;
}

export function TapButton({ disabled, onTap, pointsPerTap = 1 }: TapButtonProps) {
  const [floatingPoints, setFloatingPoints] = useState<FloatingPoint[]>([]);
  const [tilt, setTilt] = useState({ x: 0, y: 0, scale: 1 });
  const buttonRef = useRef<HTMLDivElement>(null);
  const nextId = useRef(0);

  const addFloatingPoint = (clientX: number, clientY: number) => {
    if (!buttonRef.current) return;
    const rect = buttonRef.current.getBoundingClientRect();
    const x = clientX - rect.left;
    const y = clientY - rect.top;
    const id = nextId.current++;

    setFloatingPoints((prev) => [...prev, { id, x, y, value: pointsPerTap }]);
    setTimeout(() => {
      setFloatingPoints((prev) => prev.filter((p) => p.id !== id));
    }, 800);
  };

  const handlePointerDown = (clientX: number, clientY: number) => {
    if (disabled || !buttonRef.current) return;
    const rect = buttonRef.current.getBoundingClientRect();
    const centerX = rect.left + rect.width / 2;
    const centerY = rect.top + rect.height / 2;

    const deltaX = (clientX - centerX) / (rect.width / 2);
    const deltaY = (clientY - centerY) / (rect.height / 2);

    // Dynamic 3D tilt calculation
    setTilt({
      x: -deltaY * 18,
      y: deltaX * 18,
      scale: 0.94,
    });

    addFloatingPoint(clientX, clientY);
    hapticTap("medium");
    onTap();
  };

  const handleTouchStart = (e: TouchEvent<HTMLDivElement>) => {
    if (disabled) return;
    // Support multi-touch
    for (let i = 0; i < e.changedTouches.length; i++) {
      const touch = e.changedTouches[i];
      handlePointerDown(touch.clientX, touch.clientY);
    }
  };

  const handleMouseDown = (e: MouseEvent<HTMLDivElement>) => {
    if (disabled) return;
    handlePointerDown(e.clientX, e.clientY);
  };

  const handlePointerUp = useCallback(() => {
    setTilt({ x: 0, y: 0, scale: 1 });
  }, []);

  return (
    <div className="relative flex items-center justify-center py-4">
      {/* Outer ambient glow pulse */}
      <div className="absolute h-64 w-64 rounded-full bg-gradient-to-tr from-sprx-500/20 via-sprx-400/30 to-cyber-neon/20 blur-2xl animate-pulse-slow pointer-events-none" />

      {/* Floating numbers container */}
      <div className="pointer-events-none absolute inset-0 z-30 overflow-visible">
        {floatingPoints.map((p) => (
          <span
            key={p.id}
            className="absolute select-none text-2xl sm:text-3xl font-black text-cyan-300 drop-shadow-[0_0_12px_rgba(56,189,248,0.9)] animate-float-up"
            style={{ left: `${p.x}px`, top: `${p.y}px` }}
          >
            +{p.value}
          </span>
        ))}
      </div>

      {/* 3D Coin Button */}
      <div
        ref={buttonRef}
        onMouseDown={handleMouseDown}
        onMouseUp={handlePointerUp}
        onMouseLeave={handlePointerUp}
        onTouchStart={handleTouchStart}
        onTouchEnd={handlePointerUp}
        onTouchCancel={handlePointerUp}
        style={{
          transform: `perspective(1000px) rotateX(${tilt.x}deg) rotateY(${tilt.y}deg) scale(${tilt.scale})`,
          transition: "transform 0.12s cubic-bezier(0.2, 0.8, 0.4, 1)",
        }}
        className={`relative flex h-60 w-60 sm:h-64 sm:w-64 cursor-pointer select-none items-center justify-center rounded-full p-3 transition-opacity ${
          disabled ? "opacity-40 cursor-not-allowed" : "active:cursor-grabbing"
        }`}
      >
        {/* Outer Ring */}
        <div className="absolute inset-0 rounded-full border-2 border-sprx-400/30 bg-gradient-to-b from-white/10 to-transparent shadow-[0_0_40px_rgba(14,165,233,0.35)]" />

        {/* Mid Ring with metallic gloss */}
        <div className="absolute inset-2 rounded-full border border-sprx-300/40 bg-gradient-to-b from-slate-900 via-sky-950 to-slate-900 shadow-inner" />

        {/* Inner Coin Core */}
        <div className="relative flex h-full w-full items-center justify-center rounded-full bg-gradient-to-br from-sprx-400 via-sprx-600 to-slate-950 p-2 shadow-[inset_0_4px_16px_rgba(255,255,255,0.4),0_8px_32px_rgba(0,0,0,0.6)]">
          {/* Subtle concentric decorative ring */}
          <div className="flex h-full w-full items-center justify-center rounded-full border border-white/20 bg-gradient-to-b from-sprx-500 to-sprx-700 shadow-inner">
            <div className="flex h-36 w-36 sm:h-40 sm:w-40 items-center justify-center rounded-full border border-white/30 bg-gradient-to-br from-white/20 via-transparent to-black/30 backdrop-blur-sm shadow-[0_0_25px_rgba(255,255,255,0.3)]">
              <Zap
                className="h-20 w-20 sm:h-24 sm:w-24 text-white drop-shadow-[0_0_20px_rgba(255,255,255,0.8)] filter transition-transform"
                strokeWidth={2.5}
                fill="url(#zapGradient)"
              />
              <svg width="0" height="0">
                <linearGradient id="zapGradient" x1="0%" y1="0%" x2="100%" y2="100%">
                  <stop offset="0%" stopColor="#ffffff" />
                  <stop offset="60%" stopColor="#bae6fd" />
                  <stop offset="100%" stopColor="#38bdf8" />
                </linearGradient>
              </svg>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
