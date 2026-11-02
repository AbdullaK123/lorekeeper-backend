import { useEffect, useRef, useState } from "react";
import { inspectRequest } from "./api.js";
import { buildRequest, operations } from "./workbench.js";

const labels = { entityId: "Entity ID", relationshipId: "Relationship ID", conflictId: "Conflict ID",
  messageId: "Message ID", conversationId: "Conversation ID", rel_type: "Relationship type",
  limit: "Limit", cursor: "Cursor", kind: "Entity kind", query: "Search text", resolved: "Resolved" };

export default function ApiWorkbench({ world, conversation, conversationId, onDeleted }) {
  const [operationId, setOperationId] = useState("graph");
  const [values, setValues] = useState({ limit: "20", decision: "keep_existing" });
  const [result, setResult] = useState(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const generation = useRef(0);
  const operation = operations.find((item) => item.id === operationId);
  useEffect(() => () => { generation.current += 1; }, []);
  const fields = { ...values, conversationId: values.conversationId ?? conversationId ?? "" };
  let preview;
  try { preview = buildRequest(operation, world.id, fields); } catch { /* Required field is still empty. */ }
  function setField(field, value) { setValues((old) => ({ ...old, [field]: value })); }
  function choose(id) {
    generation.current += 1;
    setBusy(false);
    setOperationId(id);
    setResult(null);
    setError("");
    setValues((old) => ({ ...old, cursor: "" }));
  }
  async function run(event) {
    event.preventDefault();
    const current = ++generation.current;
    setError("");
    setResult(null);
    setBusy(true);
    try {
      const request = buildRequest(operation, world.id, fields);
      const response = await inspectRequest(request.method, request.path, request.body);
      if (current !== generation.current) return;
      setResult(response);
      if (response.ok && operation.id === "deleteConversation") onDeleted(fields.conversationId);
    } catch (failure) {
      if (current === generation.current) setError(failure.message);
    } finally {
      if (current === generation.current) setBusy(false);
    }
  }
  return <main className="workbench">
    <header>
      <h1>World API workbench</h1>
      <p className="muted">Test UI requests against the real backend. Responses below are unmodified.</p>
      <p className="mono">World: {world.id}</p>
    </header>
    <form className="stack" onSubmit={run}>
      <label>Request
        <select value={operationId} onChange={(event) => choose(event.target.value)}>
          {operations.map((item) => <option key={item.id} value={item.id}>{item.method} · {item.label}</option>)}
        </select>
      </label>
      {(operation.fields || []).map((field) => <label key={field}>{labels[field]}
        <input value={fields[field] || ""} onChange={(event) => setField(field, event.target.value)} required spellCheck={false} />
      </label>)}
      {operation.fields?.includes("messageId") && <label>Use a message from the selected conversation
        <select value="" onChange={(event) => setField("messageId", event.target.value)}>
          <option value="">Choose a saved message…</option>
          {(conversation?.messages || []).map((message) => <option key={message.id} value={message.id}>
            {message.role} · {message.id} · {(message.content || "Tool call").slice(0, 60)}
          </option>)}
        </select>
      </label>}
      <div className="workbench-fields">
        {(operation.query || []).map((field) => <label key={field}>{labels[field]}
          {field === "resolved" ? <select value={fields[field] || ""} onChange={(event) => setField(field, event.target.value)}>
            <option value="">Both</option><option value="false">Unresolved</option><option value="true">Resolved</option>
          </select> : <input value={fields[field] || ""} onChange={(event) => setField(field, event.target.value)} spellCheck={false} />}
        </label>)}
      </div>
      {operation.decision && <label>Decision
        <select value={values.decision} onChange={(event) => setField("decision", event.target.value)}>
          <option value="keep_existing">Keep existing</option><option value="accept_new">Accept new</option>
        </select>
      </label>}
      {preview && <pre className="request-preview">{preview.method} {preview.path}{preview.body ? `\n${JSON.stringify(preview.body)}` : ""}</pre>}
      {operation.id === "deleteConversation" && <p className="error">This deletes the saved conversation.</p>}
      {operation.id === "resolve" && <p className="muted">This changes the stored conflict decision and may update canon.</p>}
      {operation.id === "retry" && <p className="muted">Accepted research can make paid model and embedding calls.</p>}
      <button className="primary" disabled={busy}>{busy ? "Request running…" : operation.method === "GET" ? "Send request" : operation.label}</button>
    </form>
    {error && <p className="error" role="alert">{error}</p>}
    <section aria-live="polite" aria-busy={busy}>
      {result ? <>
        <p className={result.ok ? "mono" : "error mono"}>HTTP {result.status} · {result.durationMs} ms</p>
        <pre className="request-preview">{result.method} {result.path}</pre>
        <pre className="response-body">{result.status === 204 ? "No response body" : JSON.stringify(result.data, null, 2)}</pre>
        {operation.query?.includes("cursor") && result.data?.next_cursor && <button className="ghost" onClick={() => setField("cursor", result.data.next_cursor)}>Use next cursor</button>}
        {operation.query?.includes("cursor") && result.ok && !result.data?.next_cursor && <p className="muted">End of results.</p>}
      </> : !busy && <p className="muted">Choose a request to inspect its status, timing, and JSON response.</p>}
    </section>
  </main>;
}
