import type { Appearance, ThemePalette } from './types';

/** Couleurs qui dépendent seulement de l'apparence claire ou sombre du thème. */
const APPEARANCE_VARIABLES: Record<Appearance, Record<string, string>> = {
  dark: {
    '--ink': '255 255 255',
    '--on-ink': '#000000',
    '--on-highlight': '#050b1e',
    '--sunken': 'rgba(0, 0, 0, 0.25)',
    '--dialog': '#141a2a',
    '--toast-tint': '#1c2333',
  },
  light: {
    '--ink': '20 22 34',
    '--on-ink': '#ffffff',
    '--on-highlight': '#ffffff',
    '--sunken': 'rgba(20, 22, 34, 0.05)',
    '--dialog': '#ffffff',
    '--toast-tint': '#ffffff',
  },
};

/** Luminance relative WCAG d'une couleur `#rrggbb`, de 0 (noir) à 1 (blanc). */
function luminance(color: string): number {
  const linear = (offset: number) => {
    const value = parseInt(color.slice(offset, offset + 2), 16) / 255;
    return value <= 0.03928 ? value / 12.92 : ((value + 0.055) / 1.055) ** 2.4;
  };
  return 0.2126 * linear(1) + 0.7152 * linear(3) + 0.0722 * linear(5);
}

/** Texte noir ou blanc, selon celui qui contraste le mieux avec la couleur de fond. */
function readableOn(color: string): string {
  const contrastWithWhite = 1.05 / (luminance(color) + 0.05);
  const contrastWithBlack = (luminance(color) + 0.05) / 0.05;
  return contrastWithWhite >= contrastWithBlack ? '#ffffff' : '#000000';
}

/** Variables CSS dérivées d'une palette ; les composants n'utilisent que ces variables. */
export function paletteVariables(palette: ThemePalette): Record<string, string> {
  const [top, middle, bottom] = palette.background;
  const halo = (color: string, size: number) => `0 0 ${size}px color-mix(in srgb, ${color} 65%, transparent)`;
  return {
    ...APPEARANCE_VARIABLES[palette.appearance],
    'color-scheme': palette.appearance,
    '--background-top': top,
    '--background-bottom': bottom,
    '--background': `linear-gradient(180deg, ${top} 0%, ${middle} 50%, ${bottom} 100%)`,
    '--accent': palette.accent,
    '--on-accent': readableOn(palette.accent),
    '--earned': palette.highlight,
    '--progress': `linear-gradient(90deg, ${palette.progress.join(', ')})`,
    '--glow': palette.glows ? halo(palette.accent, 10) : 'none',
    '--text-glow': palette.glows ? halo(palette.highlight, 8) : 'none',
  };
}

/** Applique la palette à toute la page. */
export function applyPalette(palette: ThemePalette) {
  for (const [name, value] of Object.entries(paletteVariables(palette))) {
    document.documentElement.style.setProperty(name, value);
  }
}

/** Style en ligne qui applique une palette à un seul élément, pour un aperçu. */
export function paletteStyle(palette: ThemePalette): string {
  return Object.entries(paletteVariables(palette))
    .map(([name, value]) => `${name}: ${value}`)
    .join('; ');
}
