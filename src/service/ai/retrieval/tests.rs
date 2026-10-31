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
        source_message_ids: vec![],
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
        source_message_ids: vec![],
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

    let merged = RetrievalService::rrf(vec![vec![a.clone(), b.clone()], vec![c, b]], 60.0);

    assert_eq!(merged[0].id, "b");
}

#[test]
fn rrf_keeps_entities_found_by_only_one_search() {
    let words_only = entity("w", "W", "x", "");
    let meaning_only = entity("m", "M", "x", "");

    let merged = RetrievalService::rrf(vec![vec![words_only], vec![meaning_only]], 60.0);

    let mut got = ids(&merged);
    got.sort();
    assert_eq!(got, vec!["m", "w"]);
}

#[test]
fn rrf_returns_each_entity_once() {
    let a = entity("a", "A", "x", "");

    let merged = RetrievalService::rrf(vec![vec![a.clone()], vec![a]], 60.0);

    assert_eq!(merged.len(), 1);
}

#[test]
fn rrf_handles_no_lists_and_empty_lists() {
    assert!(RetrievalService::rrf(vec![], 60.0).is_empty());
    assert!(RetrievalService::rrf(vec![vec![], vec![]], 60.0).is_empty());
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
        "Kael (character) [ID: k]: A warrior from the Ashlands.\n\
         \x20 - MEMBER_OF → Iron Conclave (faction) [ID: c]: A militant order.\n\
         \x20 - ← MENTORED from Old Mags (character) [ID: m]"
    );
}

#[test]
fn format_hits_lists_properties_without_json_quotes() {
    let mut tavern = entity("t", "Rusty Anchor", "location", "A tavern.");
    tavern.properties.insert("owner".into(), json!("Old Mags"));
    tavern.properties.insert("rooms".into(), json!(12));

    let out = format_hits(&[SearchHit {
        entity: tavern,
        neighbors: vec![],
    }]);

    assert!(
        out.contains("  Properties: owner: Old Mags; rooms: 12"),
        "{out}"
    );
}

#[test]
fn format_hits_separates_hits_with_a_blank_line() {
    let a = SearchHit {
        entity: entity("a", "A", "x", ""),
        neighbors: vec![],
    };
    let b = SearchHit {
        entity: entity("b", "B", "x", ""),
        neighbors: vec![],
    };

    assert_eq!(format_hits(&[a, b]), "A (x) [ID: a]\n\nB (x) [ID: b]");
}

#[test]
fn neighbor_properties_must_reach_the_chat_agent() {
    let mut neighbor = entity("city", "Meridian", "city", "A port.");
    neighbor
        .properties
        .insert("population".into(), json!(53000));
    let hit = SearchHit {
        entity: entity("person", "Aurel", "character", ""),
        neighbors: vec![(neighbor, rel("person", "city", "LIVES_IN"))],
    };
    let output = format_hits(&[hit]);
    assert!(
        output.contains("53000"),
        "Neighbor expansion finds the city but drops its population before the agent reads it: {output}"
    );
}

#[test]
fn relationship_qualifiers_must_reach_the_chat_agent() {
    let mut relationship = rel("person", "city", "LIVES_IN");
    relationship.description = "Only during winter.".into();
    let hit = SearchHit {
        entity: entity("person", "Aurel", "character", ""),
        neighbors: vec![(entity("city", "Meridian", "city", ""), relationship)],
    };
    let output = format_hits(&[hit]);
    assert!(
        output.contains("Only during winter."),
        "Dropping a relationship qualifier turns seasonal lore into an unconditional fact: {output}"
    );
}

#[test]
fn duplicate_rows_in_one_search_must_not_outvote_two_independent_searches() {
    let duplicate = entity("duplicate", "Duplicate", "x", "");
    let consensus = entity("consensus", "Consensus", "x", "");
    let mut noisy = vec![duplicate; 10];
    noisy.push(consensus.clone());
    let ranked = RetrievalService::rrf(vec![noisy, vec![consensus]], 60.0);
    assert_eq!(
        ranked[0].id, "consensus",
        "Repeated rows from one search crowd out lore confirmed by both searches"
    );
}

#[test]
fn search_ties_keep_the_input_order() {
    let ranked = RetrievalService::rrf(
        vec![
            vec![entity("a", "A", "x", "")],
            vec![entity("b", "B", "x", "")],
        ],
        60.0,
    );
    assert_eq!(
        ids(&ranked),
        vec!["a", "b"],
        "Equal search scores make otherwise identical prompts change unpredictably"
    );
}
