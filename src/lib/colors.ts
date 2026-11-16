import type { Platform, TrophyTier } from './types';

/** Teintes claire et sombre du cristal de chaque grade. */
export const tierPalettes: Record<TrophyTier, { light: number; dark: number }> = {
  bronze: { light: 0xf7c08a, dark: 0x7a3e18 },
  silver: { light: 0xffffff, dark: 0x6e7884 },
  gold: { light: 0xfff0a0, dark: 0xa8700f },
  platinum: { light: 0xffffff, dark: 0x6f86c2 },
};

export const platformColors: Record<Platform, string> = {
  playstation: '#003791',
  steam: '#1b2838',
  nintendo: '#8e1e2e',
};

export const coverGradients: Record<Platform, [string, string]> = {
  playstation: ['#1f4fb0', '#0a1638'],
  steam: ['#2a475e', '#111a24'],
  nintendo: ['#8e1e2e', '#2a0c24'],
};

/** Mélange linéaire de deux couleurs : 0 donne `start`, 1 donne `end`. */
export function mix(start: number, end: number, fraction: number): string {
  const channel = (color: number, shift: number) => (color >> shift) & 0xff;
  const blend = (shift: number) => Math.round(channel(start, shift) + (channel(end, shift) - channel(start, shift)) * fraction);
  return `rgb(${blend(16)}, ${blend(8)}, ${blend(0)})`;
}

export function tierGradient(tier: TrophyTier): [string, string] {
  const { light, dark } = tierPalettes[tier];
  return [mix(light, dark, 0.15), mix(light, dark, 0.6)];
}
