import type { GroupingMode, Platform, ToastPosition, TrophyTier } from './types';

export const tierLabels: Record<TrophyTier, string> = {
  bronze: 'Bronze',
  silver: 'Argent',
  gold: 'Or',
  platinum: 'Platine',
};

export const platformLabels: Record<Platform, string> = {
  playstation: 'PlayStation',
  steam: 'Steam',
  nintendo: 'Switch',
};

export const groupingLabels: Record<GroupingMode, string> = {
  all: 'Toutes les plateformes',
  playstation: 'Versions PlayStation uniquement',
  none: 'Aucun regroupement',
};

export const toastPositionLabels: Record<ToastPosition, string> = {
  topRight: 'En haut à droite',
  topLeft: 'En haut à gauche',
};

const earnedDateFormat = new Intl.DateTimeFormat('fr-FR', { dateStyle: 'long', timeStyle: 'short' });

/** « 14 juillet 2017 à 21:05 ». */
export function earnedDate(iso: string): string {
  return earnedDateFormat.format(new Date(iso));
}

/** Pourcentage arrondi à l'unité inférieure : un jeu n'affiche 100 % que s'il est complet. */
export function percent(progress: number): number {
  return Math.floor(progress * 100);
}

export function formatNumber(value: number): string {
  return value.toLocaleString('fr-FR');
}
