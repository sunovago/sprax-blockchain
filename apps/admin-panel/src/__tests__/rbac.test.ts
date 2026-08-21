import { describe, it, expect } from 'vitest';

describe('RBAC Permission Matrix Tests', () => {
  const superAdmin = { role: 'super_admin', permissions: ['*'] };
  const operationsAdmin = { role: 'operations', permissions: ['blockchain.read', 'explorer.*', 'indexer.retry', 'operations.*', 'notifications.send'] };
  const viewerAdmin = { role: 'viewer', permissions: ['*.read'] };

  const hasPermission = (user: { role: string; permissions: string[] }, perm: string): boolean => {
    if (user.role === 'super_admin') return true;
    if (user.permissions.includes('*')) return true;
    if (user.permissions.includes(perm)) return true;
    const [domain] = perm.split('.');
    return user.permissions.includes(`${domain}.*`);
  };

  it('super_admin has permission to all endpoints and operations', () => {
    expect(hasPermission(superAdmin, 'feature_flags.manage')).toBe(true);
    expect(hasPermission(superAdmin, 'indexer.retry')).toBe(true);
    expect(hasPermission(superAdmin, 'admin.manage')).toBe(true);
  });

  it('operations role can read blockchain and trigger indexer retry', () => {
    expect(hasPermission(operationsAdmin, 'blockchain.read')).toBe(true);
    expect(hasPermission(operationsAdmin, 'indexer.retry')).toBe(true);
    expect(hasPermission(operationsAdmin, 'feature_flags.manage')).toBe(false);
  });

  it('viewer role cannot mutate feature flags or retry indexer', () => {
    expect(hasPermission(viewerAdmin, 'feature_flags.manage')).toBe(false);
    expect(hasPermission(viewerAdmin, 'indexer.retry')).toBe(false);
  });
});
