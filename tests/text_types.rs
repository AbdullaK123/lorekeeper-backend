#[path = "../src/shared/text.rs"]
mod text;
use text::*;

#[test]
fn names_normalize_equivalent_spellings_and_hidden_padding() {
    assert_eq!(
        WorldName::parse("  Cafe\u{301}\u{200b} \n city ")
            .unwrap()
            .as_str(),
        "Café city"
    );
    assert_eq!(WorldName::parse(" \u{200b} ").err(), Some(TextError::Blank));
}

#[test]
fn names_reject_direction_controls_and_nul() {
    assert_eq!(
        WorldName::parse("safe\u{202e}evil").err(),
        Some(TextError::DirectionControl)
    );
    assert_eq!(Description::parse("safe\0evil").err(), Some(TextError::Nul));
    assert_eq!(
        PasswordInput::parse("safe\0evil").err(),
        Some(TextError::Nul)
    );
}

#[test]
fn raw_padding_cannot_evade_length_limit() {
    assert_eq!(
        WorldName::parse(" ".repeat(256)).err(),
        Some(TextError::TooLong { max: 255 })
    );
    assert!(
        WorldName::parse("界".repeat(255)).is_ok(),
        "Length counts characters, not UTF-8 bytes"
    );
}

#[test]
fn prose_preserves_lines_and_language_joiners() {
    let prose = ChatContent::parse("  first\r\nsecond\rthird\u{200d}  ").unwrap();
    assert_eq!(prose.as_str(), "first\nsecond\nthird\u{200d}");
    assert!(Description::parse("  ").is_ok());
    assert_eq!(ChatContent::parse(" \n ").err(), Some(TextError::Blank));
}

#[test]
fn passwords_preserve_exact_contents() {
    let raw = "  Cafe\u{301}\r\n\u{200d}  ";
    assert_eq!(PasswordInput::parse(raw).unwrap().as_str(), raw);
}

#[test]
fn json_cannot_bypass_validation() {
    assert!(serde_json::from_str::<WorldName>(r#""   ""#).is_err());
    assert!(serde_json::from_str::<WorldName>("123").is_err());
    let name: WorldName = serde_json::from_str(r#""  New   World  ""#).unwrap();
    assert_eq!(serde_json::to_string(&name).unwrap(), r#""New World""#);
}

#[test]
fn field_types_enforce_their_limits() {
    assert!(Username::parse("x".repeat(101)).is_err());
    assert!(ConversationTitle::parse("x".repeat(256)).is_err());
    assert!(EntityName::parse("x".repeat(256)).is_err());
    assert!(SearchQuery::parse("x".repeat(501)).is_err());
    assert!(Description::parse("x".repeat(10_001)).is_err());
    assert!(ChatContent::parse("x".repeat(10_001)).is_err());
    assert!(PasswordInput::parse("x".repeat(101)).is_err());
}

#[test]
fn password_debug_output_hides_credentials() {
    let password = PasswordInput::parse("Secret-123!").unwrap();
    assert_eq!(format!("{password:?}"), "PasswordInput([REDACTED])");
}
