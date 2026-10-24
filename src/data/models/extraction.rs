use rig::schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A saved relationship the writer explicitly takes back in this message, e.g. "Kael isn't from the desert" or "they aren't siblings after all".
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RetractedRelationship {
    /// Name of the source entity of the old relationship, exactly as it appears in existing_entities.
    pub from_name: String,
    /// Name of the target entity of the old relationship, exactly as it appears in existing_entities.
    pub to_name: String,
    /// Label of the old relationship, read from source to target, e.g. FROM, SIBLING_OF. Must be the label the old relationship was saved with, not a new one.
    pub rel_type: String,
}

/// A worldbuilding element stated in the writer's message: a character, location, faction, artifact, species, magic system, event, concept, title, or organization.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExtractedEntity {
    /// Canonical name: the most complete or formal version, without a leading article, e.g. "Iron Conclave", not "the Conclave".
    pub name: String,
    /// Other names or shorthand the writer uses for this entity, e.g. ["the Conclave"]. Empty if none.
    pub aliases: Vec<String>,
    /// Entity types; an entity can have several, e.g. ["character", "deity"]. Use lowercase snake_case: character, location, faction, artifact, species, concept, event, magic_system, etc.
    pub kind: Vec<String>,
    /// Concise reference-style summary of what this message says about the entity. Not a verbatim quote. For an update, include only the new information.
    pub description: String,
    /// Structured attributes explicitly stated or directly implied, e.g. {"population": "~50,000", "climate": "arctic"}. Keep units as stated. Empty if none. For an update, include only new or changed attributes.
    pub properties: HashMap<String, serde_json::Value>,
    /// The ID of the matching entity from existing_entities when this entity matches one by name, alias, or strong contextual signal; null for a new entity.
    pub existing_id: Option<String>,
}

/// A connection between two entities stated or directly implied in the writer's message.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExtractedRelationship {
    /// Name of the source entity. Must exactly match the name of an entity in this extraction or in existing_entities.
    pub from_name: String,
    /// Name of the target entity. Must exactly match the name of an entity in this extraction or in existing_entities.
    pub to_name: String,
    /// Relationship label as a SCREAMING_SNAKE_CASE verb phrase, read from source to target, e.g. LEADS, MEMBER_OF, LOCATED_IN, BORN_IN, FORMER_MEMBER_OF.
    pub rel_type: String,
    /// Context about the relationship, including any temporal or situational qualifiers, e.g. "Was expelled after attempting to fulfill a prophecy."
    pub description: String,
    /// Structured attributes of the relationship, e.g. {"bidirectional": true}. Empty if none.
    pub properties: HashMap<String, serde_json::Value>,
}

/// A contradiction between newly extracted information and an entity already in the knowledge graph.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExtractedConflict {
    /// ID of the existing entity the conflict is on.
    pub entity_id: String,
    /// The conflicting field, as a path: "kind", "description", or "properties.<key>", e.g. "properties.origin".
    pub field: String,
    /// The value currently stored in the graph for this field.
    pub existing_value: serde_json::Value,
    /// The value stated in the new message for this field.
    pub new_value: serde_json::Value,
    /// "hard" when both values cannot be true at once; "soft" when the tension might be intentional, an oversight, or narrative progression.
    pub severity: String,
    /// Human-readable explanation of the contradiction, shown to the writer. Note it if the message makes the change clearly intentional.
    pub explanation: String,
}

/// All entities and relationships extracted from one writer message.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GraphExtraction {
    /// Entities stated in the message, new or updated. Empty when the message has no extractable worldbuilding content.
    pub entities: Vec<ExtractedEntity>,
    /// Relationships between entities stated in the message. Empty when there are none.
    pub relationships: Vec<ExtractedRelationship>,
    /// Old relationships the writer clearly takes back in this message. Empty unless the message explicitly retracts a connection; a new relationship alone is not a retraction.
    pub retracted_relationships: Vec<RetractedRelationship>,
}

/// All conflicts found between one extracted entity and its existing version in the graph.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ConflictExtraction {
    /// One entry per conflicting field. Empty when there are no conflicts; additions and elaborations are not conflicts.
    pub conflicts: Vec<ExtractedConflict>,
}
