// Données échangées avec le backend Rust (voir src-tauri/src/views.rs).

export type Platform = 'playstation' | 'steam' | 'nintendo';
export type TrophyTier = 'bronze' | 'silver' | 'gold' | 'platinum';
export type GroupingMode = 'all' | 'playstation' | 'none';
export type ToastPosition = 'topRight' | 'topLeft';
export type Appearance = 'dark' | 'light';
export type Language = 'system' | 'french' | 'english';
/** Langue effectivement affichée. */
export type Locale = 'fr' | 'en';
export type Preset =
  | 'night'
  | 'indigo'
  | 'amethyst'
  | 'sakura'
  | 'ruby'
  | 'ember'
  | 'amber'
  | 'emerald'
  | 'ocean'
  | 'aurora'
  | 'twilight'
  | 'graphite'
  | 'neonCyan'
  | 'neonMagenta'
  | 'neonAcid'
  | 'synthwave'
  | 'whiteNeon'
  | 'spectralBlack';
/** Couleur au format `#rrggbb`. */
export type Color = string;

export interface ThemePalette {
  appearance: Appearance;
  background: [Color, Color, Color];
  accent: Color;
  /** Mentions « Obtenu » et cases cochées. */
  highlight: Color;
  /** Couleurs successives des barres de progression. */
  progress: Color[];
  glows: boolean;
}

export interface ThemeOption {
  preset: Preset;
  palette: ThemePalette;
}

export interface TierCount {
  tier: TrophyTier;
  earned: number;
  total: number;
}

export interface TierQuantity {
  tier: TrophyTier;
  count: number;
}

export interface Profile {
  name: string;
  level: number;
  levelProgress: number;
  nextLevel: number;
  levelTier: TrophyTier;
  points: number;
  earnedByTier: TierQuantity[];
  earnedTotal: number;
}

export interface GameSummary {
  id: string;
  title: string;
  platform: Platform;
  platforms: Platform[];
  systems: string[];
  systemLabel: string;
  coverUrl: string | null;
  progress: number;
  tierCounts: TierCount[];
  isPlatinumEarned: boolean;
  isGroup: boolean;
  earnedCount: number;
  trophyCount: number;
}

export interface LibraryOverview {
  profile: Profile;
  games: GameSummary[];
}

export interface TrophyReference {
  gameId: string;
  trophyId: string;
}

export interface TrophyView {
  id: string;
  reference: TrophyReference;
  name: string;
  detail: string;
  tier: TrophyTier;
  isHidden: boolean;
  /** Platine ajouté par Prisma aux jeux Steam et Switch. */
  isGeneratedPlatinum: boolean;
  iconUrl: string | null;
  isEarned: boolean;
  earnedAt: string | null;
  earnedOn: string[];
  canToggleEarned: boolean;
  canChangeTier: boolean;
  hasTierOverride: boolean;
}

export interface GameDetail extends GameSummary {
  isRemovable: boolean;
  trophies: TrophyView[];
}

export interface CatalogItem {
  id: string;
  title: string;
  trophyCounts: TierQuantity[];
  isAdded: boolean;
}

export type Failure =
  | { reason: 'invalidToken' }
  | { reason: 'notConnected' }
  | { reason: 'http'; service: string; status: number }
  | { reason: 'network'; service: string; detail: string };

export type SteamLocalState = { state: 'unknown' } | { state: 'notInstalled' } | { state: 'detected'; gameCount: number; player: string | null };

export type SteamWebState =
  | { state: 'idle' }
  | { state: 'importing'; analyzed: number; total: number }
  | { state: 'imported'; gameCount: number }
  | { state: 'failed'; failure: Failure };

export type PlayStationState =
  | { state: 'disconnected' }
  | { state: 'connecting' }
  | { state: 'connected' }
  | { state: 'syncing' }
  | { state: 'updating'; updated: number; total: number }
  | { state: 'synced'; gameCount: number; at: string }
  | { state: 'failed'; failure: Failure };

export interface SyncStatus {
  steamLocal: SteamLocalState;
  steamWatching: boolean;
  steamWeb: SteamWebState;
  playstation: PlayStationState;
  playstationConnected: boolean;
  syncing: boolean;
}

export interface Settings {
  profileName: string;
  steamId64: string;
  useSteamRarity: boolean;
  playstationPolling: boolean;
  groupingMode: GroupingMode;
  soundEnabled: boolean;
  toastDurationSeconds: number;
  toastPosition: ToastPosition;
  theme: Preset;
  language: Language;
}

export interface SystemInfo {
  dataDirectory: string;
  detectedSteamId64: string | null;
  launchAtLogin: boolean;
}

export interface Announcement {
  trophyName: string;
  tier: TrophyTier;
  iconUrl: string | null;
  gameTitle: string;
  slidesFromLeft: boolean;
}
