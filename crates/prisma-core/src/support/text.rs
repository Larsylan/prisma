use std::sync::LazyLock;

use regex::Regex;
use unicode_normalization::{UnicodeNormalization, char::is_combining_mark};

static TRADEMARK_SIGNS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[™®©]").expect("expression valide"));
static NON_ALPHANUMERIC: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[^a-z0-9]+").expect("expression valide"));

/// Texte comparable : sans accents, en minuscules, en caractères de largeur normale et sans ™ ® ©.
pub(crate) fn fold(text: &str) -> String {
    let without_signs = TRADEMARK_SIGNS.replace_all(text, "");
    without_signs.nfkd().filter(|character| !is_combining_mark(*character)).collect::<String>().to_lowercase()
}

/// « Pèlerin d'Hyrule » donne « pelerin-d-hyrule ».
pub(crate) fn slug(text: &str) -> String {
    NON_ALPHANUMERIC.replace_all(&fold(text), "-").trim_matches('-').to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fold_removes_accents_case_and_signs() {
        assert_eq!(fold("Élden™ RING"), "elden ring");
    }

    #[test]
    fn slug_keeps_letters_and_digits() {
        assert_eq!(slug("Pèlerin d'Hyrule"), "pelerin-d-hyrule");
        assert_eq!(slug("100 % !"), "100");
    }
}
