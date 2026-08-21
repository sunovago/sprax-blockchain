import React, { useState } from 'react';
import { Shield, Lock, User, ArrowRight, AlertCircle } from 'lucide-react';
import { useAuth } from '../context/AuthContext';

export const LoginPage: React.FC = () => {
  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const { login } = useAuth();

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    setIsSubmitting(true);
    try {
      await login(username, password);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Invalid admin credentials');
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <div className="min-h-screen w-screen bg-[#0B0F19] flex items-center justify-center p-4">
      {/* Background Glow */}
      <div className="absolute w-96 h-96 bg-primary-500/10 rounded-full blur-3xl pointer-events-none" />

      <div className="bg-[#111827] border border-[#1E293B] rounded-3xl max-w-md w-full p-8 shadow-2xl relative z-10 space-y-6">
        {/* Header */}
        <div className="text-center space-y-2">
          <div className="w-12 h-12 rounded-2xl bg-primary-500/10 border border-primary-500/30 flex items-center justify-center text-primary-400 font-black text-xl mx-auto shadow-lg shadow-cyan-500/20">
            S
          </div>
          <h1 className="text-xl font-extrabold text-white tracking-tight">SPRX Admin Control</h1>
          <p className="text-xs text-gray-400">
            Centralized operations & monitoring center for Sprax Chain.
          </p>
        </div>

        {error && (
          <div className="p-3.5 rounded-xl bg-rose-950/60 border border-rose-800 text-xs text-rose-400 flex items-center gap-2">
            <AlertCircle className="w-4 h-4 shrink-0" />
            {error}
          </div>
        )}

        <form onSubmit={handleSubmit} className="space-y-4">
          <div>
            <label className="block text-xs font-semibold text-gray-300 mb-1.5">Username</label>
            <div className="relative">
              <User className="w-4 h-4 absolute left-3 top-1/2 -translate-y-1/2 text-gray-500" />
              <input
                type="text"
                required
                placeholder="admin"
                value={username}
                onChange={(e) => setUsername(e.target.value)}
                className="w-full pl-9 pr-4 py-2.5 bg-[#0B0F19] border border-[#1E293B] rounded-xl text-xs text-white placeholder-gray-600 focus:outline-none focus:border-primary-500 transition-colors font-medium"
              />
            </div>
          </div>

          <div>
            <label className="block text-xs font-semibold text-gray-300 mb-1.5">Password</label>
            <div className="relative">
              <Lock className="w-4 h-4 absolute left-3 top-1/2 -translate-y-1/2 text-gray-500" />
              <input
                type="password"
                required
                placeholder="••••••••••••"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                className="w-full pl-9 pr-4 py-2.5 bg-[#0B0F19] border border-[#1E293B] rounded-xl text-xs text-white placeholder-gray-600 focus:outline-none focus:border-primary-500 transition-colors font-mono"
              />
            </div>
          </div>

          <button
            type="submit"
            disabled={isSubmitting || !username || !password}
            className="w-full py-2.5 rounded-xl bg-primary-500 hover:bg-primary-400 text-black font-extrabold text-xs transition-all shadow-lg shadow-cyan-900/30 flex items-center justify-center gap-2 disabled:opacity-50 disabled:cursor-not-allowed mt-2"
          >
            {isSubmitting ? (
              'Authenticating...'
            ) : (
              <>
                Sign In to Admin Center
                <ArrowRight className="w-4 h-4" />
              </>
            )}
          </button>
        </form>

        <div className="pt-4 border-t border-[#1E293B] flex items-center justify-between text-[11px] text-gray-500">
          <div className="flex items-center gap-1">
            <Shield className="w-3.5 h-3.5 text-primary-400" />
            <span>MFA Protected</span>
          </div>
          <span>Environment: <strong className="text-amber-400 font-mono">TESTNET</strong></span>
        </div>
      </div>
    </div>
  );
};
