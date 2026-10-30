//! Text validation is not SQL escaping, HTML sanitization, or intent detection.
//! Length limits count Unicode characters, before and after normalization.
use serde::{Deserialize, Serialize};
use unicode_normalization::UnicodeNormalization;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum TextError {
    #[error("Text must contain at most {max} characters")]
    TooLong { max: usize },
    #[error("NUL characters are not allowed")]
    Nul,
    #[error("Text direction controls are not allowed in names or search queries")]
    DirectionControl,
    #[error("Text must not be blank")]
    Blank,
}

#[derive(Clone, Copy)]
enum TextKind {
    SingleLine,
    Multiline,
    Opaque,
}

fn is_direction_control(ch: char) -> bool {
    matches!(ch, '\u{061c}' | '\u{200e}' | '\u{200f}' | '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')
}

fn is_invisible(ch: char) -> bool {
    matches!(
        ch,
        '\u{00ad}' | '\u{034f}' | '\u{180e}' | '\u{200b}' | '\u{2060}' | '\u{feff}'
    )
}

fn check_length(text: &str, max: usize) -> Result<(), TextError> {
    if text.chars().take(max + 1).count() > max {
        return Err(TextError::TooLong { max });
    }
    Ok(())
}

fn prepare(
    text: String,
    max: usize,
    kind: TextKind,
    allow_empty: bool,
) -> Result<String, TextError> {
    check_length(&text, max)?;
    if text.contains('\0') {
        return Err(TextError::Nul);
    }
    if matches!(kind, TextKind::Opaque) {
        return if text.is_empty() {
            Err(TextError::Blank)
        } else {
            Ok(text)
        };
    }
    if matches!(kind, TextKind::SingleLine) && text.chars().any(is_direction_control) {
        return Err(TextError::DirectionControl);
    }

    let text = text.replace("\r\n", "\n").replace('\r', "\n");
    let mut cleaned = String::new();
    for ch in text.nfc() {
        if is_invisible(ch) || (ch.is_control() && !matches!(ch, '\t' | '\n')) {
            continue;
        }
        if matches!(kind, TextKind::SingleLine) && matches!(ch, '\u{200c}' | '\u{200d}') {
            continue;
        }
        cleaned.push(if matches!(ch, '\u{2028}' | '\u{2029}') {
            '\n'
        } else {
            ch
        });
    }
    let cleaned = match kind {
        TextKind::SingleLine => cleaned.split_whitespace().collect::<Vec<_>>().join(" "),
        TextKind::Multiline => cleaned.trim().to_owned(),
        TextKind::Opaque => unreachable!(),
    };
    check_length(&cleaned, max)?;
    if !allow_empty && cleaned.is_empty() {
        return Err(TextError::Blank);
    }
    Ok(cleaned)
}

// Each wrapper keeps its String private. JSON decoding uses the same parser.
macro_rules! text_type {
    ($(#[$meta:meta])* $name:ident, $max:literal, $kind:ident, $empty:literal) => {
        $(#[$meta])*
        #[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
        #[serde(try_from = "String", into = "String")]
        pub struct $name(String);

        impl $name {
            pub const MAX_LENGTH: usize = $max;

            pub fn parse(text: impl Into<String>) -> Result<Self, TextError> {
                prepare(text.into(), Self::MAX_LENGTH, TextKind::$kind, $empty).map(Self)
            }

            pub fn as_str(&self) -> &str { &self.0 }

            pub fn into_inner(self) -> String { self.0 }
        }

        impl schemars::JsonSchema for $name {
            fn schema_name() -> std::borrow::Cow<'static, str> {
                stringify!($name).into()
            }

            fn json_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
                schemars::json_schema!({
                    "type": "string",
                    "minLength": if $empty { 0 } else { 1 },
                    "maxLength": $max
                })
            }
        }

        impl TryFrom<String> for $name {
            type Error = TextError;
            fn try_from(text: String) -> Result<Self, Self::Error> { Self::parse(text) }
        }

        impl std::str::FromStr for $name {
            type Err = TextError;
            fn from_str(text: &str) -> Result<Self, Self::Err> { Self::parse(text) }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str { self.as_str() }
        }

        impl From<$name> for String {
            fn from(text: $name) -> Self { text.into_inner() }
        }
    };
}

text_type!(/// Account display name. Nonblank, single-line.
    #[derive(Debug)]
    Username, 100, SingleLine, false);
text_type!(/// World name. Nonblank, single-line.
    #[derive(Debug)]
    WorldName, 255, SingleLine, false);
text_type!(/// Conversation title. Nonblank, single-line.
    #[derive(Debug)]
    ConversationTitle, 255, SingleLine, false);
text_type!(/// Lore entity name. Nonblank, single-line.
    #[derive(Debug)]
    EntityName, 255, SingleLine, false);
text_type!(/// Search input. Nonblank, single-line.
    #[derive(Debug)]
    SearchQuery, 500, SingleLine, false);
text_type!(/// World or lore description. Empty descriptions are allowed.
    #[derive(Debug)]
    Description, 10_000, Multiline, true);
text_type!(/// User chat input. Keeps paragraph breaks and language joiners.
    #[derive(Debug)]
    ChatContent, 10_000, Multiline, false);
text_type!(/// Password input. Preserves exact contents; strength checks remain separate.
    /// Debug output is redacted. No Display implementation.
    PasswordInput, 100, Opaque, false);

impl std::fmt::Debug for PasswordInput {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("PasswordInput([REDACTED])")
    }
}
