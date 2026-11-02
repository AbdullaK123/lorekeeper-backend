// Endpoint presets keep the testing console aligned with the backend route contract.
export const operations = [
  { id: "graph", label: "Graph page", method: "GET", path: "/graph", query: ["limit", "cursor"] },
  { id: "entities", label: "Entity page", method: "GET", path: "/entities", query: ["limit", "cursor", "kind"] },
  { id: "search", label: "Search entities", method: "GET", path: "/entities/search", query: ["query", "limit"] },
  { id: "entity", label: "Entity details", method: "GET", path: "/entities/:entityId", fields: ["entityId"] },
  { id: "neighbors", label: "Entity connections", method: "GET", path: "/entities/:entityId/neighbors", fields: ["entityId"], query: ["limit", "cursor", "kind", "rel_type"] },
  { id: "relationship", label: "Relationship details", method: "GET", path: "/relationships/:relationshipId", fields: ["relationshipId"] },
  { id: "conflicts", label: "Conflict page", method: "GET", path: "/conflicts", query: ["limit", "cursor", "resolved"] },
  { id: "conflict", label: "Conflict details", method: "GET", path: "/conflicts/:conflictId", fields: ["conflictId"] },
  { id: "resolve", label: "Resolve conflict", method: "POST", path: "/conflicts/:conflictId/resolve", fields: ["conflictId"], decision: true },
  { id: "message", label: "Source message", method: "GET", path: "/messages/:messageId", fields: ["messageId"] },
  { id: "status", label: "Research status", method: "GET", path: "/messages/:messageId/research", fields: ["messageId"] },
  { id: "retry", label: "Retry research", method: "POST", path: "/messages/:messageId/research/retry", fields: ["messageId"] },
  { id: "conversation", label: "Saved conversation", method: "GET", path: "/conversations/:conversationId", fields: ["conversationId"] },
  { id: "deleteConversation", label: "Delete conversation", method: "DELETE", path: "/conversations/:conversationId", fields: ["conversationId"] },
];

export function buildRequest(operation, worldId, values) {
  if (!worldId) throw new Error("Select a world first.");
  let path = `/world/${encodeURIComponent(worldId)}${operation.path}`;
  for (const field of operation.fields || []) {
    const value = values[field]?.trim();
    if (!value) throw new Error(`Enter ${field}.`);
    path = path.replace(`:${field}`, encodeURIComponent(value));
  }
  const params = new URLSearchParams();
  for (const field of operation.query || []) {
    if (values[field]?.trim()) params.set(field, values[field].trim());
  }
  if (params.size) path += `?${params}`;
  return { method: operation.method, path,
    body: operation.decision ? { decision: values.decision || "keep_existing" } : undefined };
}
