import React from "react";

export const BrandKitPage: React.FC = () => {
  return (
    <div className="max-w-4xl mx-auto space-y-8 animate-fadeIn">
      <div className="rounded-3xl border border-border-prominent bg-bg-surface p-6 sm:p-10 space-y-4 shadow-card">
        <span className="text-xs font-bold text-cyan-400 uppercase tracking-wider">
          Media Assets & Brand Kit
        </span>
        <h1 className="text-2xl sm:text-4xl font-extrabold text-text-primary tracking-tight">
          SPRX Visual Identity
        </h1>
        <p className="text-xs sm:text-sm text-text-secondary leading-relaxed">
          Official logos, color tokens, and typography guidelines for SPRX (Scalable Protocol for Real-world X).
        </p>
      </div>

      <div className="grid grid-cols-1 sm:grid-cols-2 gap-6">
        {/* Logo Card */}
        <div className="rounded-2xl border border-border-subtle bg-bg-surface p-6 space-y-4 flex flex-col items-center justify-center text-center">
          <div className="relative flex items-center justify-center w-20 h-20 rounded-2xl bg-gradient-to-br from-cyan-400 via-sky-500 to-indigo-600 shadow-xl border border-cyan-400/40">
            <span className="font-black text-white text-3xl font-sans">X</span>
            <span className="absolute -top-1.5 -right-1.5 w-4 h-4 rounded-full bg-emerald-400 ring-2 ring-bg-surface" />
          </div>
          <div>
            <h3 className="text-base font-bold text-text-primary">SPRX Protocol Mark</h3>
            <p className="text-xs text-text-muted mt-1">SVG Vector & PNG Format</p>
          </div>
        </div>

        {/* Colors Card */}
        <div className="rounded-2xl border border-border-subtle bg-bg-surface p-6 space-y-4">
          <h3 className="text-sm font-bold text-text-primary uppercase tracking-wider">
            Brand Color Tokens
          </h3>
          <div className="space-y-2">
            <div className="flex items-center justify-between p-2 rounded-lg bg-[#080a0f] border border-border-subtle text-xs">
              <span className="text-slate-300 font-medium">Obsidian Void</span>
              <span className="font-mono text-cyan-400">#080A0F</span>
            </div>
            <div className="flex items-center justify-between p-2 rounded-lg bg-[#00f2fe]/10 border border-[#00f2fe]/30 text-xs">
              <span className="text-cyan-400 font-medium">Hyper Cyan (Primary)</span>
              <span className="font-mono text-cyan-400">#00F2FE</span>
            </div>
            <div className="flex items-center justify-between p-2 rounded-lg bg-[#10b981]/10 border border-[#10b981]/30 text-xs">
              <span className="text-emerald-400 font-medium">Signal Emerald</span>
              <span className="font-mono text-emerald-400">#10B981</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};
