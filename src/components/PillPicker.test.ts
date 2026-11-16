import { fireEvent, render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';

import PillPicker from './PillPicker.svelte';

describe('PillPicker', () => {
  it('selects the clicked option', async () => {
    const options = [
      { value: 'all', label: 'Tous' },
      { value: 'earned', label: 'Obtenus' },
    ];
    render(PillPicker, { options, value: 'all' });

    await fireEvent.click(screen.getByRole('radio', { name: 'Obtenus' }));

    expect(screen.getByRole('radio', { name: 'Obtenus' }).getAttribute('aria-checked')).toBe('true');
    expect(screen.getByRole('radio', { name: 'Tous' }).getAttribute('aria-checked')).toBe('false');
  });
});
