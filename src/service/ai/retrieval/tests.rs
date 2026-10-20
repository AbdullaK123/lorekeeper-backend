use super::*;
use serde_json::json;
use std::collections::HashMap;

fn entity(id: &str, name: &str, kind: &str, description: &str) -> Entity {
    Entity {
        id: id.into(),
        world_id: "w".into(),
        name: name.into(),
        name_key: name.to_lowercase(),
        aliases: vec![],
        kind: vec![kind.into()],
        description: description.into(),
        properties: HashMap::new(),
    }
}

fn rel(from: &str, to: &str, rel_type: &str) -> Relationship {
    Relationship {
        id: format!("{from}-{to}"),
        world_id: "w".into(),
        from_id: from.into(),
        to_id: to.into(),
        rel_type: rel_type.into(),
        description: String::new(),
        properties: HashMap::new(),
        source_conversation: "c".into(),
    }
}

fn ids(entities: &[Entity]) -> Vec<&str> {
    entities.iter().map(|e| e.id.as_str()).collect()
}

// --- rrf ---

#[test]
fn rrf_puts_an_entity_found_by_both_searches_first() {
    let a = entity("a", "A", "x", "");
    let b = entity("b", "B", "x", "");
    let c = entity("c", "C", "x", "");

    let merged = RetrievalService::rrf(&[vec![a.clone(), b.clone()], vec![c, b]], 60.0);

    assert_eq!(merged[0].id, "b");
}

#[test]
fn rrf_keeps_entities_found_by_only_one_search() {
    let words_only = entity("w", "W", "x", "");
    let meaning_only = entity("m", "M", "x", "");

    let merged = RetrievalService::rrf(&[vec![words_only], vec![meaning_only]], 60.0);

    let mut got = ids(&merged);
    got.sort();
    assert_eq!(got, vec!["m", "w"]);
}

#[test]
fn rrf_returns_each_entity_once() {
    let a = entity("a", "A", "x", "");

    let merged = RetrievalService::rrf(&[vec![a.clone()], vec![a]], 60.0);

    assert_eq!(merged.len(), 1);
}

#[test]
fn rrf_handles_no_lists_and_empty_lists() {
    assert!(RetrievalService::rrf(&[], 60.0).is_empty());
    assert!(RetrievalService::rrf(&[vec![], vec![]], 60.0).is_empty());
}

// --- format_hits ---

#[test]
fn format_hits_says_so_when_nothing_matched() {
    assert_eq!(format_hits(&[]), "No matching lore found in this world.");
}

#[test]
fn format_hits_draws_outgoing_and_incoming_arrows() {
    let kael = entity("k", "Kael", "character", "A warrior from the Ashlands.");
    let conclave = entity("c", "Iron Conclave", "faction", "A militant order.");
    let mags = entity("m", "Old Mags", "character", "");

    let hit = SearchHit {
        entity: kael,
        neighbors: vec![
            (conclave, rel("k", "c", "MEMBER_OF")),
            (mags, rel("m", "k", "MENTORED")),
        ],
    };

    assert_eq!(
        format_hits(&[hit]),
        "Kael (character): A warrior from the Ashlands.\n\
         \x20 - MEMBER_OF → Iron Conclave (faction): A militant order.\n\
         \x20 - ← MENTORED from Old Mags (character)"
    );
}

#[test]
fn format_hits_lists_properties_without_json_quotes() {
    let mut tavern = entity("t", "Rusty Anchor", "location", "A tavern.");
    tavern.properties.insert("owner".into(), json!("Old Mags"));
    tavern.properties.insert("rooms".into(), json!(12));

    let out = format_hits(&[SearchHit { entity: tavern, neighbors: vec![] }]);

    assert!(out.contains("  Properties: owner: Old Mags; rooms: 12"), "{out}");
}

#[test]
fn format_hits_separates_hits_with_a_blank_line() {
    let a = SearchHit { entity: entity("a", "A", "x", ""), neighbors: vec![] };
    let b = SearchHit { entity: entity("b", "B", "x", ""), neighbors: vec![] };

    assert_eq!(format_hits(&[a, b]), "A (x)\n\nB (x)");
}
