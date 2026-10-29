"""Explicitly authorized, paid smoke test. Keeps its synthetic world for inspection."""
import csv
import io
import json
import os
from pathlib import Path
import time
import uuid

import requests
from adversarial_api import cypher

assert os.environ.get("QA_SMOKE_PAID") == "1", "Paid smoke test needs explicit opt-in"
BASE = os.environ.get("QA_BASE_URL", "http://127.0.0.1:18001")
LOG = Path(os.environ["QA_SMOKE_LOG"])
OUT = Path(os.environ["QA_SMOKE_OUTPUT"])
OUT.mkdir(parents=True, exist_ok=True)
TURNS = [
    "I'm building Brinewake, a stormy island chain. Vesper is its main port city, built around a tidal lighthouse. Keep your replies brief while I work through the details.",
    "Vesper's lighthouse is maintained by the Lantern Guild. Its lenses run on blue salt mined on the island of Orin. The guild buys salt from Orin's miners; it doesn't own the mines.",
    "Asha Pell captains the courier ship Nightjar. She grew up in Vesper and belongs to the Lantern Guild. Her crew usually calls her Captain Ash.",
    "Bren Vale captains the fishing boat Petrel. People on the dock call him Captain too. He's unrelated to Asha and isn't a member of the guild.",
    "Asha has a small brass key sewn inside her left coat cuff. It opens the old lens room beneath Vesper's lighthouse. Only her navigator, Iven Marr, knows she has it.",
    "Iven lost his right eye in the winter storm of 312. He navigates the Nightjar using tide charts, not magic. Asha trusts him with routes but doesn't let him negotiate contracts.",
    "The Nightjar carries blue salt from Orin to Vesper only between April and September. During winter the strait freezes and the guild uses stored salt.",
    "Who maintains Vesper's lighthouse, where does its fuel come from, and does the guild own the mines?",
    "Orin's miners formed the Deep Bell Union. Their leader is Sera Dain. She refuses to sell salt to the military, but still honors civilian contracts with the Lantern Guild.",
    "Vesper's governor, Tal Ren, imposed a toll on blue salt shipments in 319. The Lantern Guild opposes the toll. Bren supports it because the revenue repairs fishing piers.",
    "Actually, correct Asha's birthplace: she was born on Orin, then moved to Vesper at age six. I mixed up being born there with growing up there. Keep her guild membership and the rest unchanged.",
    "What is hidden in Asha's coat, what does it open, and who knows about it? Also, where was she born?",
    "The old lens room was permanently sealed by a rockfall in 318. Nobody has reopened it, and there is no other entrance.",
    "In 320, Iven meets Sera inside that old lens room to negotiate a salt contract. Those are my notes as written; I haven't decided how to reconcile them yet.",
    "Let's leave that contradiction unresolved for now. Don't invent a secret entrance or an excavation to make the meeting work.",
    "Hypothetical only: if Orin stopped exporting blue salt for a year, what would happen to Vesper's lighthouse? Don't add this export ban to canon, and say what we don't know.",
    "One more small fact: the Petrel's cook is Nessa. She keeps a black cat called Button aboard. Nessa is Bren's older sister, not his employee on a formal wage.",
    "I mean Bren, the Petrel's captain, here: Captain refuses to join the Lantern Guild because he distrusts Tal Ren's friends. This isn't about Asha.",
    "Correction to the Nightjar's route: it no longer carries blue salt from Orin to Vesper at all. It now carries letters from Vesper to Kest, a southern island. The April-to-September salt route is retired, not a second current route.",
    "Give me a short factual handoff: Asha's birthplace and secret, Iven's injury and role, Bren's guild stance and family aboard Petrel, the Nightjar's current route, and the unresolved lens-room issue. Separate unknowns and contradictions from canon.",
]

if os.environ.get("QA_SMOKE_SCENARIO"):
    scenario = json.loads(Path(os.environ["QA_SMOKE_SCENARIO"]).read_text())
    TURNS = scenario["turns"]
else:
    scenario = {"world_name": "Brinewake — smoke", "conversation_title": "Brinewake founding notes"}


session = requests.Session()

def call(method, path, **kwargs):
    response = session.request(method, BASE + path, timeout=240, **kwargs)
    response.raise_for_status()
    return response


def snapshot(world_id):
    # world_id comes from the trusted server response and is validated before query interpolation.
    uuid.UUID(world_id)
    return {
        "nodes_csv": cypher(f"MATCH (n {{world_id: '{world_id}'}}) RETURN labels(n), n.id, n.name, n.aliases, n.kind, n.description, n.properties, n.entity_id, n.field, n.existing_value, n.new_value, n.explanation, n.resolved, n.source_message_ids"),
        "relationships_csv": cypher(f"MATCH (a {{world_id: '{world_id}'}})-[r]->(b) RETURN a.name, a.id, type(r), properties(r), b.name, b.id, r.source_message_ids"),
    }


def audit_sources(graph, messages):
    user_ids = {m["id"] for m in messages if m["role"] == "user"}
    missing, invalid, duplicates = [], [], []
    for table, label_key, source_key in [
        ("nodes_csv", "n.id", "n.source_message_ids"),
        ("relationships_csv", "properties(r)", "r.source_message_ids"),
    ]:
        for row in csv.DictReader(io.StringIO(graph[table])):
            if table == "relationships_csv" and "HAS_CONFLICT" in row["type(r)"]:
                continue
            raw = row[source_key]
            sources = json.loads(raw) if raw != "Null" else []
            label = row[label_key]
            if not sources:
                missing.append(label)
            invalid.extend(source for source in sources if source not in user_ids)
            if len(sources) != len(set(sources)):
                duplicates.append(label)
    return {"missing_source_records": missing, "invalid_source_ids": invalid, "duplicate_source_records": duplicates}


def wait_research(conversation_id, expected):
    deadline = time.monotonic() + 240
    while time.monotonic() < deadline:
        text = LOG.read_text(errors="replace") if LOG.exists() else ""
        completed = text.count(f"research completed for conversation {conversation_id}")
        failed = text.count(f"research failed for conversation {conversation_id}")
        if completed + failed >= expected:
            return {"completed_total": completed, "failed_total": failed}
        time.sleep(1)
    return {"timeout": True}


email = f"smoke20-{uuid.uuid4().hex}@example.com"
credentials = {"username": "Smoke Writer", "email": email, "password": "Smoke-Test-938!"}
call("POST", "/auth/signup", json=credentials)
call("POST", "/auth/login", json=credentials)
world = call("POST", "/world", json={"name": os.environ.get("QA_SMOKE_WORLD_NAME", scenario["world_name"]), "description": "Synthetic research smoke test, 2026-10-29"}).json()["id"]
conversation = call("POST", f"/world/{world}/conversations", json={"title": scenario["conversation_title"]}).json()["id"]
report = {"world_id": world, "conversation_id": conversation, "turns": []}
print(f"SMOKE world={world} conversation={conversation}", flush=True)
(OUT / "report.json").write_text(json.dumps(report, indent=2))
for index, message in enumerate(TURNS, 1):
    started = time.monotonic()
    response = call("POST", f"/conversation/{conversation}/messages", json={"content": message})
    response.encoding = "utf-8"
    raw = response.text
    (OUT / f"turn-{index:02}.sse").write_text(raw)
    saved = call("GET", f"/conversation/{conversation}").json()
    # Store actual saved messages rather than reconstructing them from stream deltas.
    (OUT / "conversation.json").write_text(json.dumps(saved, indent=2))
    research = wait_research(conversation, index) if "finalResponse" in raw else {"not_started": True}
    item = {"turn": index, "user": message, "seconds": round(time.monotonic() - started, 1),
            "final_response": "finalResponse" in raw, "stream_error": "event: error" in raw,
            "research": research, "graph": snapshot(world)}
    item["source_audit"] = audit_sources(item["graph"], saved["messages"])
    item["user_message_id"] = saved["messages"][-2]["id"]
    report["turns"].append(item)
    (OUT / "report.json").write_text(json.dumps(report, indent=2))
    print(f"SMOKE turn={index}/{len(TURNS)} seconds={item['seconds']} final={item['final_response']} research={research}", flush=True)
    if research.get("timeout") or not item["final_response"]:
        raise SystemExit("Smoke stopped: incomplete turn; inspect artifacts before retrying paid calls")
(OUT / "final-graph.json").write_text(json.dumps(snapshot(world), indent=2))
print(f"SMOKE COMPLETE artifacts={OUT}", flush=True)
