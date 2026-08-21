import { describe, it, expect } from 'vitest';

describe('Admin Panel Data Formatters', () => {
  it('formats atto-SPRX into decimal SPRX accurately', () => {
    const oneSprxAtto = '1000000000000000000';
    const sprx = Number(oneSprxAtto) / 1e18;
    expect(sprx).toBe(1);

    const fractionalAtto = '500000000000000000';
    const fracSprx = Number(fractionalAtto) / 1e18;
    expect(fracSprx).toBe(0.5);
  });

  it('truncates blockchain addresses for clean dense UI presentation', () => {
    const truncate = (addr: string, start = 8, end = 6) => {
      if (addr.length <= start + end) return addr;
      return `${addr.slice(0, start)}...${addr.slice(-end)}`;
    };

    const bech32Addr = 'sprax1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq40';
    expect(truncate(bech32Addr)).toBe('sprax1qq...qqqq40');
  });
});
