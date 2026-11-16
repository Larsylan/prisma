import { describe, expect, it } from 'vitest';

import { earnedDate, percent } from './format';

describe('percent', () => {
  it('rounds down so that only a complete game shows 100 %', () => {
    expect(percent(0.999)).toBe(99);
    expect(percent(1)).toBe(100);
    expect(percent(0)).toBe(0);
  });
});

describe('earnedDate', () => {
  it('writes the date in French', () => {
    expect(earnedDate('2017-07-14T19:05:00Z')).toMatch(/14 juillet 2017/);
  });
});
