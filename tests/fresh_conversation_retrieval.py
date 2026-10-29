"""Paid retrieval check: one question per empty conversation in a retained synthetic world."""
import csv
import io
import json
import os
from pathlib import Path
import time
import uuid

import requests
from adversarial_api import cypher, sql

assert os.environ.get("QA_SMOKE_PAID") == "1", "Explicit paid-test opt-in required"
BASE = os.environ.get("QA_BASE_URL", "http://127.0.0.1:18001")
OUT = Path(os.environ["QA_SMOKE_OUTPUT"])
LOG = Path(os.environ["QA_SMOKE_LOG"])
OUT.mkdir(parents=True, exist_ok=True)
fixture = json.loads(Path("tests/artifacts/smoke-2026-10-29-10turns/report.json").read_text())
world = str(uuid.UUID(fixture["world_id"]))
email = sql(f"SELECT u.email FROM users u JOIN worlds w ON w.user_id = u.id WHERE w.id = '{world}'::uuid")
assert email.startswith("smoke20-") and email.endswith("@example.com"), "Only use the synthetic smoke account"
session = requests.Session()


def call(method, path, **kwargs):
    response = session.request(method, BASE + path, timeout=240, **kwargs)
    response.raise_for_status()
    return response


def snapshot():
    return {
        "nodes": cypher(f"MATCH (n {{world_id: '{world}'}}) RETURN labels(n), n.id, n.name, n.description, n.properties, n.existing_value, n.new_value, n.explanation, n.resolved, n.source_message_ids"),
        "relationships": cypher(f"MATCH (a {{world_id: '{world}'}})-[r]->(b) RETURN a.id, type(r), properties(r), b.id"),
    }


def wait_research(conversation):
    deadline = time.monotonic() + 240
    while time.monotonic() < deadline:
        text = LOG.read_text(errors="replace")
        if f"research completed for conversation {conversation}" in text:
            return "complete"
        if f"research failed for conversation {conversation}" in text:
            return "failed"
        time.sleep(1)
    return "timeout"


QUESTIONS = [
    ("minor_facts_and_correction", "Where was Mira Venn born, when did she move to Sablecross, what does she keep hidden, and who knows about it? Use only saved lore."),
    ("relationship_conditions", "Who owns the Glass Aqueduct, what is its present condition, and does it currently supply the reservoir? Use only saved lore."),
    ("shared_title", "What are Oren Dask's and Lysa Renn's roles and access rules? Both are called Warden; distinguish them using saved lore."),
    ("unresolved_conflicts", "Which contradictions remain unresolved in this world? Give both versions without resolving them. Use only saved lore."),
    ("unknown_facts", "How many people live in Sablecross, and how much water remains in its reservoir? If those facts aren't saved, say so."),
]
call("POST", "/auth/login", json={"email": email, "password": "Smoke-Test-938!"})
report = {"world_id": world, "original_conversation_id": fixture["conversation_id"], "before": snapshot(), "cases": []}
(OUT / "report.json").write_text(json.dumps(report, indent=2))
for index, (name, question) in enumerate(QUESTIONS, 1):
    conversation = call("POST", f"/world/{world}/conversations", json={"title": f"Fresh retrieval: {name}"}).json()["id"]
    initial = call("GET", f"/conversation/{conversation}").json()
    assert initial["messages"] == [], "Retrieval check requires an empty conversation"
    started = time.monotonic()
    response = call("POST", f"/conversation/{conversation}/messages", json={"content": question})
    response.encoding = "utf-8"
    raw = response.text
    (OUT / f"{name}.sse").write_text(raw)
    saved = call("GET", f"/conversation/{conversation}").json()
    status = wait_research(conversation) if '"type":"finalResponse"' in raw or '"type": "finalResponse"' in raw else "not_started"
    messages = saved["messages"]
    item = {"name": name, "conversation_id": conversation, "initial_message_count": 0,
            "question": question, "messages": messages, "research_status": status,
            "seconds": round(time.monotonic() - started, 1), "final_response": "finalResponse" in raw,
            "stream_error": "event: error" in raw}
    report["cases"].append(item)
    (OUT / "report.json").write_text(json.dumps(report, indent=2))
    print(f"RETRIEVAL case={index}/5 name={name} final={item['final_response']} research={status}", flush=True)
    assert status != "timeout" and item["final_response"], "Incomplete request; inspect artifacts before paid retry"
report["after"] = snapshot()
report["graph_unchanged"] = all(
    sorted(csv.reader(io.StringIO(report["before"][key]))) == sorted(csv.reader(io.StringIO(report["after"][key])))
    for key in report["before"]
)
(OUT / "report.json").write_text(json.dumps(report, indent=2))
print(f"RETRIEVAL COMPLETE graph_unchanged={report['graph_unchanged']} artifacts={OUT}", flush=True)
