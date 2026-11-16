import { describe, expect, it } from 'vitest';

import { paletteVariables } from './palette';
import type { ThemePalette } from './types';

const night: ThemePalette = {
  appearance: 'dark',
  background: ['#0b1d46', '#050b1e', '#02040c'],
  accent: '#2d6bf6',
  highlight: '#6fb1ff',
  progress: ['#6fb1ff', '#2d6bf6'],
  glows: false,
};

const whiteNeon: ThemePalette = {
  appearance: 'light',
  background: ['#ffffff', '#f4f2fb', '#e9e6f5'],
  accent: '#00e5ff',
  highlight: '#a4006f',
  progress: ['#00c8f0', '#8f3dff', '#ff1fb4'],
  glows: true,
};

describe('paletteVariables', () => {
  it('uses white ink on a dark theme and dark ink on a light one', () => {
    expect(paletteVariables(night)['--ink']).toBe('255 255 255');
    expect(paletteVariables(whiteNeon)['--ink']).toBe('20 22 34');
    expect(paletteVariables(whiteNeon)['color-scheme']).toBe('light');
  });

  it('picks the most readable text color for the accent', () => {
    expect(paletteVariables(night)['--on-accent']).toBe('#ffffff');
    expect(paletteVariables(whiteNeon)['--on-accent']).toBe('#000000');
  });

  it('draws every progress color and only makes neon themes glow', () => {
    expect(paletteVariables(whiteNeon)['--progress']).toBe('linear-gradient(90deg, #00c8f0, #8f3dff, #ff1fb4)');
    expect(paletteVariables(whiteNeon)['--glow']).toContain('#00e5ff');
    expect(paletteVariables(night)['--glow']).toBe('none');
  });
});
