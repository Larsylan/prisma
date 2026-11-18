import { afterEach, describe, expect, it } from 'vitest';

import { setLocale } from './i18n/index.svelte';
import { playstationText, steamLocalText, steamWebText } from './status';

afterEach(() => setLocale('fr'));

describe('source states', () => {
  it('describe the local Steam client in the active language', () => {
    const detected = { state: 'detected', gameCount: 3, player: 'Alex' } as const;
    expect(steamLocalText(detected)).toBe('3 jeux Steam détectés en local · Alex');
    setLocale('en');
    expect(steamLocalText(detected)).toBe('3 Steam games found locally · Alex');
    expect(steamLocalText({ state: 'detected', gameCount: 1, player: null })).toBe('1 Steam game found locally');
  });

  it('explain failures instead of showing raw errors', () => {
    setLocale('en');
    expect(playstationText({ state: 'failed', failure: { reason: 'invalidToken' } })).toMatch(/^Invalid or expired NPSSO token/);
    expect(steamWebText({ state: 'failed', failure: { reason: 'http', service: 'Steam Web API', status: 403 } })).toBe(
      'Steam Web API: HTTP error 403.',
    );
  });

  it('stay silent when there is nothing to report', () => {
    expect(steamLocalText({ state: 'unknown' })).toBe('');
    expect(steamWebText({ state: 'idle' })).toBe('');
    expect(playstationText({ state: 'disconnected' })).toBe('');
  });
});
