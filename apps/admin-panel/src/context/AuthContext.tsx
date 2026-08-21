import React, { createContext, useContext, useState, useEffect } from 'react';
import { AdminProfile } from '../types';
import { api } from '../services/api';

interface AuthContextType {
  user: AdminProfile | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  login: (u: string, p: string) => Promise<void>;
  logout: () => void;
  hasPermission: (perm: string) => boolean;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

export const AuthProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const [user, setUser] = useState<AdminProfile | null>(null);
  const [isLoading, setIsLoading] = useState(true);

  const checkAuth = async () => {
    const token = localStorage.getItem('sprax_admin_token');
    if (!token) {
      setUser(null);
      setIsLoading(false);
      return;
    }
    try {
      const profile = await api.getMe();
      setUser(profile);
    } catch {
      setUser(null);
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    checkAuth();
    const handleAuthChange = () => checkAuth();
    window.addEventListener('sprax_auth_changed', handleAuthChange);
    return () => window.removeEventListener('sprax_auth_changed', handleAuthChange);
  }, []);

  const login = async (u: string, p: string) => {
    await api.login(u, p);
    await checkAuth();
  };

  const logout = () => {
    api.logout();
    setUser(null);
  };

  const hasPermission = (perm: string): boolean => {
    if (!user) return false;
    if (user.role === 'super_admin') return true;
    if (user.permissions.includes('*')) return true;
    if (user.permissions.includes(perm)) return true;
    const [domain] = perm.split('.');
    return user.permissions.includes(`${domain}.*`);
  };

  return (
    <AuthContext.Provider
      value={{
        user,
        isAuthenticated: !!user,
        isLoading,
        login,
        logout,
        hasPermission,
      }}
    >
      {children}
    </AuthContext.Provider>
  );
};

export const useAuth = () => {
  const ctx = useContext(AuthContext);
  if (!ctx) throw new Error('useAuth must be used within an AuthProvider');
  return ctx;
};
