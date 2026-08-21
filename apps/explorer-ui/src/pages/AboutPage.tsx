import React from "react";

export const AboutPage: React.FC = () => {
  return (
    <div className="max-w-4xl mx-auto space-y-8 animate-fadeIn">
      <div className="rounded-3xl border border-border-prominent bg-bg-surface p-6 sm:p-10 space-y-4 shadow-card">
        <span className="text-xs font-bold text-cyan-400 uppercase tracking-wider">
          About SPRX
        </span>
        <h1 className="text-2xl sm:text-4xl font-extrabold text-text-primary tracking-tight">
          Scalable Protocol for Real-world X
        </h1>
        <p className="text-xs sm:text-sm text-text-secondary leading-relaxed">
          SPRX is built by an open collective of cryptographers, distributed systems engineers, and Web3 frontend architects dedicated to making blockchain infrastructure practical for real-world economic activity.
        </p>
      </div>

      <div className="rounded-3xl border border-border-subtle bg-bg-surface p-6 sm:p-10 space-y-6 text-xs sm:text-sm text-text-secondary leading-relaxed shadow-sm">
        <h2 className="text-xl font-bold text-text-primary">Our Core Mission</h2>
        <p>
          We believe blockchain technology achieves its true potential when it seamlessly connects with real-world payments, tangible asset ownership, verifiable identity, and physical infrastructure.
        </p>
        <div className="pt-4 border-t border-border-subtle text-xs text-text-muted">
          All core crates in the SPRX Protocol are open-source under Apache-2.0 / MIT licenses.
        </div>
      </div>
    </div>
  );
};
