use std::collections::HashMap;

/// Informations mémorisées entre deux synchronisations pour éviter de tout relire.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SourceCache {
    /// Date de dernière mise à jour de chaque liste de trophées PlayStation, par identifiant de communication.
    pub playstation_revisions: HashMap<String, String>,
    /// Pourcentage de joueurs ayant obtenu chaque succès, par appid puis par nom d'API.
    pub steam_rarity: HashMap<u32, HashMap<String, f64>>,
    pub steam_titles: HashMap<u32, String>,
}
