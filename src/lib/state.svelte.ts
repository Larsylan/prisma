import { api, events } from './api';
import { applyPalette } from './palette';
import type { LibraryOverview, SyncStatus } from './types';

/** État partagé par les écrans, tenu à jour par les événements du backend. */
class AppState {
  overview = $state<LibraryOverview | null>(null);
  status = $state<SyncStatus | null>(null);
  /** Incrémenté à chaque modification de la bibliothèque, pour recharger les fiches ouvertes. */
  revision = $state(0);

  async start() {
    await Promise.all([
      events.onLibraryChanged(() => this.refresh()),
      events.onStatusChanged((status) => (this.status = status)),
      events.onThemeChanged(applyPalette),
    ]);
    const [overview, status, palette] = await Promise.all([api.libraryOverview(), api.syncStatus(), api.activeTheme()]);
    applyPalette(palette);
    [this.overview, this.status] = [overview, status];
  }

  async refresh() {
    this.overview = await api.libraryOverview();
    this.revision += 1;
  }
}

export const appState = new AppState();
