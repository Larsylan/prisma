import type { Locale } from '../types';
import { en } from './en';
import { fr, type Dictionary } from './fr';
import { it } from './it';

const DICTIONARIES: Record<Locale, Dictionary> = { fr, en, it };

const active = $state({ locale: 'fr' as Locale });

/** Textes de la langue active ; un composant qui les lit se met à jour quand la langue change. */
export const t: Dictionary = new Proxy({} as Dictionary, {
  get: (_, key) => DICTIONARIES[active.locale][key as keyof Dictionary],
});

export function currentLocale(): Locale {
  return active.locale;
}

export function setLocale(locale: Locale) {
  active.locale = locale;
  document.documentElement.lang = locale;
}
