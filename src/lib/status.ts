import { shortTime } from './format';
import { t } from './i18n/index.svelte';
import type { Failure, PlayStationState, SteamLocalState, SteamWebState } from './types';

/** Phrases affichées pour l'état des sources ; une chaîne vide n'affiche rien. */
export function steamLocalText(state: SteamLocalState): string {
  switch (state.state) {
    case 'unknown':
      return '';
    case 'notInstalled':
      return t.status.steamNotInstalled;
    case 'detected': {
      const summary = t.status.steamDetected(state.gameCount);
      return state.player ? `${summary} · ${state.player}` : summary;
    }
  }
}

export function steamWebText(state: SteamWebState): string {
  switch (state.state) {
    case 'idle':
      return '';
    case 'importing':
      return state.total === 0 ? t.status.steamWebStarting : t.status.steamWebImporting(state.analyzed, state.total);
    case 'imported':
      return t.status.steamWebImported(state.gameCount);
    case 'failed':
      return failureText(state.failure);
  }
}

export function playstationText(state: PlayStationState): string {
  switch (state.state) {
    case 'disconnected':
      return '';
    case 'connecting':
      return t.status.playstationConnecting;
    case 'connected':
      return t.status.playstationConnected;
    case 'syncing':
      return t.status.playstationSyncing;
    case 'updating':
      return t.status.playstationUpdating(state.updated, state.total);
    case 'synced':
      return t.status.playstationSynced(state.gameCount, shortTime(state.at));
    case 'failed':
      return failureText(state.failure);
  }
}

function failureText(failure: Failure): string {
  switch (failure.reason) {
    case 'invalidToken':
      return t.status.invalidToken;
    case 'notConnected':
      return t.status.notConnected;
    case 'http':
      return t.status.httpError(failure.service, failure.status);
    case 'network':
      return t.status.networkError(failure.service, failure.detail);
  }
}

