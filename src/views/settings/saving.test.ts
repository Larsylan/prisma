import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { debounced } from './saving';

describe('debounced', () => {
  beforeEach(() => vi.useFakeTimers());
  afterEach(() => vi.useRealTimers());

  it('runs once after a burst of calls', () => {
    const action = vi.fn();
    const save = debounced(action);

    save(400);
    save(400);
    save(400);
    vi.advanceTimersByTime(399);
    expect(action).not.toHaveBeenCalled();

    vi.advanceTimersByTime(1);
    expect(action).toHaveBeenCalledOnce();
  });
});
