use std::collections::{BTreeSet, HashMap, HashSet};

use serde::{Deserialize, Serialize};

use super::title;
use crate::domain::{Game, GameId, Platform, Trophy, TrophyRef, TrophyTier, system};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GroupingMode {
    #[default]
    All,
    Playstation,
    None,
}

impl GroupingMode {
    fn includes(self, platform: Platform) -> bool {
        match self {
            GroupingMode::All => true,
            GroupingMode::Playstation => platform == Platform::Playstation,
            GroupingMode::None => false,
        }
    }
}

/// La version la mieux classée fournit titre, jaquette et icônes ; Steam et Switch passent
/// en dernier pour que les grades natifs PlayStation l'emportent.
const SYSTEM_PRIORITY: [&str; 6] = [system::PS5, system::PS4, system::PS3, system::PS_VITA, system::STEAM, system::SWITCH];

/// Réunit à l'affichage les versions d'un même jeu (PS3, PS4, PS5, PS Vita, Steam et Switch).
/// Les versions restent stockées séparément.
#[derive(Debug, Default)]
pub struct GameGrouper {
    /// La normalisation d'un titre est coûteuse et la bibliothèque est regroupée à chaque modification.
    matching_keys: HashMap<String, String>,
}

impl GameGrouper {
    pub fn group<'a>(&mut self, games: impl IntoIterator<Item = &'a Game>, mode: GroupingMode) -> Vec<Game> {
        let mut entries = Vec::new();
        let mut versions_by_key: HashMap<String, Vec<&Game>> = HashMap::new();
        for game in games {
            if mode.includes(game.platform) {
                versions_by_key.entry(self.matching_key(&game.title)).or_default().push(game);
            } else {
                entries.push(game.clone());
            }
        }
        for (key, versions) in versions_by_key {
            if versions.len() > 1 && !key.is_empty() {
                entries.push(combine(versions, &key));
            } else {
                entries.extend(versions.into_iter().cloned());
            }
        }
        entries
    }

    fn matching_key(&mut self, title: &str) -> String {
        self.matching_keys.entry(title.to_owned()).or_insert_with(|| title::matching_key(title)).clone()
    }
}

fn combine(mut versions: Vec<&Game>, key: &str) -> Game {
    versions.sort_by_key(|game| game_priority(game));
    let primary = versions[0];
    let title = title::display_title(&primary.title);

    let mut merger = TrophyMerger::new(title.clone());
    for version in &versions {
        for trophy in &version.trophies {
            merger.add(trophy, version);
        }
    }

    let systems: BTreeSet<&String> = versions.iter().flat_map(|version| &version.systems).collect();
    let mut systems: Vec<String> = systems.into_iter().cloned().collect();
    systems.sort_by_key(|system| display_order(system));

    Game {
        id: GameId::group(key),
        platform: primary.platform,
        title,
        systems,
        cover_url: primary.cover_url.clone(),
        trophies: merger.into_trophies(),
        is_group: true,
    }
}

fn system_priority(system: &str) -> usize {
    SYSTEM_PRIORITY.iter().position(|candidate| *candidate == system).unwrap_or(SYSTEM_PRIORITY.len())
}

fn game_priority(game: &Game) -> usize {
    game.systems.iter().map(|system| system_priority(system)).min().unwrap_or(SYSTEM_PRIORITY.len())
}

/// PlayStation de la plus ancienne à la plus récente, puis Steam et Switch : « PS4 / PS5 / Steam / Switch ».
fn display_order(system: &str) -> (u8, isize) {
    let priority = system_priority(system) as isize;
    if system == system::STEAM || system == system::SWITCH { (1, priority) } else { (0, -priority) }
}

/// Fusionne les trophées de même nom ; un jeu n'a qu'un seul platine.
struct TrophyMerger {
    group_title: String,
    trophies: Vec<Trophy>,
    index_by_key: HashMap<String, usize>,
    contributors_by_key: HashMap<String, HashSet<GameId>>,
}

impl TrophyMerger {
    fn new(group_title: String) -> Self {
        TrophyMerger { group_title, trophies: Vec::new(), index_by_key: HashMap::new(), contributors_by_key: HashMap::new() }
    }

    fn into_trophies(self) -> Vec<Trophy> {
        self.trophies
    }

    fn add(&mut self, trophy: &Trophy, version: &Game) {
        let key = self.merge_key(trophy, version);
        self.contributors_by_key.entry(key.clone()).or_default().insert(version.id.clone());
        let system = version.systems.join("/");

        match self.index_by_key.get(&key) {
            Some(&index) => absorb(&mut self.trophies[index], trophy, system),
            None => {
                self.index_by_key.insert(key, self.trophies.len());
                self.trophies.push(self.grouped_copy(trophy, version, system));
            }
        }
    }

    fn merge_key(&self, trophy: &Trophy, version: &Game) -> String {
        let own_key = format!("{}#{}", version.id, trophy.id);
        if trophy.tier == TrophyTier::Platinum {
            return "platinum".into();
        }
        if trophy.name.is_empty() {
            return own_key;
        }
        let key = title::comparable_name(&trophy.name);
        let is_homonym_in_same_version = self.contributors_by_key.get(&key).is_some_and(|ids| ids.contains(&version.id));
        if is_homonym_in_same_version { format!("{key}#{own_key}") } else { key }
    }

    fn grouped_copy(&self, trophy: &Trophy, version: &Game, system: String) -> Trophy {
        let mut copy = trophy.clone();
        copy.id = format!("{}#{}", version.id, trophy.id);
        copy.origin = Some(TrophyRef::new(version.id.clone(), trophy.id.clone()));
        copy.earned_on = if trophy.is_earned { vec![system] } else { Vec::new() };
        if copy.is_generated_platinum {
            copy.name = self.group_title.clone();
        }
        copy
    }
}

fn absorb(merged: &mut Trophy, other: &Trophy, system: String) {
    merged.is_hidden = merged.is_hidden && other.is_hidden;
    if merged.detail.is_empty() {
        merged.detail = other.detail.clone();
    }
    if !other.is_earned {
        return;
    }
    merged.is_earned = true;
    merged.earned_on.push(system);
    if let Some(date) = other.earned_at {
        if merged.earned_at.is_none_or(|current| date < current) {
            merged.earned_at = Some(date);
        }
    }
}
