import { afterEach, describe, expect, it } from 'vitest';

import { earnedDate, formatNumber, percent } from './format';
import { setLocale } from './i18n/index.svelte';

afterEach(() => setLocale('fr'));

describe('percent', () => {
  it('rounds down so that only a complete game shows 100 %', () => {
    expect(percent(0.999)).toBe(99);
    expect(percent(1)).toBe(100);
    expect(percent(0)).toBe(0);
  });
});

describe('earnedDate', () => {
  it('writes the date in the active language', () => {
    expect(earnedDate('2017-07-14T19:05:00Z')).toMatch(/14 juillet 2017/);
    setLocale('en');
    expect(earnedDate('2017-07-14T19:05:00Z')).toMatch(/July 14, 2017/);
  });
});

describe('formatNumber', () => {
  it('groups digits the way the active language does', () => {
    setLocale('en');
    expect(formatNumber(12345)).toBe('12,345');
  });
});
