import { fireEvent, render, screen } from '@testing-library/svelte';
import { describe, expect, it, vi } from 'vitest';

import type { TrophyView } from '../../lib/types';

const { setTrophyEarned } = vi.hoisted(() => ({ setTrophyEarned: vi.fn() }));
vi.mock('../../lib/api', () => ({ api: { setTrophyEarned } }));

const { default: TrophyRow } = await import('./TrophyRow.svelte');

function trophy(overrides: Partial<TrophyView>): TrophyView {
  return {
    id: '1',
    reference: { gameId: 'switch:custom-tiny-isles', trophyId: 'premier-boss' },
    name: 'Premier boss',
    detail: 'Vaincre un premier boss.',
    tier: 'bronze',
    isHidden: false,
    isGeneratedPlatinum: false,
    iconUrl: null,
    isEarned: false,
    earnedAt: null,
    earnedOn: [],
    canToggleEarned: false,
    canChangeTier: false,
    hasTierOverride: false,
    ...overrides,
  };
}

describe('TrophyRow', () => {
  it('conceals a hidden trophy until it is revealed', async () => {
    render(TrophyRow, { trophy: trophy({ isHidden: true }), gameTitle: 'Tiny Isles', isGrayscaleWhenLocked: false });
    expect(screen.getByText('Trophée caché')).toBeTruthy();

    await fireEvent.click(screen.getByRole('button', { name: 'Afficher' }));

    expect(screen.getByText('Premier boss')).toBeTruthy();
  });

  it('checks a Switch trophy by hand', async () => {
    const view = trophy({ canToggleEarned: true });
    render(TrophyRow, { trophy: view, gameTitle: 'Tiny Isles', isGrayscaleWhenLocked: false });

    await fireEvent.click(screen.getByRole('button', { name: 'Marquer comme obtenu' }));

    expect(setTrophyEarned).toHaveBeenCalledWith(view.reference, true);
  });

  it('shows where and when a trophy was earned', () => {
    render(TrophyRow, {
      trophy: trophy({ isEarned: true, earnedAt: '2017-07-14T19:05:00Z', earnedOn: ['PS4', 'PS5'] }),
      gameTitle: 'Tiny Isles', isGrayscaleWhenLocked: false,
    });

    expect(screen.getByText(/Obtenu le 14 juillet 2017 .* · PS4, PS5/)).toBeTruthy();
  });
});
