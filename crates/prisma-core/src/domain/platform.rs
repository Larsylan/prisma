use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    Playstation,
    Steam,
    Nintendo,
}

impl Platform {
    pub const ALL: [Platform; 3] = [Platform::Playstation, Platform::Steam, Platform::Nintendo];

    /// Plateforme d'un système tel qu'affiché (« PS5 », « Steam », « Switch »…).
    pub fn from_system(name: &str) -> Option<Self> {
        match name {
            system::STEAM => Some(Platform::Steam),
            system::SWITCH => Some(Platform::Nintendo),
            _ if system::PLAYSTATION.contains(&name) => Some(Platform::Playstation),
            _ => None,
        }
    }
}

pub mod system {
    pub const PS5: &str = "PS5";
    pub const PS4: &str = "PS4";
    pub const PS3: &str = "PS3";
    pub const PS_VITA: &str = "PS Vita";
    pub const STEAM: &str = "Steam";
    pub const SWITCH: &str = "Switch";

    pub(crate) const PLAYSTATION: [&str; 4] = [PS5, PS4, PS3, PS_VITA];
}
