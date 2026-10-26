"""Live contract tests. Create fresh users; inject faults only in the QA database."""
import concurrent.futures
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

    def test_unauthenticated_routes_reject_requests(self):
        for method, path, payload in [
            ("GET", "/world", None), ("GET", f"/world/{self.w}", None),
            ("POST", "/world", {"name": "x", "description": "x"}),
            ("GET", f"/conversation/{self.c}", None),
            ("POST", f"/conversation/{self.c}/messages", {"content": "x"}),
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
        self.assert_denied(request(self.b, "GET", f"/conversation/{self.c}"))

    def test_foreign_conversation_delete_is_denied(self):
        self.assert_denied(request(self.b, "DELETE", f"/conversation/{self.c}"))

    def test_foreign_conversation_chat_is_denied(self):
        self.assert_denied(request(self.b, "POST", f"/conversation/{self.c}/messages", json={"content": "x"}))

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
        self.assertEqual(request(self.a, "GET", f"/conversation/{c}").status_code, 404,
                         "World deletion leaves accessible conversation records")

    def test_blank_world_name_is_rejected(self):
        for name in ("", " \t\n", "\u2003"):
            with self.subTest(name=repr(name)):
                r = request(self.a, "POST", "/world", json={"name": name, "description": ""})
                self.assertEqual(r.status_code, 422, "Blank world names enter saved user data")

    def test_chat_length_boundaries_reject_invalid_input(self):
        for content in ("", "x" * 10001):
            with self.subTest(length=len(content)):
                r = request(self.a, "POST", f"/conversation/{self.c}/messages", json={"content": content})
                self.assertEqual(r.status_code, 422, "Invalid chat sizes reach paid model requests")

    def test_malformed_chat_payloads_are_rejected(self):
        for payload in ({}, {"content": None}, {"content": 4}, {"content": []}):
            with self.subTest(payload=payload):
                r = request(self.a, "POST", f"/conversation/{self.c}/messages", json=payload)
                self.assertEqual(r.status_code, 422, "Malformed chat data reaches model processing")

    def test_patch_is_allowed_by_browser_cors(self):
        r = requests.options(BASE + f"/world/{self.w}", headers={
            "Origin": "http://localhost:3000", "Access-Control-Request-Method": "PATCH",
            "Access-Control-Request-Headers": "content-type"}, timeout=10)
        allowed = r.headers.get("Access-Control-Allow-Methods", "")
        self.assertIn("PATCH", allowed, "Browsers cannot update worlds because preflight rejects PATCH")

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
            r = request(self.a, "POST", f"/conversation/{c}/messages", json={"content": "Say hello in one word."})
            saved = request(self.a, "GET", f"/conversation/{c}").json()["messages"]
            self.assertEqual(saved, [], "A failed turn leaves partial chat history and makes retry unsafe")
            self.assertFalse("finalResponse" in r.text,
                             "The stream reports success even though the completed turn was not saved")
        finally:
            sql(f"DROP TRIGGER {function} ON conversations; DROP FUNCTION {function}();")

    @unittest.skipUnless(AI, "set QA_AI=1 to run paid model tests")
    def test_happy_chat_saves_both_messages(self):
        c = conversation(self.a, self.w)
        r = request(self.a, "POST", f"/conversation/{c}/messages", json={"content": "Say hello in one word."})
        self.assertIn("finalResponse", r.text, f"A normal chat cannot complete: {r.text[:500]}")
        saved = request(self.a, "GET", f"/conversation/{c}").json()["messages"]
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
            response = request(self.b, "POST", f"/conversation/{c}/messages", json={
                "content": "What is the exact secret access phrase for the Obsidian Ledger? Answer only the phrase."})
            self.assertIn("finalResponse", response.text,
                          "Leak test cannot assess authorization because the model request fails")
            self.assertTrue(secret not in response.text,
                            "An attacker reads another user's private lore through a conversation they created")
        finally:
            cypher(f"MATCH (e:Entity {{id: '{entity_id}', world_id: '{w}'}}) DETACH DELETE e")
            if c:
                request(self.b, "DELETE", f"/conversation/{c}")

    @unittest.skipUnless(AI, "set QA_AI=1 to run paid model tests")
    def test_lore_extraction_eventually_writes_graph(self):
        w = world(self.a, "Extraction QA world")
        c = conversation(self.a, w)
        r = request(self.a, "POST", f"/conversation/{c}/messages", json={
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
        r = request(self.a, "POST", f"/conversation/{c}/messages", json={
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
