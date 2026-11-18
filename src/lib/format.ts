import { currentLocale } from './i18n/index.svelte';

const LOCALE_TAGS = { fr: 'fr-FR', en: 'en-US' };

const localeTag = () => LOCALE_TAGS[currentLocale()];

/** « 14 juillet 2017 à 21:05 », ou « July 14, 2017 at 9:05 PM ». */
export function earnedDate(iso: string): string {
  return new Intl.DateTimeFormat(localeTag(), { dateStyle: 'long', timeStyle: 'short' }).format(new Date(iso));
}

export function shortTime(iso: string): string {
  return new Intl.DateTimeFormat(localeTag(), { timeStyle: 'short' }).format(new Date(iso));
}

/** Pourcentage arrondi à l'unité inférieure : un jeu n'affiche 100 % que s'il est complet. */
export function percent(progress: number): number {
  return Math.floor(progress * 100);
}

export function formatNumber(value: number): string {
  return value.toLocaleString(localeTag());
}

/** Recherche insensible à la casse et tri alphabétique, selon les règles de la langue active. */
export function matchesSearch(text: string, search: string): boolean {
  return text.toLocaleLowerCase(localeTag()).includes(search.trim().toLocaleLowerCase(localeTag()));
}

export function compareTitles(lhs: string, rhs: string): number {
  return lhs.localeCompare(rhs, localeTag());
}
