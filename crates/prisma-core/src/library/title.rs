use std::sync::LazyLock;

use regex::Regex;

use crate::support::text::fold;

static PLATFORM_WORDS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b(ps ?vita|ps ?[345]|playstation ?[345]|trophies|trophees|trophy)\b").expect("expression valide"));
static EDITION_WORDS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b(game of the year|goty|definitive|complete|deluxe|ultimate|enhanced|directors? cut|director s cut|edition)\b")
        .expect("expression valide")
});
static NON_WORD: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[^\p{L}\p{N}]+").expect("expression valide"));
static TRADEMARKS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[™®]").expect("expression valide"));
static PLATFORM_MENTION: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\s*[(\[]?\b(PS ?Vita|PS ?[345]|PlayStation ?[345])\b[)\]]?").expect("expression valide"));
static TRAILING_SEPARATORS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[\s:–-]+$").expect("expression valide"));

/// Clé indépendante de la plateforme et de l'édition :
/// « Ghost of Tsushima™ (PS5) » et « Ghost of Tsushima DIRECTOR'S CUT » donnent « ghost of tsushima ».
pub(crate) fn matching_key(title: &str) -> String {
    let folded = fold(title);
    let without_platform = PLATFORM_WORDS.replace_all(&folded, " ");
    let without_edition = EDITION_WORDS.replace_all(&words_only(&without_platform), " ").into_owned();
    words_only(&without_edition)
}

/// Titre sans mention de plateforme : « Ghost of Tsushima™ (PS5) » donne « Ghost of Tsushima ».
pub(crate) fn display_title(title: &str) -> String {
    let cleaned = TRADEMARKS.replace_all(title, "");
    let cleaned = PLATFORM_MENTION.replace_all(&cleaned, "");
    let cleaned = TRAILING_SEPARATORS.replace_all(&cleaned, "");
    if cleaned.is_empty() { title.to_owned() } else { cleaned.into_owned() }
}

/// Nom comparable sans tenir compte des accents, de la casse ni de la ponctuation.
pub(crate) fn comparable_name(name: &str) -> String {
    words_only(&fold(name))
}

fn words_only(text: &str) -> String {
    NON_WORD.replace_all(text, " ").trim().to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matching_key_ignores_platform_and_edition() {
        for (title, key) in [
            ("Ghost of Tsushima™ (PS5)", "ghost of tsushima"),
            ("Ghost of Tsushima DIRECTOR'S CUT", "ghost of tsushima"),
            ("The Last of Us Remastered Trophies", "the last of us remastered"),
            ("Élden Ring – Édition Deluxe", "elden ring"),
        ] {
            assert_eq!(matching_key(title), key, "{title}");
        }
    }

    #[test]
    fn display_title_drops_platform_mention() {
        assert_eq!(display_title("Ghost of Tsushima™ (PS5)"), "Ghost of Tsushima");
        assert_eq!(display_title("PS5"), "PS5");
    }
}
