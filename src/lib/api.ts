import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

import type {
  Announcement,
  CatalogItem,
  GameDetail,
  LibraryOverview,
  Settings,
  SyncStatus,
  SystemInfo,
  ThemeOption,
  ThemePalette,
  TrophyReference,
  TrophyTier,
} from './types';


const tauriApi = {
  libraryOverview: () => invoke<LibraryOverview>('library_overview'),
  gameDetail: (id: string) => invoke<GameDetail | null>('game_detail', { id }),
  setTrophyEarned: (reference: TrophyReference, earned: boolean) => invoke<void>('set_trophy_earned', { reference, earned }),
  setTrophyTier: (reference: TrophyReference, tier: TrophyTier | null) => invoke<void>('set_trophy_tier', { reference, tier }),
  removeGame: (id: string) => invoke<void>('remove_game', { id }),
  switchCatalog: () => invoke<CatalogItem[]>('switch_catalog'),
  addSwitchGame: (catalogId: string) => invoke<void>('add_switch_game', { catalogId }),
  addCustomSwitchGame: (title: string) => invoke<void>('add_custom_switch_game', { title }),
  syncAll: () => invoke<void>('sync_all'),
  importSteamHistory: () => invoke<void>('import_steam_history'),
  syncPlaystation: () => invoke<void>('sync_playstation'),
  connectPlaystation: (npsso: string) => invoke<void>('connect_playstation', { npsso }),
  disconnectPlaystation: (removingTrophies: boolean) => invoke<void>('disconnect_playstation', { removingTrophies }),
  steamApiKey: () => invoke<string>('steam_api_key'),
  setSteamApiKey: (key: string) => invoke<void>('set_steam_api_key', { key }),
  syncStatus: () => invoke<SyncStatus>('sync_status'),
  settings: () => invoke<Settings>('settings'),
  updateSettings: (settings: Settings) => invoke<void>('update_settings', { settings }),
  testNotification: (tier: TrophyTier | null) => invoke<void>('test_notification', { tier }),
  systemInfo: () => invoke<SystemInfo>('system_info'),
  setLaunchAtLogin: (enabled: boolean) => invoke<void>('set_launch_at_login', { enabled }),
  themeGallery: () => invoke<ThemeOption[]>('theme_gallery'),
  activeTheme: () => invoke<ThemePalette>('active_theme'),
};

const tauriEvents = {
  onLibraryChanged: (handler: () => void): Promise<UnlistenFn> => listen('library-changed', handler),
  onStatusChanged: (handler: (status: SyncStatus) => void): Promise<UnlistenFn> =>
    listen<SyncStatus>('status-changed', (event) => handler(event.payload)),
  onNavigate: (handler: (view: string) => void): Promise<UnlistenFn> => listen<string>('navigate', (event) => handler(event.payload)),
  onToastShow: (handler: (announcement: Announcement) => void): Promise<UnlistenFn> =>
    listen<Announcement>('toast-show', (event) => handler(event.payload)),
  onToastHide: (handler: () => void): Promise<UnlistenFn> => listen('toast-hide', handler),
  onThemeChanged: (handler: (palette: ThemePalette) => void): Promise<UnlistenFn> =>
    listen<ThemePalette>('theme-changed', (event) => handler(event.payload)),
};

export type Api = typeof tauriApi;
export type Events = typeof tauriEvents;

/**
 * En mode démo (`npm run demo`), l'interface tourne seule dans un navigateur, sur une bibliothèque fictive.
 * Le module de démo n'est chargé que dans ce mode : il est absent de l'app distribuée.
 */
const demo = import.meta.env.MODE === 'demo' ? await import('./demo') : null;

/** Accès au backend Rust : commandes et événements. */
export const api: Api = demo?.demoApi ?? tauriApi;
export const events: Events = demo?.demoEvents ?? tauriEvents;
