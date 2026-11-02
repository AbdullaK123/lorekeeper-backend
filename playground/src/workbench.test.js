import test from "node:test";
import assert from "node:assert/strict";
import { buildRequest, operations } from "./workbench.js";
import { api, inspectRequest } from "./api.js";

const operation = (id) => operations.find((item) => item.id === id);

test("query values cannot change world scope or inject extra query keys", () => {
  const request = buildRequest(operation("search"), "world/a", { query: "harbor &limit=999", limit: "10" });
  const parsed = new URL(request.path, "http://test");
  assert.equal(parsed.pathname, "/world/world%2Fa/entities/search");
  assert.equal(parsed.searchParams.get("query"), "harbor &limit=999");
  assert.deepEqual(parsed.searchParams.getAll("limit"), ["10"]);
});

test("neighbor filters and opaque cursor survive URL encoding", () => {
  const request = buildRequest(operation("neighbors"), "world", { entityId: "entity", cursor: "a+b/=", kind: "person", rel_type: "KNOWS", limit: "2" });
  const parsed = new URL(request.path, "http://test");
  assert.equal(parsed.searchParams.get("cursor"), "a+b/=");
  assert.equal(parsed.searchParams.get("rel_type"), "KNOWS");
});

test("missing record IDs never send a request to a collection route", () => {
  assert.throws(() => buildRequest(operation("retry"), "world", {}), /messageId/);
  assert.throws(() => buildRequest(operation("graph"), "", {}), /world/);
});

test("conflict decisions and retry use explicit mutation methods", () => {
  assert.deepEqual(buildRequest(operation("resolve"), "world", { conflictId: "conflict", decision: "accept_new" }),
    { method: "POST", path: "/world/world/conflicts/conflict/resolve", body: { decision: "accept_new" } });
  assert.equal(buildRequest(operation("retry"), "world", { messageId: "source" }).path,
    "/world/world/messages/source/research/retry");
  assert.equal(buildRequest(operation("deleteConversation"), "world", { conversationId: "convo" }).method, "DELETE");
});

test("client preserves HTTP failures for inspection and includes session credentials", async () => {
  const original = globalThis.fetch;
  try {
    globalThis.fetch = async (path, options) => {
      assert.equal(path, "/api/world/w/messages/m/research/retry");
      assert.equal(options.credentials, "include");
      return new Response(JSON.stringify({ error: "conflict", message: "Already running" }), { status: 409 });
    };
    const result = await inspectRequest("POST", "/world/w/messages/m/research/retry");
    assert.equal(result.status, 409);
    assert.equal(result.data.message, "Already running");
    assert.equal(result.ok, false);
  } finally { globalThis.fetch = original; }
});

test("conversation client uses its parent world and handles empty delete responses", async () => {
  const original = globalThis.fetch;
  try {
    globalThis.fetch = async (path, options) => {
      assert.equal(path, "/api/world/w/conversations/c");
      return options.method === "DELETE" ? new Response(null, { status: 204 }) : new Response('{"id":"c"}');
    };
    assert.deepEqual(await api.conversation("w", "c"), { id: "c" });
    assert.equal(await api.deleteConversation("w", "c"), null);
  } finally { globalThis.fetch = original; }
});
