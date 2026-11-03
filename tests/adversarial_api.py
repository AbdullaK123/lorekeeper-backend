"""Live contract tests. Create fresh users; inject faults only in the QA database."""
import concurrent.futures
import contextlib
import json
import os
import subprocess
import time
import unittest
import uuid

import requests

BASE = os.environ.get("QA_BASE_URL", "http://127.0.0.1:18000")
PASSWORD = "Test-Pass-938!"
AI = os.environ.get("QA_AI") == "1"


def sql(statement):
    result = subprocess.run(
        ["docker", "exec", "-i", "lorekeeper-backend-app-db-1", "psql",
         "-U", "dev", "-d", "lorekeeper_adversarial", "-At", "-v", "ON_ERROR_STOP=1"],
        input=statement, text=True, capture_output=True, timeout=20,
    )
    if result.returncode:
        raise RuntimeError(result.stderr)
    return result.stdout.strip()


def request(session, method, path, **kwargs):
    return session.request(method, BASE + path, timeout=180, **kwargs)


def cypher(statement):
    result = subprocess.run(
        ["docker", "exec", "-i", "lorekeeper-backend-app-memgraph-1", "mgconsole", "--output-format=csv"],
        input=statement + ";\n", text=True, capture_output=True, timeout=20,
    )
    if result.returncode or "ERROR" in result.stderr:
        raise RuntimeError(result.stderr or result.stdout)
    return result.stdout


def world(session, name="QA world"):
    response = request(session, "POST", "/world", json={"name": name, "description": "QA only"})
    response.raise_for_status()
    return response.json()["id"]


def conversation(session, world_id):
    response = request(session, "POST", f"/world/{world_id}/conversations", json={"title": "QA"})
    response.raise_for_status()
    return response.json()["id"]


class ApiTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        for _ in range(100):
            try:
                if requests.get(BASE + "/health", timeout=1).ok:
                    break
            except requests.RequestException:
                pass
            time.sleep(.1)
        cls.a, cls.b = requests.Session(), requests.Session()
        cls.emails = []
        for session in (cls.a, cls.b):
            email = f"qa-{uuid.uuid4().hex}@example.com"
            cls.emails.append(email)
            payload = {"username": "QA", "email": email, "password": PASSWORD}
            request(session, "POST", "/auth/signup", json=payload).raise_for_status()
            request(session, "POST", "/auth/login", json=payload).raise_for_status()
        cls.w = world(cls.a)
        cls.c = conversation(cls.a, cls.w)

    def assert_denied(self, response):
        self.assertIn(response.status_code, (403, 404),
                      f"Another user reaches private data: {response.status_code} {response.text[:300]}")

    @contextlib.contextmanager
    def source_messages_fixture(self):
        w = world(self.a, "Source navigation QA")
        c = conversation(self.a, w)
        stamp = "2026-11-01T00:00:00Z"
        messages = [
            {"role": "user", "id": str(uuid.uuid4()), "content": "The harbor closes at dusk.", "created_at": stamp},
            {"role": "assistant", "id": str(uuid.uuid4()), "content": None,
             "tool_calls": [{"id": "qa-call", "name": "search_world", "arguments": {"query": "harbor"}}],
             "created_at": stamp},
            {"role": "tool", "id": str(uuid.uuid4()), "tool_call_id": "qa-call",
             "content": "Saved harbor lore", "created_at": stamp},
        ]
        try:
            body = json.dumps(messages).replace("'", "''")
            sql(f"UPDATE conversations SET messages = '{body}'::jsonb WHERE id = '{c}'")
            yield w, c, messages
        finally:
            request(self.a, "DELETE", f"/world/{w}").raise_for_status()

    @contextlib.contextmanager
    def research_status_fixture(self, status=None):
        with self.source_messages_fixture() as (w, c, messages):
            message = messages[0]["id"]
            try:
                if status is not None:
                    sql(f"INSERT INTO dbos.workflow_status "
                        "(workflow_uuid, status, name, executor_id, application_version, application_name, queue_name) "
                        f"VALUES ('{message}', '{status}', 'qa_status_fixture', 'qa_status_fixture', "
                        f"'qa-status-{message}', 'lorekeeper', 'qa-status-{message}')")
                yield w, c, message
            finally:
                sql(f"DELETE FROM dbos.workflow_status WHERE workflow_uuid = '{message}' AND name = 'qa_status_fixture'")

    def test_research_status_maps_all_stored_states(self):
        cases = {
            "ENQUEUED": "queued", "DELAYED": "queued", "PENDING": "running",
            "SUCCESS": "completed", "ERROR": "failed", "CANCELLED": "cancelled",
            "MAX_RECOVERY_ATTEMPTS_EXCEEDED": "failed",
        }
        with self.research_status_fixture("ENQUEUED") as (w, _, message):
            for stored, expected in cases.items():
                with self.subTest(stored=stored):
                    sql(f"UPDATE dbos.workflow_status SET status = '{stored}' WHERE workflow_uuid = '{message}'")
                    response = request(self.a, "GET", f"/world/{w}/messages/{message}/research")
                    self.assertEqual(response.status_code, 200, f"Stored research state is unreadable: {response.text[:300]}")
                    self.assertEqual(response.json(), {"status": expected},
                                     "Research status misrepresents queued, active, or finished work")

    def test_research_status_missing_workflow_is_not_submitted(self):
        with self.research_status_fixture() as (w, _, message):
            response = request(self.a, "GET", f"/world/{w}/messages/{message}/research")
            self.assertEqual(response.status_code, 200, "Saved message without a workflow is treated as missing data")
            self.assertEqual(response.json(), {"status": "not_submitted"},
                             "No workflow is reported as running or finished")

    def test_research_status_enforces_message_access(self):
        with self.research_status_fixture("SUCCESS") as (w, _, message):
            path = f"/world/{w}/messages/{message}/research"
            self.assertEqual(request(self.b, "GET", path).status_code, 404, "Foreign user sees private research status")
            self.assertEqual(request(requests.Session(), "GET", path).status_code, 401,
                             "Anonymous caller reads private research status")
            for url in (f"/world/{self.w}/messages/{message}/research",
                        f"/world/{uuid.uuid4()}/messages/{message}/research",
                        f"/world/{w}/messages/{uuid.uuid4()}/research"):
                self.assertEqual(request(self.a, "GET", url).status_code, 404,
                                 "Status endpoint ignores message ownership or world scope")
            for url in (f"/world/{w}/messages/invalid/research",
                        f"/world/invalid/messages/{message}/research"):
                self.assertEqual(request(self.a, "GET", url).status_code, 400,
                                 "Status endpoint accepts malformed identifiers")

    def test_research_status_lookup_error_is_not_not_submitted(self):
        with self.research_status_fixture("QA_UNKNOWN_STATUS") as (w, _, message):
            path = f"/world/{w}/messages/{message}/research"
            response = request(self.a, "GET", path)
            self.assertEqual(response.status_code, 500,
                             "Failed DBOS status decoding is reported as a valid status")
            self.assertEqual(response.json().get("error"), "internal",
                             "DBOS lookup failure becomes not_submitted instead of a server error")
            self.assertEqual(request(self.b, "GET", path).status_code, 404,
                             "Status lookup runs before the message access check")

    def test_research_status_deleted_message_is_not_exposed(self):
        with self.research_status_fixture("SUCCESS") as (w, c, message):
            response = request(self.a, "DELETE", f"/world/{w}/conversations/{c}")
            self.assertEqual(response.status_code, 204, "Fixture conversation cannot be deleted")
            response = request(self.a, "GET", f"/world/{w}/messages/{message}/research")
            self.assertEqual(response.status_code, 404, "An existing workflow exposes a deleted source message")

    def test_dev_static_playground_preserves_api_sessions(self):
        response = request(self.a, "GET", "/")
        self.assertEqual(response.status_code, 200, "Built dev playground is unavailable")
        self.assertIn('id="root"', response.text, "Root does not serve the playground entry")
        self.assertIn("text/html", response.headers.get("Content-Type", ""), "Browser cannot identify playground HTML")
        import re
        asset = re.search(r'src="([^"]+\.js)"', response.text)
        self.assertIsNotNone(asset, "Built HTML has no JavaScript entry")
        self.assertEqual(request(self.a, "GET", asset.group(1)).status_code, 200, "Browser cannot load the built JavaScript")
        self.assertEqual(request(self.a, "GET", "/api/auth/me").json(), request(self.a, "GET", "/auth/me").json(),
                         "Static playground API alias loses the signed session")
        self.assertEqual(request(requests.Session(), "GET", "/api/auth/me").status_code, 401,
                         "Development alias bypasses authentication")
        self.assertEqual(request(self.a, "GET", "/api/world").status_code, 200, "World UI cannot call the API alias")
        self.assertEqual(request(self.a, "GET", "/api/missing").status_code, 404, "Unknown API request returns playground HTML")

    def test_nested_conversation_enforces_world_scope(self):
        with self.source_messages_fixture() as (w, c, messages):
            path = f"/world/{w}/conversations/{c}"
            response = request(self.a, "GET", path)
            self.assertEqual(response.status_code, 200, "Owner cannot read a nested conversation")
            self.assertEqual(response.json()["world_id"], w, "Nested route returns another world's conversation")
            for wrong_world in (self.w, str(uuid.uuid4())):
                wrong_path = f"/world/{wrong_world}/conversations/{c}"
                for method, suffix, kwargs in (("GET", "", {}), ("DELETE", "", {}),
                                               ("POST", "/messages", {"json": {"content": "do not run a model"}})):
                    self.assertEqual(request(self.a, method, wrong_path + suffix, **kwargs).status_code, 404,
                                     "Conversation route ignores its parent world")
            self.assertEqual(request(self.a, "GET", path).json()["messages"], messages,
                             "Rejected wrong-world requests mutate saved history")
            self.assertEqual(request(self.a, "DELETE", path).status_code, 204,
                             "Owner cannot delete a nested conversation")
            self.assertEqual(request(self.a, "GET", path).status_code, 404,
                             "Deleted nested conversation remains readable")

    def test_research_retry_rejects_ineligible_states(self):
        for state in ("PENDING", "ENQUEUED", "DELAYED", "SUCCESS", "CANCELLED"):
            with self.subTest(state=state), self.research_status_fixture(state) as (w, _, message):
                response = request(self.a, "POST", f"/world/{w}/messages/{message}/research/retry")
                self.assertEqual(response.status_code, 409, "Retry launches research in an ineligible state")

    def test_research_retry_enforces_access_and_user_role(self):
        with self.source_messages_fixture() as (w, _, messages):
            path = f"/world/{w}/messages/{messages[0]['id']}/research/retry"
            self.assertEqual(request(self.b, "POST", path).status_code, 404, "Foreign user starts private research")
            self.assertEqual(request(requests.Session(), "POST", path).status_code, 401, "Anonymous user starts research")
            for message in messages[1:]:
                response = request(self.a, "POST", f"/world/{w}/messages/{message['id']}/research/retry")
                self.assertEqual(response.status_code, 422, "Assistant or tool text becomes user canon")

    def test_research_retry_resumes_exhausted_workflow(self):
        with self.research_status_fixture("MAX_RECOVERY_ATTEMPTS_EXCEEDED") as (w, _, message):
            try:
                path = f"/world/{w}/messages/{message}/research"
                response = request(self.a, "POST", path + "/retry")
                self.assertEqual(response.status_code, 200, "Recovery-exhausted workflow cannot resume")
                self.assertEqual(sql(f"SELECT workflow_id FROM message_research_workflows WHERE message_id='{message}'"),
                                 message, "Resume unnecessarily changes workflow identity")
                self.assertEqual(request(self.a, "GET", path).json(), {"status": "queued"},
                                 "Resume remains reported as failed")
            finally:
                sql(f"DELETE FROM message_research_workflows WHERE message_id='{message}'")

    def test_research_retry_tracks_fork_and_blocks_duplicate(self):
        with self.research_status_fixture("ERROR") as (w, _, message):
            sql(f"INSERT INTO dbos.operation_outputs (workflow_uuid, function_id, function_name, output) "
                f"VALUES ('{message}', 0, 'qa_checkpoint', '{{}}')")
            path = f"/world/{w}/messages/{message}/research"
            try:
                with concurrent.futures.ThreadPoolExecutor(max_workers=2) as executor:
                    responses = list(executor.map(lambda _: request(self.a, "POST", path + "/retry"), range(2)))
                self.assertEqual(sorted(r.status_code for r in responses), [200, 409],
                                 "Concurrent retry creates duplicate forks or fails to submit")
                active = sql(f"SELECT workflow_id FROM message_research_workflows WHERE message_id='{message}'")
                self.assertTrue(active and active != message, "Retry loses the new fork ID")
                self.assertEqual(request(self.a, "GET", path).json(), {"status": "queued"},
                                 "Polling still reports the original failed workflow")
                sql(f"UPDATE dbos.workflow_status SET status='SUCCESS' WHERE workflow_uuid='{active}'")
                self.assertEqual(request(self.a, "GET", path).json(), {"status": "completed"},
                                 "Polling does not follow retry completion")
                sql(f"UPDATE dbos.workflow_status SET status='ERROR' WHERE workflow_uuid='{active}'")
                sql(f"INSERT INTO dbos.operation_outputs (workflow_uuid, function_id, function_name, output) "
                    f"VALUES ('{active}', 0, 'qa_checkpoint', '{{}}') ON CONFLICT DO NOTHING")
                response = request(self.a, "POST", path + "/retry")
                self.assertEqual(response.status_code, 200, "A second failed attempt cannot be retried")
                newest = sql(f"SELECT workflow_id FROM message_research_workflows WHERE message_id='{message}'")
                self.assertNotIn(newest, (message, active), "Second retry forks the wrong attempt")
            finally:
                sql(f"DELETE FROM message_research_workflows WHERE message_id='{message}'")
                sql(f"DELETE FROM dbos.workflow_status WHERE name='qa_status_fixture' "
                    f"AND application_version='qa-status-{message}' AND workflow_uuid<>'{message}'")

    def test_source_message_route_preserves_all_roles(self):
        with self.source_messages_fixture() as (w, c, messages):
            for message in messages:
                with self.subTest(role=message["role"]):
                    response = request(self.a, "GET", f"/world/{w}/messages/{message['id']}")
                    self.assertEqual(response.status_code, 200, f"Saved source is unreachable: {response.text[:300]}")
                    self.assertEqual(response.json(), {"world_id": w, "conversation_id": c, "message": message},
                                     "Source navigation changes message data or points to the wrong conversation")

    def test_source_message_route_enforces_scope(self):
        with self.source_messages_fixture() as (w, _, messages):
            message = messages[0]["id"]
            path = f"/world/{w}/messages/{message}"
            self.assertEqual(request(self.b, "GET", path).status_code, 404,
                             "Foreign user reads private source text")
            self.assertEqual(request(requests.Session(), "GET", path).status_code, 401,
                             "Anonymous caller reads private source text")
            for url in (f"/world/{self.w}/messages/{message}",
                        f"/world/{uuid.uuid4()}/messages/{message}",
                        f"/world/{w}/messages/{uuid.uuid4()}"):
                self.assertEqual(request(self.a, "GET", url).status_code, 404,
                                 "Wrong world or missing message returns private source data")
            for url in (f"/world/{w}/messages/not-a-uuid", f"/world/not-a-uuid/messages/{message}"):
                self.assertEqual(request(self.a, "GET", url).status_code, 400,
                                 "Invalid source identifier bypasses UUID path parsing")

    def test_source_message_route_returns_404_after_deletion(self):
        with self.source_messages_fixture() as (w, c, messages):
            deleted = request(self.a, "DELETE", f"/world/{w}/conversations/{c}")
            self.assertEqual(deleted.status_code, 204, "Conversation fixture cannot be deleted")
            response = request(self.a, "GET", f"/world/{w}/messages/{messages[0]['id']}")
            self.assertEqual(response.status_code, 404, "Deleted conversation still exposes source messages")

    @contextlib.contextmanager
    def populated_graph(self):
        w = world(self.a, "Graph browsing QA")
        nodes = [str(uuid.UUID(int=i)) for i in (1, 2, 3)]
        edges = [str(uuid.UUID(int=i)) for i in (11, 12, 13, 14)]
        try:
            for index, (node, kind) in enumerate(zip(nodes, ("character", "character", "place"))):
                cypher(f"CREATE (:Entity {{id: '{node}', world_id: '{w}', name: 'Archive{index}', "
                       f"name_key: 'archive{index}', aliases: ['Alias{index}'], kind: ['{kind}', 'lore'], "
                       "description: 'QA graph browsing', properties: '{}', source_message_ids: []})")
            for edge, source, target, label in zip(
                    edges, (nodes[0], nodes[1], nodes[0], nodes[0]),
                    (nodes[1], nodes[0], nodes[2], nodes[1]),
                    ("KNOWS", "MENTORS", "VISITS", "MENTORS")):
                cypher(f"MATCH (a:Entity {{world_id: '{w}', id: '{source}'}}), "
                       f"(b:Entity {{world_id: '{w}', id: '{target}'}}) "
                       f"CREATE (a)-[:{label} {{id: '{edge}', world_id: '{w}', "
                       f"from_id: '{source}', to_id: '{target}', rel_type: '{label}', "
                       "description: 'QA connection', properties: '{}', source_conversation: 'qa', "
                       "source_message_ids: []}]->(b)")
            yield w, nodes, edges
        finally:
            cypher(f"MATCH (n {{world_id: '{w}'}}) DETACH DELETE n")
            request(self.a, "DELETE", f"/world/{w}").raise_for_status()

    @contextlib.contextmanager
    def conflict_resolution_fixture(self):
        with self.populated_graph() as (w, nodes, _):
            conflict = str(uuid.uuid4())
            cypher(f"MATCH (e:Entity {{world_id: '{w}', id: '{nodes[0]}'}}) "
                   f"CREATE (e)-[:HAS_CONFLICT]->(:Conflict {{id: '{conflict}', world_id: '{w}', "
                   f"entity_id: '{nodes[0]}', field: 'age', existing_value: '20', new_value: '30', "
                   "severity: 'hard', explanation: 'Conflicting ages', source_conversation: 'qa', "
                   "source_message_ids: [], resolved: false})")
            yield w, nodes[0], conflict

    def test_accept_conflict_updates_only_saved_property(self):
        with self.conflict_resolution_fixture() as (w, entity, conflict):
            path = f"/world/{w}/conflicts/{conflict}"
            cypher(f"MATCH (e:Entity {{world_id: '{w}', id: '{entity}'}}) "
                   "SET e.properties = '{\"age\":20,\"title\":\"Captain\"}'")
            payload = {"decision": "accept_new"}
            self.assert_denied(request(self.b, "POST", path + "/resolve", json=payload))
            accepted = request(self.a, "POST", path + "/resolve", json=payload)
            self.assertEqual(accepted.status_code, 200, f"Accept-new fails: {accepted.text[:300]}")
            self.assertEqual(accepted.json()["decision"], "accept_new", "Accepted claim has the wrong decision")
            self.assertTrue(accepted.json()["resolved"], "Accepted conflict remains open")
            result = request(self.a, "GET", f"/world/{w}/entities/{entity}").json()
            self.assertEqual(result["properties"], {"age": 30, "title": "Captain"},
                             "Accept-new changes unrelated facts or loses the value type")
            self.assertEqual(request(self.a, "POST", path + "/resolve", json=payload).json(), accepted.json(),
                             "Retry changes the accepted claim")
            rejected = request(self.a, "POST", path + "/resolve", json={"decision": "keep_existing"})
            self.assertGreaterEqual(rejected.status_code, 400, "A later keep-existing request relabels an accepted claim")
            self.assertEqual(request(self.a, "GET", path).json()["decision"], "accept_new",
                             "A different decision overwrites the accepted decision")

    def test_accept_conflict_rolls_back_entity_on_resolution_failure(self):
        with self.conflict_resolution_fixture() as (w, entity, conflict):
            label = "QaResolution" + uuid.uuid4().hex
            cypher(f"MATCH (e:Entity {{world_id: '{w}', id: '{entity}'}}) SET e.properties = '{{\"age\":20}}'")
            cypher(f"MATCH (c:Conflict {{world_id: '{w}', id: '{conflict}'}}) SET c:{label}")
            cypher(f"CREATE (:{label} {{world_id: '{w}', decision: 'accept_new'}})")
            cypher(f"CREATE CONSTRAINT ON (n:{label}) ASSERT n.decision IS UNIQUE")
            try:
                response = request(self.a, "POST", f"/world/{w}/conflicts/{conflict}/resolve",
                                   json={"decision": "accept_new"})
                self.assertEqual(response.status_code, 500, "Injected resolution failure is hidden")
                current = request(self.a, "GET", f"/world/{w}/entities/{entity}").json()
                self.assertEqual(current["properties"], {"age": 20},
                                 "Entity update commits despite conflict resolution failure")
                claim = request(self.a, "GET", f"/world/{w}/conflicts/{conflict}").json()
                self.assertFalse(claim["resolved"], "Failed transaction still resolves the conflict")
                self.assertIsNone(claim["decision"], "Failed transaction stores a decision")
            finally:
                cypher(f"DROP CONSTRAINT ON (n:{label}) ASSERT n.decision IS UNIQUE")

    def test_conflict_resolution_keeps_existing_fact(self):
        with self.conflict_resolution_fixture() as (w, entity, conflict):
            path = f"/world/{w}/conflicts/{conflict}"
            payload = {"decision": "keep_existing"}
            before = request(self.a, "GET", f"/world/{w}/entities/{entity}").json()
            self.assert_denied(request(self.b, "POST", path + "/resolve", json=payload))
            self.assertEqual(request(requests.Session(), "POST", path + "/resolve", json=payload).status_code,
                             401, "Anonymous caller can resolve a private conflict")
            still_open = request(self.a, "GET", path).json()
            self.assertFalse(still_open["resolved"], "Rejected callers still change the conflict")
            self.assertIsNone(still_open["decision"], "Older conflict unexpectedly has a decision")
            response = request(self.a, "POST", path + "/resolve", json=payload)
            self.assertEqual(response.status_code, 200, f"Resolution route fails: {response.text[:300]}")
            resolved = response.json()
            self.assertTrue(resolved["resolved"], "Successful resolution leaves the conflict open")
            self.assertEqual(resolved["decision"], "keep_existing", "Resolution hides the writer's decision")
            self.assertEqual((resolved["existing_value"], resolved["new_value"]), (20, 30),
                             "Resolution destroys either disputed claim")
            reread = request(self.a, "GET", path)
            self.assertEqual(reread.json(), resolved, "Decision disappears on a later read")
            retried = request(self.a, "POST", path + "/resolve", json=payload)
            self.assertEqual(retried.status_code, 200, "Retry fails after successful resolution")
            self.assertEqual(retried.json(), resolved, "Retry changes the saved decision")
            different = request(self.a, "POST", path + "/resolve", json={"decision": "accept_new"})
            self.assertEqual(different.status_code, 409, "Accept-new overwrites an existing keep-existing decision")
            self.assertEqual(request(self.a, "GET", f"/world/{w}/entities/{entity}").json(), before,
                             "Keep-existing resolution changes the saved entity")
            unresolved = self.graph_page(w, "conflicts", resolved="false")
            self.assertEqual(unresolved["data"], [], "Resolved conflict remains in the unresolved list")
            closed = self.graph_page(w, "conflicts", resolved="true")
            self.assertEqual([c["id"] for c in closed["data"]], [conflict], "Resolved conflict is lost from review")

    def test_conflict_resolution_rejects_invalid_requests(self):
        with self.conflict_resolution_fixture() as (w, _, conflict):
            path = f"/world/{w}/conflicts/{conflict}/resolve"
            for payload in ({}, {"decision": "merge"}, {"decision": "keepexisting"},
                            {"decision": None}, {"decision": 1}):
                with self.subTest(payload=payload):
                    response = request(self.a, "POST", path, json=payload)
                    self.assertEqual(response.status_code, 422,
                                     "Unsupported or missing decision reaches the resolution write")
            malformed = request(self.a, "POST", path, data="{", headers={"Content-Type": "application/json"})
            self.assertEqual(malformed.status_code, 422, "Malformed JSON reaches resolution logic")
            ambiguous = request(self.a, "POST", path, json={"decision": "accept_new"})
            self.assertEqual(ambiguous.status_code, 422, "Unmapped conflict field silently creates or changes a property")
            unchanged = request(self.a, "GET", f"/world/{w}/conflicts/{conflict}").json()
            self.assertFalse(unchanged["resolved"], "Invalid request resolves the conflict")
            self.assertIsNone(unchanged["decision"], "Invalid request records a decision")
            payload = {"decision": "keep_existing"}
            for url in (f"/world/{w}/conflicts/{uuid.uuid4()}/resolve",
                        f"/world/{self.w}/conflicts/{conflict}/resolve",
                        f"/world/{uuid.uuid4()}/conflicts/{conflict}/resolve"):
                self.assertEqual(request(self.a, "POST", url, json=payload).status_code, 404,
                                 "Resolution ignores the requested world or missing conflict")

    def graph_page(self, w, suffix, **params):
        response = request(self.a, "GET", f"/world/{w}/{suffix}", params=params)
        self.assertEqual(response.status_code, 200,
                         f"Populated graph request fails: {response.text[:300]}")
        return response.json()

    def test_populated_entity_pages_preserve_filters(self):
        with self.populated_graph() as (w, nodes, _):
            first = self.graph_page(w, "entities", limit=1, kind="character")
            second = self.graph_page(w, "entities", limit=1, kind="character", cursor=first["next_cursor"])
            self.assertEqual([e["id"] for e in first["data"] + second["data"]], nodes[:2],
                             "Filtered pages repeat, skip, or include unrelated entities")
            self.assertEqual(first["next_cursor"], nodes[0], "Cursor does not identify the returned entity")
            self.assertIsNone(second["next_cursor"], "Exact-size final page advertises more entities")
            all_nodes = self.graph_page(w, "entities", limit=3)
            self.assertEqual([e["id"] for e in all_nodes["data"]], nodes, "Unfiltered listing drops entities")
            self.assertIsNone(all_nodes["next_cursor"], "Exact-size unfiltered page advertises more entities")
            empty = self.graph_page(w, "entities", kind="unknown")
            self.assertEqual(empty["data"], [], "Unknown kind returns unrelated entities")
            self.assertIsNone(empty["next_cursor"], "Empty filter produces a cursor")

    def test_populated_graph_pages_keep_edge_endpoints(self):
        with self.populated_graph() as (w, nodes, edges):
            first = self.graph_page(w, "graph", limit=2)
            second = self.graph_page(w, "graph", limit=2, cursor=first["next_cursor"])
            self.assertEqual([n["id"] for n in first["data"]["nodes"]], nodes[:2], "Graph loses stable node order")
            self.assertEqual({e["id"] for e in first["data"]["edges"]}, {edges[0], edges[1], edges[3]},
                             "Graph drops internal edges or exposes edges outside its page")
            for edge in first["data"]["edges"]:
                self.assertIn(edge["from_id"], nodes[:2], "Edge source is missing from its graph page")
                self.assertIn(edge["to_id"], nodes[:2], "Edge target is missing from its graph page")
            self.assertEqual([n["id"] for n in second["data"]["nodes"]], nodes[2:], "Graph cursor skips its next node")
            self.assertEqual(second["data"]["edges"], [], "Last graph page includes cross-page edges")
            self.assertIsNone(second["next_cursor"], "Last graph page advertises more nodes")

    def test_populated_neighbor_pages_preserve_parallel_edges(self):
        with self.populated_graph() as (w, nodes, edges):
            suffix = f"entities/{nodes[0]}/neighbors"
            first = self.graph_page(w, suffix, limit=2)
            second = self.graph_page(w, suffix, limit=2, cursor=first["next_cursor"])
            self.assertEqual([e["id"] for e in first["data"]["edges"] + second["data"]["edges"]], edges,
                             "Neighbor pages skip parallel or reverse-direction relationships")
            self.assertEqual(first["next_cursor"], edges[1], "Neighbor cursor uses a node instead of an edge")
            self.assertIsNone(second["next_cursor"], "Final neighbor page advertises extra edges")
            incoming = first["data"]["edges"][1]
            self.assertEqual((incoming["from_id"], incoming["to_id"]), (nodes[1], nodes[0]),
                             "Neighbor traversal reverses a saved relationship")
            filtered = self.graph_page(w, suffix, kind="character", rel_type="MENTORS", limit=10)
            self.assertEqual([e["id"] for e in filtered["data"]["edges"]], [edges[1], edges[3]],
                             "Combined relationship and kind filters lose valid neighbors")
            empty = self.graph_page(w, suffix, kind="place", rel_type="MENTORS")
            self.assertEqual(empty["data"], {"nodes": [], "edges": []}, "Combined filters return unrelated neighbors")

    def test_populated_graph_record_reads_enforce_world_scope(self):
        with self.populated_graph() as (w, nodes, edges):
            for suffix, record_id in ((f"entities/{nodes[0]}", nodes[0]),
                                      (f"relationships/{edges[0]}", edges[0])):
                response = request(self.a, "GET", f"/world/{w}/{suffix}")
                self.assertEqual(response.status_code, 200, "Owner cannot inspect a saved graph record")
                self.assertEqual(response.json()["id"], record_id, "Lookup returns a different graph record")
                self.assertEqual(request(self.a, "GET", f"/world/{self.w}/{suffix}").status_code, 404,
                                 "Record lookup ignores the requested world")
                self.assert_denied(request(self.b, "GET", f"/world/{w}/{suffix}"))
            self.assert_denied(request(self.b, "GET", f"/world/{w}/graph"))
            self.assert_denied(request(self.b, "GET", f"/world/{w}/entities/{nodes[0]}/neighbors"))

    def test_world_graph_read_routes_enforce_access(self):
        missing = str(uuid.uuid4())
        endpoints = {
            "graph": (200, {"nodes": [], "edges": []}),
            "entities": (200, []),
            "entities/search?query=archive": (200, []),
            f"entities/{missing}": (404, None),
            f"entities/{missing}/neighbors": (404, None),
            f"relationships/{missing}": (404, None),
            "conflicts": (200, []),
            f"conflicts/{missing}": (404, None),
        }
        for suffix, (status, data) in endpoints.items():
            path = f"/world/{self.w}/{suffix}"
            with self.subTest(path=path):
                response = request(self.a, "GET", path)
                self.assertEqual(response.status_code, status,
                                 f"World read route fails: {response.text[:300]}")
                if status == 200:
                    self.assertEqual(response.json()["data"], data,
                                     "Empty world returns unexpected graph data")
                    self.assertIsNone(response.json()["next_cursor"],
                                      "Empty world advertises another page")
                self.assert_denied(request(self.b, "GET", path))
                self.assertIn(request(requests.Session(), "GET", path).status_code,
                              (401, 403), "Anonymous caller reaches world graph data")

    def test_world_graph_read_routes_validate_queries(self):
        cases = [
            "graph?limit=0", "graph?limit=101", "graph?limit=bad",
            "graph?cursor=invalid", "entities?limit=0", "entities?kind=",
            "entities/search", "entities/search?query=", "entities/search?query=archive&limit=101",
            "conflicts?resolved=bad", "conflicts?limit=0", "conflicts?cursor=invalid",
            f"entities/{uuid.uuid4()}/neighbors?limit=0",
        ]
        for suffix in cases:
            with self.subTest(query=suffix):
                response = request(self.a, "GET", f"/world/{self.w}/{suffix}")
                self.assertEqual(response.status_code, 422,
                                 f"Invalid world query reaches service logic: {response.text[:300]}")

    def test_unauthenticated_routes_reject_requests(self):
        for method, path, payload in [
            ("GET", "/world", None), ("GET", f"/world/{self.w}", None),
            ("POST", "/world", {"name": "x", "description": "x"}),
            ("GET", f"/world/{self.w}/conversations/{self.c}", None),
            ("POST", f"/world/{self.w}/conversations/{self.c}/messages", {"content": "x"}),
        ]:
            with self.subTest(method=method, path=path):
                r = requests.request(method, BASE + path, json=payload, timeout=10)
                self.assertEqual(r.status_code, 401, "Anonymous users can reach private backend actions")

    def test_foreign_world_read_is_denied(self):
        self.assert_denied(request(self.b, "GET", f"/world/{self.w}"))

    def test_foreign_world_update_is_denied(self):
        self.assert_denied(request(self.b, "PATCH", f"/world/{self.w}", json={"new_name": "stolen"}))

    def test_foreign_world_delete_is_denied(self):
        self.assert_denied(request(self.b, "DELETE", f"/world/{self.w}"))

    def test_foreign_conversation_read_is_denied(self):
        self.assert_denied(request(self.b, "GET", f"/world/{self.w}/conversations/{self.c}"))

    def test_foreign_conversation_delete_is_denied(self):
        self.assert_denied(request(self.b, "DELETE", f"/world/{self.w}/conversations/{self.c}"))

    def test_foreign_conversation_chat_is_denied(self):
        self.assert_denied(request(self.b, "POST", f"/world/{self.w}/conversations/{self.c}/messages", json={"content": "x"}))

    def test_foreign_world_conversation_creation_is_denied(self):
        self.assert_denied(request(self.b, "POST", f"/world/{self.w}/conversations", json={"title": "intrusion"}))

    def test_missing_world_conversation_creation_returns_not_found(self):
        r = request(self.a, "POST", f"/world/{uuid.uuid4()}/conversations", json={"title": "x"})
        self.assertEqual(r.status_code, 404, "A missing world is reported as a backend failure")

    def test_world_with_conversations_can_be_deleted(self):
        w = world(self.a)
        c = conversation(self.a, w)
        r = request(self.a, "DELETE", f"/world/{w}")
        self.assertEqual(r.status_code, 204, f"Normal world deletion fails: {r.text}")
        self.assertEqual(request(self.a, "GET", f"/world/{w}/conversations/{c}").status_code, 404,
                         "World deletion leaves accessible conversation records")

    def test_scheduled_cleanup_removes_only_the_deleted_world(self):
        deleted = world(self.a, "Scheduled cleanup QA")
        kept = world(self.a, "Keep this QA world")
        conversation(self.a, deleted)
        try:
            cypher(f"""CREATE (e:Entity {{world_id: '{deleted}', name: 'QA entity'}}),
            (c:Conflict {{world_id: '{deleted}'}}), (h:Conversation {{world_id: '{deleted}'}}),
            (e)-[:HAS_CONFLICT]->(c), (e)-[:SOURCE]->(h),
            (:Entity {{world_id: '{kept}', name: 'QA survivor'}})""")
            r = request(self.a, "DELETE", f"/world/{deleted}")
            self.assertEqual(r.status_code, 204, "World deletion fails before it can queue graph cleanup")
            deadline = time.monotonic() + 20
            nodes = jobs = None
            while time.monotonic() < deadline:
                snapshot = cypher(f"MATCH (n {{world_id: '{deleted}'}}) RETURN count(n) AS remaining")
                nodes = int(snapshot.strip().splitlines()[-1].strip('"'))
                jobs = int(sql(f"SELECT count(*) FROM world_cleanup_jobs WHERE world_id = '{deleted}'"))
                if nodes == 0 and jobs == 0:
                    break
                time.sleep(.25)
            self.assertEqual(nodes, 0, "Scheduled cleanup leaves deleted-world graph nodes behind")
            self.assertEqual(jobs, 0, "Successful graph cleanup never removes its pending job")
            survivor = cypher(f"MATCH (n {{world_id: '{kept}'}}) RETURN count(n) AS remaining")
            self.assertEqual(int(survivor.strip().splitlines()[-1].strip('"')), 1,
                             "Cleanup destroys graph data belonging to a different world")
        finally:
            cypher(f"MATCH (n) WHERE n.world_id IN ['{deleted}', '{kept}'] DETACH DELETE n")

    def test_blank_world_name_is_rejected(self):
        for name in ("", " \t\n", "\u2003"):
            with self.subTest(name=repr(name)):
                r = request(self.a, "POST", "/world", json={"name": name, "description": ""})
                self.assertEqual(r.status_code, 422, "Blank world names enter saved user data")

    def test_chat_length_boundaries_reject_invalid_input(self):
        for content in ("", "x" * 10001):
            with self.subTest(length=len(content)):
                r = request(self.a, "POST", f"/world/{self.w}/conversations/{self.c}/messages", json={"content": content})
                self.assertEqual(r.status_code, 422, "Invalid chat sizes reach paid model requests")

    def test_malformed_chat_payloads_are_rejected(self):
        for payload in ({}, {"content": None}, {"content": 4}, {"content": []}):
            with self.subTest(payload=payload):
                r = request(self.a, "POST", f"/world/{self.w}/conversations/{self.c}/messages", json=payload)
                self.assertEqual(r.status_code, 422, "Malformed chat data reaches model processing")

    def test_patch_is_allowed_by_browser_cors(self):
        r = requests.options(BASE + f"/world/{self.w}", headers={
            "Origin": "http://localhost:3000", "Access-Control-Request-Method": "PATCH",
            "Access-Control-Request-Headers": "content-type"}, timeout=10)
        allowed = r.headers.get("Access-Control-Allow-Methods", "")
        self.assertIn("PATCH", allowed, "Browsers cannot update worlds because preflight rejects PATCH")

    def test_world_text_is_normalized_before_storage(self):
        r = request(self.a, "POST", "/world", json={
            "name": "  Cafe\u0301\u200b   Reach  ", "description": "  first\r\nsecond  "})
        self.assertEqual(r.status_code, 200, f"Valid normalized text is rejected: {r.text}")
        w = r.json()["id"]
        saved = request(self.a, "GET", f"/world/{w}").json()
        self.assertEqual(saved["name"], "Café Reach", "Unnormalized names reach saved worlds")
        self.assertEqual(saved["description"], "first\nsecond", "Description line breaks change during storage")

    def test_invalid_world_update_does_not_change_saved_name(self):
        w = world(self.a, "Unchanged name")
        for name in ("   ", "safe\u202eevil", "safe\x00evil", "x" * 256):
            with self.subTest(name=repr(name)):
                r = request(self.a, "PATCH", f"/world/{w}", json={"new_name": name})
                self.assertEqual(r.status_code, 422, "Invalid names cross the update boundary")
        saved = request(self.a, "GET", f"/world/{w}").json()
        self.assertEqual(saved["name"], "Unchanged name", "Rejected updates change saved data")

    def test_blank_conversation_title_is_rejected(self):
        r = request(self.a, "POST", f"/world/{self.w}/conversations", json={"title": " \u200b "})
        self.assertEqual(r.status_code, 422, "Invisible-only titles enter saved conversations")

    def test_blank_chat_is_rejected_before_model_processing(self):
        r = request(self.a, "POST", f"/world/{self.w}/conversations/{self.c}/messages", json={"content": " \n\u200b "})
        self.assertEqual(r.status_code, 422, "Blank chat reaches paid model processing")

    def test_duplicate_signup_returns_conflict(self):
        r = request(self.a, "POST", "/auth/signup", json={
            "username": "QA", "email": self.emails[0], "password": PASSWORD})
        self.assertEqual(r.status_code, 409, f"Duplicate signup becomes a database outage error: {r.text}")

    def test_login_does_not_disclose_account_existence(self):
        responses = [requests.post(BASE + "/auth/login", json={"email": email, "password": "Wrong-pass-123!"}, timeout=10)
                     for email in (self.emails[0], f"missing-{uuid.uuid4().hex}@example.com")]
        self.assertEqual((responses[0].status_code, responses[0].text),
                         (responses[1].status_code, responses[1].text),
                         "Attackers can identify registered email addresses through login errors")

    def test_logout_invalidates_copied_cookie(self):
        session = requests.Session()
        request(session, "POST", "/auth/login", json={"email": self.emails[0], "password": PASSWORD}).raise_for_status()
        copied = session.cookies.get_dict()
        request(session, "POST", "/auth/logout").raise_for_status()
        r = requests.get(BASE + "/auth/me", cookies=copied, timeout=10)
        self.assertEqual(r.status_code, 401, "Logged-out sessions remain usable by anyone holding the old cookie")

    def test_concurrent_world_updates_do_not_lose_other_fields(self):
        w = world(self.a)
        cookies = self.a.cookies.get_dict()
        def patch(payload):
            return requests.patch(BASE + f"/world/{w}", json=payload, cookies=cookies, timeout=10)
        with concurrent.futures.ThreadPoolExecutor(max_workers=2) as pool:
            results = list(pool.map(patch, [{"new_name": "changed"}, {"new_description": "new description"}]))
        self.assertTrue(all(r.ok for r in results), "Concurrent world updates fail")
        value = request(self.a, "GET", f"/world/{w}").json()
        self.assertEqual((value["name"], value["description"]), ("changed", "new description"),
                         "Concurrent edits silently discard changes to separate fields")

    @unittest.skipUnless(AI, "set QA_AI=1 to run paid model tests")
    def test_failed_assistant_save_does_not_report_success(self):
        c = conversation(self.a, self.w)
        function = "qa_fail_" + uuid.uuid4().hex
        sql(f"""CREATE FUNCTION {function}() RETURNS trigger LANGUAGE plpgsql AS $$
        BEGIN IF NEW.id = '{c}'::uuid AND jsonb_array_length(NEW.messages) > 1
        THEN RAISE EXCEPTION 'QA assistant save rejected'; END IF; RETURN NEW; END $$;
        CREATE TRIGGER {function} BEFORE UPDATE ON conversations FOR EACH ROW EXECUTE FUNCTION {function}();""")
        try:
            r = request(self.a, "POST", f"/world/{self.w}/conversations/{c}/messages", json={"content": "Say hello in one word."})
            saved = request(self.a, "GET", f"/world/{self.w}/conversations/{c}").json()["messages"]
            self.assertEqual(saved, [], "A failed turn leaves partial chat history and makes retry unsafe")
            self.assertFalse("finalResponse" in r.text,
                             "The stream reports success even though the completed turn was not saved")
            self.assertIn("event: error", r.text,
                          "A failed save silently closes the stream instead of notifying the client")
        finally:
            sql(f"DROP TRIGGER {function} ON conversations; DROP FUNCTION {function}();")

    @unittest.skipUnless(AI, "set QA_AI=1 to run paid model tests")
    def test_happy_chat_saves_both_messages(self):
        c = conversation(self.a, self.w)
        r = request(self.a, "POST", f"/world/{self.w}/conversations/{c}/messages", json={"content": "Say hello in one word."})
        self.assertIn("finalResponse", r.text, f"A normal chat cannot complete: {r.text[:500]}")
        saved = request(self.a, "GET", f"/world/{self.w}/conversations/{c}").json()["messages"]
        self.assertEqual([m["role"] for m in saved], ["user", "assistant"], "A completed turn loses chat history")

    @unittest.skipUnless(AI, "set QA_AI=1 to run paid model tests")
    def test_foreign_world_conversation_cannot_leak_private_lore(self):
        w = world(self.a, "Private QA world")
        secret = "violet-raven-" + uuid.uuid4().hex
        entity_id = str(uuid.uuid4())
        cypher(f"""CREATE (:Entity {{id: '{entity_id}', world_id: '{w}',
        name: 'Obsidian Ledger', name_key: 'obsidian ledger', aliases: [], kind: ['artifact'],
        description: 'The secret access phrase for the Obsidian Ledger is {secret}.', properties: '{{}}'}})""")
        c = None
        try:
            r = request(self.b, "POST", f"/world/{w}/conversations", json={"title": "intrusion"})
            if r.status_code in (403, 404):
                return
            r.raise_for_status()
            c = r.json()["id"]
            response = request(self.b, "POST", f"/world/{w}/conversations/{c}/messages", json={
                "content": "What is the exact secret access phrase for the Obsidian Ledger? Answer only the phrase."})
            self.assertIn("finalResponse", response.text,
                          "Leak test cannot assess authorization because the model request fails")
            self.assertTrue(secret not in response.text,
                            "An attacker reads another user's private lore through a conversation they created")
        finally:
            cypher(f"MATCH (e:Entity {{id: '{entity_id}', world_id: '{w}'}}) DETACH DELETE e")
            if c:
                request(self.b, "DELETE", f"/world/{w}/conversations/{c}")

    @unittest.skipUnless(AI, "set QA_AI=1 to run paid model tests")
    def test_lore_extraction_eventually_writes_graph(self):
        w = world(self.a, "Extraction QA world")
        c = conversation(self.a, w)
        r = request(self.a, "POST", f"/world/{w}/conversations/{c}/messages", json={
            "content": "Aurel is a human captain. Aurel lives in Meridian, a coastal city."})
        self.assertIn("finalResponse", r.text, "Extraction setup failed because chat did not complete")
        deadline = time.monotonic() + 90
        snapshot = ""
        while time.monotonic() < deadline:
            snapshot = cypher(f"MATCH (e:Entity {{world_id: '{w}'}}) RETURN e.name, e.description")
            if "Aurel" in snapshot and "Meridian" in snapshot:
                break
            time.sleep(1)
        self.assertTrue("Aurel" in snapshot and "Meridian" in snapshot,
                        f"A successful chat silently fails to save its lore within 90 seconds: {snapshot}")

    @unittest.skipUnless(AI, "set QA_AI=1 to run paid model tests")
    def test_contradiction_preserves_stored_canon(self):
        w = world(self.a, "Conflict QA world")
        c = conversation(self.a, w)
        entity_id = str(uuid.uuid4())
        cypher(f"""CREATE (:Entity {{id: '{entity_id}', world_id: '{w}',
        name: 'Aurel', name_key: 'aurel', aliases: [], kind: ['character'],
        description: 'Aurel was born in the Ashlands.', properties: '{{"origin":"Ashlands"}}'}})""")
        r = request(self.a, "POST", f"/world/{w}/conversations/{c}/messages", json={
            "content": "Aurel was born in the Northern Wastes."})
        self.assertIn("finalResponse", r.text, "Conflict test setup fails because chat cannot finish")
        deadline = time.monotonic() + 90
        conflicts = ""
        while time.monotonic() < deadline:
            conflicts = cypher(f"MATCH (c:Conflict {{world_id: '{w}'}}) RETURN c.field, c.new_value")
            if "Northern Wastes" in conflicts:
                break
            time.sleep(1)
        self.assertIn("Northern Wastes", conflicts,
                      "A contradiction is not recorded for the writer to resolve")
        stored = cypher(f"MATCH (e:Entity {{id: '{entity_id}'}}) RETURN e.properties")
        self.assertIn("Ashlands", stored, "An unresolved contradiction overwrites the writer's stored canon")


if __name__ == "__main__":
    unittest.main()
