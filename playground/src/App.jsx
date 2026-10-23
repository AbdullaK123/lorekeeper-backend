import { useCallback, useEffect, useRef, useState } from "react";
import { api, streamChatTurn } from "./api.js";

/* ------------------------------------------------------------------ */
/* Auth                                                                */
/* ------------------------------------------------------------------ */

function AuthScreen({ onAuthed }) {
  const [mode, setMode] = useState("login");
  const [username, setUsername] = useState("");
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);

  async function submit(e) {
    e.preventDefault();
    setBusy(true);
    setError("");
    try {
      if (mode === "signup") await api.signup(username, email, password);
      await api.login(email, password);
      onAuthed(await api.me());
    } catch (err) {
      setError(err.message);
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="auth">
      <form className="auth-card" onSubmit={submit}>
        <div className="brand">
          <span className="brand-mark" aria-hidden>◈</span> Lorekeeper <span className="tag">playground</span>
        </div>
        <div className="seg" role="tablist">
          <button type="button" role="tab" aria-selected={mode === "login"} className={mode === "login" ? "on" : ""} onClick={() => setMode("login")}>Log in</button>
          <button type="button" role="tab" aria-selected={mode === "signup"} className={mode === "signup" ? "on" : ""} onClick={() => setMode("signup")}>Sign up</button>
        </div>
        {mode === "signup" && (
          <label>Username<input value={username} onChange={(e) => setUsername(e.target.value)} required autoComplete="username" /></label>
        )}
        <label>Email<input type="email" value={email} onChange={(e) => setEmail(e.target.value)} required autoComplete="email" /></label>
        <label>
          Password
          <input type="password" value={password} onChange={(e) => setPassword(e.target.value)} required autoComplete={mode === "signup" ? "new-password" : "current-password"} />
          {mode === "signup" && <span className="hint">8+ chars, upper + lower case, a number, a symbol.</span>}
        </label>
        {error && <p className="error" role="alert">{error}</p>}
        <button className="primary" disabled={busy}>{busy ? "…" : mode === "login" ? "Log in" : "Create account"}</button>
      </form>
    </div>
  );
}

/* ------------------------------------------------------------------ */
/* Sidebar: worlds + conversations                                     */
/* ------------------------------------------------------------------ */

function Sidebar({ user, worlds, world, conversations, conversationId, onPickWorld, onPickConversation, onCreateWorld, onCreateConversation, onLogout }) {
  const [newWorld, setNewWorld] = useState({ open: false, name: "", description: "" });
  const [newTitle, setNewTitle] = useState("");

  return (
    <aside className="sidebar">
      <div className="brand small"><span className="brand-mark" aria-hidden>◈</span> Lorekeeper</div>

      <section>
        <div className="section-head">
          <h2>Worlds</h2>
          <button className="ghost icon" aria-label="New world" onClick={() => setNewWorld((s) => ({ ...s, open: !s.open }))}>+</button>
        </div>
        {newWorld.open && (
          <form
            className="stack"
            onSubmit={async (e) => {
              e.preventDefault();
              await onCreateWorld(newWorld.name, newWorld.description);
              setNewWorld({ open: false, name: "", description: "" });
            }}
          >
            <input placeholder="Name" value={newWorld.name} onChange={(e) => setNewWorld((s) => ({ ...s, name: e.target.value }))} required />
            <textarea placeholder="Description" rows={2} value={newWorld.description} onChange={(e) => setNewWorld((s) => ({ ...s, description: e.target.value }))} />
            <button className="primary small">Create world</button>
          </form>
        )}
        <ul className="list">
          {worlds.map((w) => (
            <li key={w.id}>
              <button className={w.id === world?.id ? "item on" : "item"} onClick={() => onPickWorld(w)} title={w.description}>{w.name}</button>
            </li>
          ))}
          {worlds.length === 0 && !newWorld.open && <li className="muted">No worlds yet.</li>}
        </ul>
      </section>

      {world && (
        <section>
          <div className="section-head"><h2>Conversations</h2></div>
          <form
            className="row"
            onSubmit={async (e) => {
              e.preventDefault();
              await onCreateConversation(newTitle.trim() || "Untitled");
              setNewTitle("");
            }}
          >
            <input placeholder="New conversation…" value={newTitle} onChange={(e) => setNewTitle(e.target.value)} />
            <button className="ghost icon" aria-label="Create conversation">+</button>
          </form>
          <ul className="list">
            {conversations.map((c) => (
              <li key={c.id}>
                <button className={c.id === conversationId ? "item on" : "item"} onClick={() => onPickConversation(c.id)}>{c.title}</button>
              </li>
            ))}
          </ul>
        </section>
      )}

      <div className="sidebar-foot">
        <span className="muted" title={user.email}>{user.username}</span>
        <button className="ghost small" onClick={onLogout}>Log out</button>
      </div>
    </aside>
  );
}

/* ------------------------------------------------------------------ */
/* Turn trace: what happened during one streamed reply                  */
/* ------------------------------------------------------------------ */

function emptyTrace(userText) {
  return {
    userText,
    text: "",
    reasoning: "",
    tools: [], // { id, name, args, result }
    status: "Thinking…",
    usage: null,
    error: null,
    done: false,
    startedAt: performance.now(),
    firstTokenMs: null,
    totalMs: null,
    events: [],
  };
}

function callId(id) {
  if (!id) return "";
  return typeof id === "string" ? id : id.provider || JSON.stringify(id);
}

function applyEvent(trace, ev) {
  const t = { ...trace, events: [...trace.events, ev] };
  if (ev.kind === "error") return { ...t, error: ev.message, status: "Error", done: true };
  if (ev.kind !== "item") return t;

  const item = ev.item;
  switch (item.type) {
    case "streamAssistantItem": {
      const v = item.value || {};
      if (v.event === "text") {
        return { ...t, text: t.text + (v.text || ""), status: "Writing…", firstTokenMs: t.firstTokenMs ?? performance.now() - t.startedAt };
      }
      if (v.event === "reasoning") return { ...t, reasoning: t.reasoning + (v.text || ""), status: "Reasoning…" };
      if (v.event === "start" && v.kind === "tool_call") return { ...t, status: `Calling ${v.name}…` };
      return t;
    }
    case "toolCall": {
      const call = item.tool_call || {};
      const tool = { id: callId(call.id), name: call.function?.name, args: call.function?.arguments, result: null };
      return { ...t, tools: [...t.tools, tool] };
    }
    case "toolResult": {
      const r = item.tool_result || {};
      const text = (r.content || []).map((c) => c.text ?? JSON.stringify(c)).join("\n");
      const id = callId(r.call);
      return { ...t, status: "Thinking…", tools: t.tools.map((tool) => (tool.id === id ? { ...tool, result: text } : tool)) };
    }
    case "finalResponse":
      return { ...t, usage: item.usage, status: "Done", done: true, totalMs: performance.now() - t.startedAt };
    default:
      return t;
  }
}

/* ------------------------------------------------------------------ */
/* Chat                                                                */
/* ------------------------------------------------------------------ */

function Bubble({ role, children, live }) {
  return (
    <div className={`msg ${role}`}>
      <div className="msg-role">{role === "user" ? "You" : "Lorekeeper"}</div>
      <div className={`msg-body ${live ? "live" : ""}`}>{children}</div>
    </div>
  );
}

function Chat({ conversation, trace, onSend, streaming, onStop }) {
  const [draft, setDraft] = useState("");
  const endRef = useRef(null);
  const messages = (conversation?.messages || []).filter((m) => m.role === "user" || (m.role === "assistant" && m.content));

  useEffect(() => {
    endRef.current?.scrollIntoView({ behavior: "smooth", block: "end" });
  }, [messages.length, trace?.text, trace?.status]);

  if (!conversation) {
    return <main className="chat empty-state"><p className="muted">Pick or create a world and a conversation to start.</p></main>;
  }

  function submit(e) {
    e.preventDefault();
    const text = draft.trim();
    if (!text || streaming) return;
    setDraft("");
    onSend(text);
  }

  const liveTurn = trace && !trace.persisted;

  return (
    <main className="chat">
      <header className="chat-head">
        <h1>{conversation.title}</h1>
        <span className="muted mono">{conversation.id.slice(0, 8)}</span>
      </header>

      <div className="messages" aria-live="polite">
        {messages.map((m, i) => (
          <Bubble key={i} role={m.role}>{m.content}</Bubble>
        ))}
        {liveTurn && (
          <>
            <Bubble role="user">{trace.userText}</Bubble>
            <Bubble role="assistant" live={!trace.done}>
              {trace.text || (!trace.error && <span className="status"><span className="pulse" aria-hidden />{trace.status}</span>)}
              {trace.text && !trace.done && <span className="caret" aria-hidden />}
              {trace.error && <span className="error">{trace.error}</span>}
            </Bubble>
          </>
        )}
        <div ref={endRef} />
      </div>

      <form className="composer" onSubmit={submit}>
        <textarea
          value={draft}
          placeholder="Talk about your world…   (Enter to send, Shift+Enter for a new line)"
          rows={2}
          onChange={(e) => setDraft(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter" && !e.shiftKey) submit(e);
          }}
        />
        {streaming ? (
          <button type="button" className="ghost" onClick={onStop}>Stop</button>
        ) : (
          <button className="primary" disabled={!draft.trim()}>Send</button>
        )}
      </form>
    </main>
  );
}

/* ------------------------------------------------------------------ */
/* Inspector: everything the backend did for the last turn              */
/* ------------------------------------------------------------------ */

function Inspector({ trace }) {
  const [tab, setTab] = useState("trace");
  if (!trace) {
    return (
      <aside className="inspector">
        <h2>Inspector</h2>
        <p className="muted">Send a message to see tool calls, retrieved lore, reasoning, timing and raw SSE events.</p>
      </aside>
    );
  }
  const u = trace.usage;
  return (
    <aside className="inspector">
      <div className="section-head">
        <h2>Inspector</h2>
        <div className="seg tiny" role="tablist">
          <button role="tab" aria-selected={tab === "trace"} className={tab === "trace" ? "on" : ""} onClick={() => setTab("trace")}>Trace</button>
          <button role="tab" aria-selected={tab === "raw"} className={tab === "raw" ? "on" : ""} onClick={() => setTab("raw")}>Raw ({trace.events.length})</button>
        </div>
      </div>

      {tab === "trace" ? (
        <div className="stack">
          <dl className="stats">
            <div><dt>Status</dt><dd>{trace.status}</dd></div>
            <div><dt>First token</dt><dd>{trace.firstTokenMs != null ? `${(trace.firstTokenMs / 1000).toFixed(1)}s` : "—"}</dd></div>
            <div><dt>Total</dt><dd>{trace.totalMs != null ? `${(trace.totalMs / 1000).toFixed(1)}s` : "—"}</dd></div>
            <div><dt>Tokens</dt><dd>{u ? `${u.input_tokens} in / ${u.output_tokens} out` : "—"}</dd></div>
            <div><dt>Cost</dt><dd>{u?.cost?.total != null ? `$${u.cost.total.toFixed(5)}` : "—"}</dd></div>
          </dl>

          <details open>
            <summary>Tool calls ({trace.tools.length})</summary>
            {trace.tools.length === 0 && <p className="muted">None this turn. (The automatic search still ran before the reply.)</p>}
            {trace.tools.map((tool) => (
              <div key={tool.id} className="tool">
                <div className="mono"><strong>{tool.name}</strong>({JSON.stringify(tool.args)})</div>
                <pre>{tool.result ?? "…waiting for result"}</pre>
              </div>
            ))}
          </details>

          {trace.reasoning && (
            <details>
              <summary>Reasoning</summary>
              <pre>{trace.reasoning}</pre>
            </details>
          )}

          <p className="muted small">Extraction runs in the background after the reply is saved. Query the graph a few seconds later — see the playground README.</p>
        </div>
      ) : (
        <div className="raw">
          {trace.events.map((ev, i) => (
            <pre key={i}>{JSON.stringify(ev.kind === "item" ? ev.item : ev, null, 1)}</pre>
          ))}
        </div>
      )}
    </aside>
  );
}

/* ------------------------------------------------------------------ */
/* App                                                                 */
/* ------------------------------------------------------------------ */

export default function App() {
  const [user, setUser] = useState(undefined); // undefined = checking, null = logged out
  const [worlds, setWorlds] = useState([]);
  const [world, setWorld] = useState(null);
  const [conversations, setConversations] = useState([]);
  const [conversationId, setConversationId] = useState(null);
  const [conversation, setConversation] = useState(null);
  const [trace, setTrace] = useState(null);
  const [streaming, setStreaming] = useState(false);
  const [error, setError] = useState("");
  const abortRef = useRef(null);

  const guard = useCallback(async (fn) => {
    try {
      setError("");
      return await fn();
    } catch (err) {
      if (err.status === 401) setUser(null);
      setError(err.message);
    }
  }, []);

  useEffect(() => {
    api.me().then(setUser).catch(() => setUser(null));
  }, []);

  useEffect(() => {
    if (user) guard(async () => setWorlds(await api.worlds()));
  }, [user, guard]);

  async function pickWorld(w) {
    setWorld(w);
    setConversationId(null);
    setConversation(null);
    setTrace(null);
    await guard(async () => setConversations((await api.conversations(w.id)).conversations));
  }

  async function pickConversation(id) {
    setConversationId(id);
    setTrace(null);
    await guard(async () => setConversation(await api.conversation(id)));
  }

  async function send(text) {
    setStreaming(true);
    setTrace(emptyTrace(text));
    const controller = new AbortController();
    abortRef.current = controller;
    try {
      await streamChatTurn(conversationId, text, {
        signal: controller.signal,
        onEvent: (ev) => setTrace((t) => applyEvent(t, ev)),
      });
    } catch (err) {
      if (err.name !== "AbortError") setTrace((t) => ({ ...t, error: err.message, status: "Error", done: true }));
    } finally {
      setStreaming(false);
      abortRef.current = null;
      // Reload the saved history; keep the trace for the inspector.
      const fresh = await guard(() => api.conversation(conversationId));
      if (fresh) {
        setConversation(fresh);
        setTrace((t) => (t ? { ...t, persisted: true, done: true } : t));
      }
    }
  }

  if (user === undefined) return <div className="auth"><p className="muted">Loading…</p></div>;
  if (user === null) return <AuthScreen onAuthed={setUser} />;

  return (
    <div className="app">
      <Sidebar
        user={user}
        worlds={worlds}
        world={world}
        conversations={conversations}
        conversationId={conversationId}
        onPickWorld={pickWorld}
        onPickConversation={pickConversation}
        onCreateWorld={(name, description) =>
          guard(async () => {
            const w = await api.createWorld(name, description);
            setWorlds((ws) => [...ws, w]);
            await pickWorld(w);
          })
        }
        onCreateConversation={(title) =>
          guard(async () => {
            const c = await api.createConversation(world.id, title);
            setConversations((cs) => [c, ...cs]);
            await pickConversation(c.id);
          })
        }
        onLogout={() => guard(async () => { await api.logout(); setUser(null); })}
      />
      <div className="center">
        {error && <div className="banner" role="alert">{error}<button className="ghost small" onClick={() => setError("")} aria-label="Dismiss">×</button></div>}
        <Chat conversation={conversation} trace={trace} onSend={send} streaming={streaming} onStop={() => abortRef.current?.abort()} />
      </div>
      <Inspector trace={trace} />
    </div>
  );
}
