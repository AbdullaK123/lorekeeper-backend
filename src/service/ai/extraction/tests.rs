use super::*;
use serde_json::json;

fn kael() -> Entity {
    Entity {
        id: "abc-123".into(),
        world_id: "world-1".into(),
        name: "Kael".into(),
        name_key: "kael".into(),
        aliases: vec!["the Kid".into()],
        kind: vec!["character".into()],
        description: "A warrior from the Ashlands.".into(),
        properties: HashMap::from([("origin".into(), json!("Ashlands"))]),
    }
}

fn update() -> ExtractedEntity {
    ExtractedEntity {
        name: "Kael".into(),
        aliases: vec![],
        kind: vec![],
        description: String::new(),
        properties: HashMap::new(),
        is_updated: true,
        existing_id: Some("abc-123".into()),
    }
}

fn merge(existing: &Entity, extracted: &ExtractedEntity, conflicts: &[&str]) -> Entity {
    let conflicts: Vec<String> = conflicts.iter().map(|s| s.to_string()).collect();
    ExtractionService::merge(existing, extracted, &conflicts)
}

#[test]
fn adds_new_property_and_keeps_old_ones() {
    let mut new = update();
    new.properties.insert("species".into(), json!("half-elf"));

    let merged = merge(&kael(), &new, &[]);

    assert_eq!(merged.properties["origin"], json!("Ashlands"));
    assert_eq!(merged.properties["species"], json!("half-elf"));
}

#[test]
fn new_property_value_wins_when_not_a_conflict() {
    let mut new = update();
    new.properties.insert("origin".into(), json!("the Ashlands, near the Spire"));

    let merged = merge(&kael(), &new, &[]);

    assert_eq!(merged.properties["origin"], json!("the Ashlands, near the Spire"));
}

#[test]
fn conflicting_property_keeps_stored_value() {
    let mut new = update();
    new.properties.insert("origin".into(), json!("Northern Wastes"));

    let merged = merge(&kael(), &new, &["properties.origin"]);

    assert_eq!(merged.properties["origin"], json!("Ashlands"));
}

#[test]
fn a_conflict_on_one_property_does_not_block_the_others() {
    let mut new = update();
    new.properties.insert("origin".into(), json!("Northern Wastes"));
    new.properties.insert("species".into(), json!("half-elf"));

    let merged = merge(&kael(), &new, &["properties.origin"]);

    assert_eq!(merged.properties["origin"], json!("Ashlands"));
    assert_eq!(merged.properties["species"], json!("half-elf"));
}

#[test]
fn aliases_are_added_without_duplicates() {
    let mut new = update();
    new.aliases = vec!["the Kid".into(), "Ashborn".into()];

    let merged = merge(&kael(), &new, &[]);

    assert_eq!(merged.aliases, vec!["the Kid".to_string(), "Ashborn".to_string()]);
}

#[test]
fn kind_is_added_without_duplicates() {
    let mut new = update();
    new.kind = vec!["character".into(), "deity".into()];

    let merged = merge(&kael(), &new, &[]);

    assert_eq!(merged.kind, vec!["character".to_string(), "deity".to_string()]);
}

#[test]
fn conflicting_kind_is_left_alone() {
    let mut new = update();
    new.kind = vec!["location".into()];

    let merged = merge(&kael(), &new, &["kind"]);

    assert_eq!(merged.kind, vec!["character".to_string()]);
}

#[test]
fn description_is_appended() {
    let mut new = update();
    new.description = "Is half-elf.".into();

    let merged = merge(&kael(), &new, &[]);

    assert_eq!(merged.description, "A warrior from the Ashlands. Is half-elf.");
}

#[test]
fn empty_description_changes_nothing() {
    let mut new = update();
    new.description = "   ".into();

    let merged = merge(&kael(), &new, &[]);

    assert_eq!(merged.description, "A warrior from the Ashlands.");
}

#[test]
fn conflicting_description_is_left_alone() {
    let mut new = update();
    new.description = "A scholar from the north.".into();

    let merged = merge(&kael(), &new, &["description"]);

    assert_eq!(merged.description, "A warrior from the Ashlands.");
}

#[test]
fn appending_to_an_empty_description_leaves_no_leading_space() {
    let mut existing = kael();
    existing.description = String::new();
    let mut new = update();
    new.description = "Is half-elf.".into();

    let merged = merge(&existing, &new, &[]);

    assert_eq!(merged.description, "Is half-elf.");
}

#[test]
fn identity_fields_never_change() {
    let mut new = update();
    new.name = "KAEL THE GREAT".into();

    let merged = merge(&kael(), &new, &[]);

    assert_eq!(merged.id, "abc-123");
    assert_eq!(merged.world_id, "world-1");
    assert_eq!(merged.name, "Kael");
    assert_eq!(merged.name_key, "kael");
}

// A retried job merges the same extraction again: nothing may pile up.
#[test]
fn merging_twice_gives_the_same_result_as_once() {
    let mut new = update();
    new.description = "Is half-elf.".into();
    new.aliases = vec!["Ashborn".into()];
    new.kind = vec!["deity".into()];
    new.properties.insert("species".into(), json!("half-elf"));

    let once = merge(&kael(), &new, &[]);
    let twice = merge(&once, &new, &[]);

    assert_eq!(twice.description, once.description);
    assert_eq!(twice.aliases, once.aliases);
    assert_eq!(twice.kind, once.kind);
    assert_eq!(twice.properties, once.properties);
}
