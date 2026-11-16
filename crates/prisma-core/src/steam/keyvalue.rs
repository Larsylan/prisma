//! Format KeyValues de Valve : binaire dans `appcache/stats`, texte dans les fichiers `.vdf` et `.acf`.

/// Arbre KeyValues. Les clés ne tiennent pas compte de la casse.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum KeyValue {
    Object(Vec<(String, KeyValue)>),
    Text(String),
    Integer(i64),
    Float(f64),
}

impl KeyValue {
    pub(crate) fn get(&self, key: &str) -> Option<&KeyValue> {
        self.entries().iter().find(|(candidate, _)| candidate.eq_ignore_ascii_case(key)).map(|(_, value)| value)
    }

    pub(crate) fn entries(&self) -> &[(String, KeyValue)] {
        match self {
            KeyValue::Object(entries) => entries,
            _ => &[],
        }
    }

    pub(crate) fn text(&self) -> Option<String> {
        match self {
            KeyValue::Text(text) => Some(text.clone()),
            KeyValue::Integer(value) => Some(value.to_string()),
            KeyValue::Float(value) => Some(value.to_string()),
            KeyValue::Object(_) => None,
        }
    }

    pub(crate) fn integer(&self) -> Option<i64> {
        match self {
            KeyValue::Integer(value) => Some(*value),
            KeyValue::Text(text) => text.parse().ok(),
            KeyValue::Float(value) => Some(*value as i64),
            KeyValue::Object(_) => None,
        }
    }
}

#[derive(Debug, PartialEq, thiserror::Error)]
pub(crate) enum ParseError {
    #[error("fichier tronqué")]
    Truncated,
    #[error("type de valeur inconnu : {0:#04x}")]
    UnknownType(u8),
}

pub(crate) fn parse_binary(bytes: &[u8]) -> Result<KeyValue, ParseError> {
    BinaryReader { bytes, position: 0 }.read_object()
}

struct BinaryReader<'a> {
    bytes: &'a [u8],
    position: usize,
}

const END_TAGS: [u8; 2] = [0x08, 0x0B];

impl BinaryReader<'_> {
    fn read_object(&mut self) -> Result<KeyValue, ParseError> {
        let mut entries = Vec::new();
        while let Some(&tag) = self.bytes.get(self.position) {
            self.position += 1;
            if END_TAGS.contains(&tag) {
                break;
            }
            let key = self.read_c_string()?;
            let value = match tag {
                0x00 => self.read_object()?,
                0x01 => KeyValue::Text(self.read_c_string()?),
                0x02 | 0x04 | 0x06 => KeyValue::Integer(i64::from(i32::from_le_bytes(self.read_array()?))),
                0x03 => KeyValue::Float(f64::from(f32::from_le_bytes(self.read_array()?))),
                0x07 => KeyValue::Integer(u64::from_le_bytes(self.read_array()?) as i64),
                0x0A => KeyValue::Integer(i64::from_le_bytes(self.read_array()?)),
                unknown => return Err(ParseError::UnknownType(unknown)),
            };
            entries.push((key, value));
        }
        Ok(KeyValue::Object(entries))
    }

    fn read_c_string(&mut self) -> Result<String, ParseError> {
        let remaining = &self.bytes[self.position..];
        let length = remaining.iter().position(|&byte| byte == 0).ok_or(ParseError::Truncated)?;
        self.position += length + 1;
        Ok(String::from_utf8_lossy(&remaining[..length]).into_owned())
    }

    fn read_array<const N: usize>(&mut self) -> Result<[u8; N], ParseError> {
        let bytes = self.bytes.get(self.position..self.position + N).ok_or(ParseError::Truncated)?;
        self.position += N;
        Ok(bytes.try_into().expect("tranche de la bonne longueur"))
    }
}

pub(crate) fn parse_text(text: &str) -> KeyValue {
    let tokens = tokenize(text);
    read_text_object(&mut tokens.into_iter())
}

#[derive(Debug, PartialEq)]
enum Token {
    Text(String),
    Open,
    Close,
}

fn read_text_object(tokens: &mut impl Iterator<Item = Token>) -> KeyValue {
    let mut entries = Vec::new();
    while let Some(token) = tokens.next() {
        let Token::Text(key) = token else {
            if token == Token::Close {
                break;
            }
            continue;
        };
        match tokens.next() {
            Some(Token::Open) => entries.push((key, read_text_object(tokens))),
            Some(Token::Text(value)) => entries.push((key, KeyValue::Text(value))),
            Some(Token::Close) | None => break,
        }
    }
    KeyValue::Object(entries)
}

fn tokenize(text: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut characters = text.chars().peekable();
    while let Some(&character) = characters.peek() {
        match character {
            '{' => {
                characters.next();
                tokens.push(Token::Open);
            }
            '}' => {
                characters.next();
                tokens.push(Token::Close);
            }
            '"' => {
                characters.next();
                tokens.push(Token::Text(read_quoted(&mut characters)));
            }
            '/' if text_starts_comment(&characters) => skip_line(&mut characters),
            _ if character.is_whitespace() => {
                characters.next();
            }
            _ => tokens.push(Token::Text(read_bare(&mut characters))),
        }
    }
    tokens
}

type Characters<'a> = std::iter::Peekable<std::str::Chars<'a>>;

fn text_starts_comment(characters: &Characters<'_>) -> bool {
    let mut lookahead = characters.clone();
    lookahead.next();
    lookahead.next() == Some('/')
}

fn read_quoted(characters: &mut Characters<'_>) -> String {
    let mut text = String::new();
    while let Some(character) = characters.next() {
        match character {
            '"' => break,
            '\\' => match characters.next() {
                Some('n') => text.push('\n'),
                Some('t') => text.push('\t'),
                Some(escaped) => text.push(escaped),
                None => break,
            },
            _ => text.push(character),
        }
    }
    text
}

fn read_bare(characters: &mut Characters<'_>) -> String {
    let mut text = String::new();
    while let Some(&character) = characters.peek() {
        if character.is_whitespace() || matches!(character, '{' | '}' | '"') {
            break;
        }
        text.push(character);
        characters.next();
    }
    text
}

fn skip_line(characters: &mut Characters<'_>) {
    for character in characters.by_ref() {
        if character == '\n' {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::BinaryWriter;

    #[test]
    fn reads_binary_objects() {
        let mut writer = BinaryWriter::default();
        writer.begin("root").text("Name", "Borderlands").int32("count", -2).end().end();

        let root = parse_binary(&writer.bytes).expect("fichier valide");

        assert_eq!(root.get("ROOT").and_then(|root| root.get("name")).and_then(KeyValue::text).as_deref(), Some("Borderlands"));
        assert_eq!(root.get("root").and_then(|root| root.get("count")).and_then(KeyValue::integer), Some(-2));
    }

    #[test]
    fn rejects_truncated_or_unknown_data() {
        assert_eq!(parse_binary(&[0x01, 0x61]), Err(ParseError::Truncated));
        assert_eq!(parse_binary(&[0x42, 0x61, 0x00]), Err(ParseError::UnknownType(0x42)));
    }

    #[test]
    fn reads_text_vdf() {
        let text = r#"
        // Commentaire
        "users"
        {
            "76561197972611406"
            {
                "PersonaName"   "Alex \"le\" joueur"
                "MostRecent"    "1"
            }
        }
        "#;

        let user = parse_text(text).get("users").and_then(|users| users.get("76561197972611406")).cloned();

        assert_eq!(user.as_ref().and_then(|user| user.get("PersonaName")).and_then(KeyValue::text).as_deref(), Some("Alex \"le\" joueur"));
        assert_eq!(user.as_ref().and_then(|user| user.get("mostrecent")).and_then(KeyValue::integer), Some(1));
    }
}
