// Thin client for the Lorekeeper backend. Every path goes through Vite's /api proxy.

export async function inspectRequest(method, path, body) {
  const started = performance.now();
  const res = await fetch(`/api${path}`, {
    method,
    credentials: "include",
    headers: body === undefined ? {} : { "Content-Type": "application/json" },
    body: body === undefined ? undefined : JSON.stringify(body),
  });
  const text = await res.text();
  let data;
  try { data = text ? JSON.parse(text) : null; } catch { data = text; }
  return { method, path, status: res.status, ok: res.ok,
    durationMs: Math.round(performance.now() - started), data };
}

async function request(method, path, body) {
  const result = await inspectRequest(method, path, body);
  if (!result.ok) {
    const data = result.data;
    const message = (data && (data.message || data.error)) || data || "Request failed";
    const err = new Error(`${result.status} ${typeof message === "string" ? message : JSON.stringify(message)}`);
    err.status = result.status;
    throw err;
  }
  return result.data;
}

export const api = {
  me: () => request("GET", "/auth/me"),
  login: (email, password) => request("POST", "/auth/login", { email, password }),
  signup: (username, email, password) => request("POST", "/auth/signup", { username, email, password }),
  logout: () => request("POST", "/auth/logout"),

  worlds: () => request("GET", "/world"),
  createWorld: (name, description) => request("POST", "/world", { name, description }),
  deleteWorld: (id) => request("DELETE", `/world/${id}`),

  conversations: (worldId) => request("GET", `/world/${worldId}/conversations`),
  createConversation: (worldId, title) => request("POST", `/world/${worldId}/conversations`, { title }),
  conversation: (worldId, id) => request("GET", `/world/${worldId}/conversations/${id}`),
  deleteConversation: (worldId, id) => request("DELETE", `/world/${worldId}/conversations/${id}`),
};

/**
 * POST a chat turn and read the SSE response as it arrives.
 * EventSource can't POST, so this parses the stream by hand:
 * messages are separated by a blank line; each has optional `event:` and `data:` lines.
 */
export async function streamChatTurn(worldId, conversationId, content, { onEvent, signal }) {
  const res = await fetch(`/api/world/${worldId}/conversations/${conversationId}/messages`, {
    method: "POST",
    credentials: "include",
    headers: { "Content-Type": "application/json", Accept: "text/event-stream" },
    body: JSON.stringify({ content }),
    signal,
  });
  if (!res.ok) {
    throw new Error(`${res.status} ${await res.text()}`);
  }

  const reader = res.body.getReader();
  const decoder = new TextDecoder();
  let buffer = "";

  for (;;) {
    const { value, done } = await reader.read();
    if (done) break;
    buffer += decoder.decode(value, { stream: true });

    let boundary;
    while ((boundary = buffer.indexOf("\n\n")) !== -1) {
      const raw = buffer.slice(0, boundary);
      buffer = buffer.slice(boundary + 2);

      let eventName = "message";
      const dataLines = [];
      for (const line of raw.split("\n")) {
        if (line.startsWith("event:")) eventName = line.slice(6).trim();
        else if (line.startsWith("data:")) dataLines.push(line.slice(5).replace(/^ /, ""));
      }
      if (dataLines.length === 0) continue;
      const data = dataLines.join("\n");

      if (eventName === "error") {
        onEvent({ kind: "error", message: data });
        continue;
      }
      try {
        onEvent({ kind: "item", item: JSON.parse(data) });
      } catch {
        onEvent({ kind: "unparsed", raw: data });
      }
    }
  }
}
