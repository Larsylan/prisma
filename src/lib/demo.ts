/**
 * Bibliothèque fictive servie sans backend, pour développer l'interface dans un navigateur
 * et produire les captures d'écran : `npm run demo`.
 */
import type { UnlistenFn } from '@tauri-apps/api/event';

import type { Api, Events } from './api';
import demoThemes from './demo-themes.json';
import type {
  Announcement,
  CatalogItem,
  GameDetail,
  GameSummary,
  LibraryOverview,
  Locale,
  Platform,
  Settings,
  SyncStatus,
  SystemInfo,
  ThemeOption,
  ThemePalette,
  TierCount,
  TrophyTier,
  TrophyView,
} from './types';

const parameters = new URLSearchParams(location.search);

/** `?lang=en` ou `?lang=fr` ; à défaut, la langue du navigateur. */
const LOCALE: Locale = (() => {
  const requested = parameters.get('lang');
  if (requested === 'fr' || requested === 'en') return requested;
  return navigator.language.startsWith('fr') ? 'fr' : 'en';
})();

const NAMES: Record<Locale, [string, string][]> = {
  fr: [
    ['Premier pas', 'Terminer le prologue.'],
    ['Éclaireur', 'Découvrir toutes les régions.'],
    ['Collectionneur', 'Réunir 50 objets rares.'],
    ['Sans une égratignure', 'Vaincre un boss sans subir de dégâts.'],
    ['Maître artisan', 'Fabriquer un objet légendaire.'],
    ['Explorateur aguerri', 'Parcourir 100 km.'],
    ['Duel au sommet', 'Remporter un duel classé.'],
    ['Trésor oublié', 'Ouvrir le coffre caché.'],
    ['Course contre la montre', 'Finir une course en moins de 2 minutes.'],
    ['Ami des bêtes', 'Apprivoiser une créature.'],
    ['Cartographe', 'Compléter la carte.'],
    ['Le grand saut', 'Sauter depuis le plus haut sommet.'],
    ['Lumière retrouvée', 'Rallumer tous les phares.'],
    ['Fin du voyage', "Terminer l'histoire."],
  ],
  en: [
    ['First Steps', 'Finish the prologue.'],
    ['Scout', 'Discover every region.'],
    ['Collector', 'Gather 50 rare items.'],
    ['Not a Scratch', 'Defeat a boss without taking damage.'],
    ['Master Crafter', 'Craft a legendary item.'],
    ['Seasoned Explorer', 'Travel 100 km.'],
    ['Duel at the Top', 'Win a ranked duel.'],
    ['Forgotten Treasure', 'Open the hidden chest.'],
    ['Against the Clock', 'Finish a race in under 2 minutes.'],
    ['Animal Friend', 'Tame a creature.'],
    ['Cartographer', 'Complete the map.'],
    ['The Big Leap', 'Jump from the highest peak.'],
    ['Light Restored', 'Relight every lighthouse.'],
    ["Journey's End", 'Finish the story.'],
  ],
};
const PLAYSTATION_PLATINUM: Record<Locale, [string, string]> = {
  fr: ['Légende vivante', 'Obtenir tous les autres trophées.'],
  en: ['Living Legend', 'Earn every other trophy.'],
};
const DAY = 86_400_000;
const POINTS: Record<TrophyTier, number> = { bronze: 15, silver: 30, gold: 90, platinum: 300 };

interface DemoGame {
  id: string;
  title: string;
  platform: Platform;
  systems: string[];
  trophyCount: number;
  earnedRatio: number;
  daysAgo: number;
  hasPlatinum: boolean;
}

const GAMES: DemoGame[] = [
  { id: 'group:lumen', title: 'Lumen Odyssey', platform: 'playstation', systems: ['PS4', 'PS5'], trophyCount: 14, earnedRatio: 0.86, daysAgo: 0, hasPlatinum: true },
  { id: 'steam:1', title: 'Starfall Colony', platform: 'steam', systems: ['Steam'], trophyCount: 12, earnedRatio: 1, daysAgo: 2, hasPlatinum: true },
  { id: 'psn:neon', title: 'Neon Drift', platform: 'playstation', systems: ['PS5'], trophyCount: 10, earnedRatio: 0.6, daysAgo: 4, hasPlatinum: false },
  { id: 'switch:custom-tiny-isles', title: 'Tiny Isles', platform: 'nintendo', systems: ['Switch'], trophyCount: 11, earnedRatio: 0.64, daysAgo: 6, hasPlatinum: true },
  { id: 'psn:crown', title: 'Hollow Crown', platform: 'playstation', systems: ['PS4'], trophyCount: 14, earnedRatio: 1, daysAgo: 12, hasPlatinum: true },
  { id: 'steam:2', title: 'Pixel Gardens', platform: 'steam', systems: ['Steam'], trophyCount: 8, earnedRatio: 0.5, daysAgo: 20, hasPlatinum: true },
  { id: 'psn:skyforge', title: 'Skyforge Tactics', platform: 'playstation', systems: ['PS4', 'PS5'], trophyCount: 12, earnedRatio: 0.25, daysAgo: 40, hasPlatinum: true },
  { id: 'steam:3', title: 'Echoes of Iron', platform: 'steam', systems: ['Steam'], trophyCount: 9, earnedRatio: 0.33, daysAgo: 60, hasPlatinum: true },
  { id: 'psn:harbor', title: 'Moonlit Harbor', platform: 'playstation', systems: ['PS Vita'], trophyCount: 11, earnedRatio: 0.9, daysAgo: 90, hasPlatinum: false },
];

const now = Date.now();

function trophies(game: DemoGame): TrophyView[] {
  const earnedCount = Math.floor(game.trophyCount * game.earnedRatio);
  const regular = Array.from({ length: game.trophyCount }, (_, index): TrophyView => {
    const tier: TrophyTier = index >= game.trophyCount - 2 ? 'gold' : index >= game.trophyCount - 4 ? 'silver' : 'bronze';
    const isEarned = index < earnedCount;
    const [name, detail] = NAMES[LOCALE][index % NAMES[LOCALE].length];
    return {
      id: `${game.id}#${index}`,
      reference: { gameId: game.id, trophyId: String(index) },
      name,
      detail,
      tier,
      isHidden: !isEarned && index === game.trophyCount - 1,
      isGeneratedPlatinum: false,
      iconUrl: null,
      isEarned,
      earnedAt: isEarned ? new Date(now - (game.daysAgo + (earnedCount - index) * 1.7) * DAY).toISOString() : null,
      earnedOn: isEarned && game.systems.length > 1 ? [game.systems[index % 2]] : [],
      canToggleEarned: game.platform === 'nintendo',
      canChangeTier: game.platform === 'steam',
      hasTierOverride: false,
    };
  });
  if (!game.hasPlatinum) return regular;
  const isComplete = earnedCount === game.trophyCount;
  const platinum: TrophyView = {
    ...regular[0],
    id: `${game.id}#platinum`,
    name: game.platform === 'playstation' ? PLAYSTATION_PLATINUM[LOCALE][0] : game.title,
    detail: game.platform === 'playstation' ? PLAYSTATION_PLATINUM[LOCALE][1] : '',
    isGeneratedPlatinum: game.platform !== 'playstation',
    tier: 'platinum',
    isEarned: isComplete,
    earnedAt: isComplete ? new Date(now - game.daysAgo * DAY).toISOString() : null,
    earnedOn: [],
    canToggleEarned: false,
    canChangeTier: false,
  };
  return [platinum, ...regular];
}

function tierCounts(list: TrophyView[]): TierCount[] {
  return (['platinum', 'gold', 'silver', 'bronze'] as TrophyTier[])
    .map((tier) => {
      const ofTier = list.filter((trophy) => trophy.tier === tier);
      return { tier, earned: ofTier.filter((trophy) => trophy.isEarned).length, total: ofTier.length };
    })
    .filter((count) => count.total > 0);
}

function detail(game: DemoGame): GameDetail {
  const list = trophies(game);
  const weighted = list.filter((trophy) => trophy.tier !== 'platinum');
  const total = weighted.reduce((sum, trophy) => sum + POINTS[trophy.tier], 0);
  const earned = weighted.filter((trophy) => trophy.isEarned).reduce((sum, trophy) => sum + POINTS[trophy.tier], 0);
  const summary: GameSummary = {
    id: game.id,
    title: game.title,
    platform: game.platform,
    platforms: [game.platform],
    systems: game.systems,
    systemLabel: game.systems.join(' / '),
    coverUrl: null,
    progress: earned / total,
    tierCounts: tierCounts(list),
    isPlatinumEarned: list.some((trophy) => trophy.tier === 'platinum' && trophy.isEarned),
    isGroup: game.systems.length > 1,
    earnedCount: list.filter((trophy) => trophy.isEarned).length,
    trophyCount: list.length,
  };
  return { ...summary, isRemovable: game.platform === 'nintendo', trophies: list };
}

const details = GAMES.map(detail);
const allTrophies = details.flatMap((game) => game.trophies).filter((trophy) => trophy.isEarned);

const overview: LibraryOverview = {
  profile: {
    name: 'Alex',
    level: 36,
    levelProgress: 0.42,
    nextLevel: 37,
    levelTier: 'bronze',
    points: allTrophies.reduce((sum, trophy) => sum + POINTS[trophy.tier], 0),
    earnedByTier: (['platinum', 'gold', 'silver', 'bronze'] as TrophyTier[]).map((tier) => ({
      tier,
      count: allTrophies.filter((trophy) => trophy.tier === tier).length,
    })),
    earnedTotal: allTrophies.length,
  },
  games: details,
};

const syncedAt = new Date(now);
syncedAt.setHours(14, 5, 0, 0);

const status: SyncStatus = {
  steamLocal: { state: 'detected', gameCount: 3, player: 'Alex' },
  steamWatching: true,
  steamWeb: { state: 'idle' },
  playstation: { state: 'synced', gameCount: 6, at: syncedAt.toISOString() },
  playstationConnected: true,
  syncing: false,
};

const settings: Settings = {
  profileName: 'Alex',
  steamId64: '',
  useSteamRarity: false,
  playstationPolling: true,
  groupingMode: 'all',
  soundEnabled: true,
  toastDurationSeconds: 6,
  toastPosition: 'topRight',
  theme: 'night',
  language: LOCALE === 'fr' ? 'french' : 'english',
};

const system: SystemInfo = { dataDirectory: '~/Library/Application Support/Prisma', detectedSteamId64: '76561197972611406', launchAtLogin: true };

const catalog: CatalogItem[] = ['Aurora Quest', 'Moss Kingdom', 'Tiny Isles'].map((title) => ({
  id: title.toLowerCase().replace(/ /g, '-'),
  title,
  trophyCounts: [
    { tier: 'platinum', count: 1 },
    { tier: 'gold', count: 2 },
    { tier: 'silver', count: 3 },
    { tier: 'bronze', count: 8 },
  ],
  isAdded: title === 'Tiny Isles',
}));

const demoAnnouncement: Announcement = {
  trophyName: NAMES[LOCALE][12][0],
  tier: 'gold',
  iconUrl: null,
  gameTitle: 'Lumen Odyssey',
  slidesFromLeft: false,
};

/** Généré à partir du cœur Rust : `cargo run -p prisma-core --example themes > src/lib/demo-themes.json`. */
const gallery = demoThemes as unknown as ThemeOption[];

/** `?theme=ocean` applique un thème dans le mode démo, pour les captures. */
function demoPalette(): ThemePalette {
  const preset = parameters.get('theme');
  return (gallery.find((option) => option.preset === preset) ?? gallery[0]).palette;
}

const nothing = async () => {};

export const demoApi: Api = {
  libraryOverview: async () => overview,
  gameDetail: async (id: string) => details.find((game) => game.id === id) ?? null,
  setTrophyEarned: nothing,
  setTrophyTier: nothing,
  removeGame: nothing,
  switchCatalog: async () => catalog,
  addSwitchGame: nothing,
  addCustomSwitchGame: nothing,
  syncAll: nothing,
  importSteamHistory: nothing,
  syncPlaystation: nothing,
  connectPlaystation: nothing,
  disconnectPlaystation: nothing,
  steamApiKey: async () => '',
  setSteamApiKey: nothing,
  syncStatus: async () => status,
  settings: async () => settings,
  updateSettings: nothing,
  testNotification: nothing,
  systemInfo: async () => system,
  setLaunchAtLogin: nothing,
  themeGallery: async () => gallery,
  activeTheme: async () => demoPalette(),
  locale: async () => LOCALE,
};

const unlisten: UnlistenFn = () => {};

export const demoEvents: Events = {
  onLibraryChanged: async () => unlisten,
  onStatusChanged: async () => unlisten,
  onNavigate: async () => unlisten,
  onToastShow: async (handler) => {
    handler(demoAnnouncement);
    return unlisten;
  },
  onToastHide: async () => unlisten,
  onThemeChanged: async () => unlisten,
  onLanguageChanged: async () => unlisten,
};
