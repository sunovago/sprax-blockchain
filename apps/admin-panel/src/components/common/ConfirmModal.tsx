import React, { useState } from 'react';
import { AlertTriangle, X } from 'lucide-react';

interface ConfirmModalProps {
  isOpen: boolean;
  title: string;
  message: string;
  confirmPhrase?: string;
  confirmLabel?: string;
  isDangerous?: boolean;
  onConfirm: () => void;
  onCancel: () => void;
}

export const ConfirmModal: React.FC<ConfirmModalProps> = ({
  isOpen,
  title,
  message,
  confirmPhrase,
  confirmLabel = 'Confirm Action',
  isDangerous = false,
  onConfirm,
  onCancel,
}) => {
  const [typedValue, setTypedValue] = useState('');

  if (!isOpen) return null;

  const isConfirmed = confirmPhrase ? typedValue === confirmPhrase : true;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/75 backdrop-blur-sm animate-fade-in">
      <div className="bg-[#111827] border border-[#1E293B] rounded-2xl max-w-md w-full p-6 shadow-2xl relative">
        <button
          onClick={onCancel}
          className="absolute right-4 top-4 text-gray-400 hover:text-white p-1 rounded-lg transition-colors"
        >
          <X className="w-5 h-5" />
        </button>

        <div className="flex items-center gap-3">
          <div
            className={`p-2.5 rounded-xl ${
              isDangerous ? 'bg-rose-950/60 text-rose-400 border border-rose-800' : 'bg-amber-950/60 text-amber-400 border border-amber-800'
            }`}
          >
            <AlertTriangle className="w-5 h-5" />
          </div>
          <h3 className="text-base font-bold text-white">{title}</h3>
        </div>

        <p className="mt-3 text-xs text-gray-300 leading-relaxed">{message}</p>

        {confirmPhrase && (
          <div className="mt-4">
            <label className="block text-xs font-semibold text-gray-400 mb-1.5">
              Type <span className="text-rose-400 font-mono font-bold select-all">{confirmPhrase}</span> to continue:
            </label>
            <input
              type="text"
              value={typedValue}
              onChange={(e) => setTypedValue(e.target.value)}
              placeholder={confirmPhrase}
              className="w-full px-3 py-2 bg-[#0B0F19] border border-[#1E293B] rounded-lg text-xs font-mono text-white placeholder-gray-600 focus:outline-none focus:border-rose-500 transition-colors"
            />
          </div>
        )}

        <div className="mt-6 flex items-center justify-end gap-3">
          <button
            onClick={onCancel}
            className="px-4 py-2 rounded-xl text-xs font-semibold text-gray-300 hover:bg-[#1E293B] border border-[#1E293B] transition-colors"
          >
            Cancel
          </button>
          <button
            onClick={() => {
              if (isConfirmed) {
                onConfirm();
                setTypedValue('');
              }
            }}
            disabled={!isConfirmed}
            className={`px-4 py-2 rounded-xl text-xs font-semibold transition-all disabled:opacity-40 disabled:cursor-not-allowed ${
              isDangerous
                ? 'bg-rose-600 hover:bg-rose-500 text-white shadow-lg shadow-rose-900/30'
                : 'bg-primary-500 hover:bg-primary-400 text-black shadow-lg shadow-cyan-900/30 font-bold'
            }`}
          >
            {confirmLabel}
          </button>
        </div>
      </div>
    </div>
  );
};
