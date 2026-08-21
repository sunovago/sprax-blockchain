import React, { useEffect, useState } from 'react';
import { Sliders, Save, AlertCircle } from 'lucide-react';
import { api } from '../services/api';
import { AppVersionConfig } from '../types';

export const AppVersionsPage: React.FC = () => {
  const [config, setConfig] = useState<AppVersionConfig>({
    platform: 'android',
    min_version: '1.0.0',
    latest_version: '1.0.0',
    recommended_version: '1.0.0',
    force_update: false,
    maintenance_mode: false,
    maintenance_message: 'SPRX Network is undergoing scheduled maintenance.',
    update_url_android: 'https://github.com/sprax-chain/sprax-chain/releases/latest',
  });
  const [isSaving, setIsSaving] = useState(false);
  const [saveSuccess, setSaveSuccess] = useState(false);

  useEffect(() => {
    const fetchConfig = async () => {
      try {
        const res = await api.getAppVersions();
        setConfig(res);
      } catch (err) {
        console.error('Failed to fetch version config', err);
      }
    };
    fetchConfig();
  }, []);

  const handleSave = async (e: React.FormEvent) => {
    e.preventDefault();
    setIsSaving(true);
    setSaveSuccess(false);
    try {
      await api.updateAppVersions(config);
      setSaveSuccess(true);
      setTimeout(() => setSaveSuccess(false), 3000);
    } catch (err) {
      alert(err instanceof Error ? err.message : 'Update failed');
    } finally {
      setIsSaving(false);
    }
  };

  return (
    <div className="space-y-6 max-w-3xl">
      <div>
        <h1 className="text-xl font-bold text-white tracking-tight">Mobile App Version Control</h1>
        <p className="text-xs text-gray-400 mt-1">
          Configure minimum supported versions, force update policies, and remote maintenance alerts.
        </p>
      </div>

      <form onSubmit={handleSave} className="bg-[#111827] border border-[#1E293B] rounded-2xl p-6 space-y-5">
        <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
          <div>
            <label className="block text-xs font-semibold text-gray-300 mb-1">
              Minimum Supported Version
            </label>
            <input
              type="text"
              value={config.min_version}
              onChange={(e) => setConfig({ ...config, min_version: e.target.value })}
              className="w-full px-3 py-2 bg-[#0B0F19] border border-[#1E293B] rounded-lg text-xs font-mono text-white focus:outline-none focus:border-primary-500"
            />
          </div>
          <div>
            <label className="block text-xs font-semibold text-gray-300 mb-1">
              Latest Released Version
            </label>
            <input
              type="text"
              value={config.latest_version}
              onChange={(e) => setConfig({ ...config, latest_version: e.target.value })}
              className="w-full px-3 py-2 bg-[#0B0F19] border border-[#1E293B] rounded-lg text-xs font-mono text-white focus:outline-none focus:border-primary-500"
            />
          </div>
        </div>

        <div>
          <label className="block text-xs font-semibold text-gray-300 mb-1">
            Android APK Download URL
          </label>
          <input
            type="text"
            value={config.update_url_android}
            onChange={(e) => setConfig({ ...config, update_url_android: e.target.value })}
            className="w-full px-3 py-2 bg-[#0B0F19] border border-[#1E293B] rounded-lg text-xs font-mono text-white focus:outline-none focus:border-primary-500"
          />
        </div>

        <div className="pt-2 border-t border-[#1E293B] space-y-4">
          <div className="flex items-center justify-between">
            <div>
              <div className="text-xs font-semibold text-white">Force App Update</div>
              <div className="text-[11px] text-gray-400">Block users below the minimum version from transacting</div>
            </div>
            <input
              type="checkbox"
              checked={config.force_update}
              onChange={(e) => setConfig({ ...config, force_update: e.target.checked })}
              className="w-4 h-4 rounded border-gray-700 bg-gray-900 text-primary-500 focus:ring-primary-500"
            />
          </div>

          <div className="flex items-center justify-between">
            <div>
              <div className="text-xs font-semibold text-white">App Maintenance Mode</div>
              <div className="text-[11px] text-gray-400">Show maintenance banner in Flutter wallet</div>
            </div>
            <input
              type="checkbox"
              checked={config.maintenance_mode}
              onChange={(e) => setConfig({ ...config, maintenance_mode: e.target.checked })}
              className="w-4 h-4 rounded border-gray-700 bg-gray-900 text-primary-500 focus:ring-primary-500"
            />
          </div>
        </div>

        <div>
          <label className="block text-xs font-semibold text-gray-300 mb-1">
            Maintenance Message
          </label>
          <textarea
            value={config.maintenance_message}
            onChange={(e) => setConfig({ ...config, maintenance_message: e.target.value })}
            rows={3}
            className="w-full px-3 py-2 bg-[#0B0F19] border border-[#1E293B] rounded-lg text-xs text-white focus:outline-none focus:border-primary-500"
          />
        </div>

        <div className="flex items-center justify-between pt-2">
          {saveSuccess ? (
            <div className="flex items-center gap-1.5 text-xs text-emerald-400">
              <AlertCircle className="w-4 h-4" />
              Settings updated and audited successfully!
            </div>
          ) : <div />}

          <button
            type="submit"
            disabled={isSaving}
            className="inline-flex items-center gap-2 px-5 py-2 rounded-xl bg-primary-500 hover:bg-primary-400 text-black font-bold text-xs transition-all shadow-lg shadow-cyan-900/30 disabled:opacity-50"
          >
            <Save className="w-4 h-4" />
            {isSaving ? 'Saving...' : 'Save Configuration'}
          </button>
        </div>
      </form>
    </div>
  );
};
