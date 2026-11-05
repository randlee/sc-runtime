"""Release-fixture identity checks shared by checkout and registry qualification."""
import json
import unittest

PUBLIC = {"sc-config", "sc-transport", "sc-command", "sc-runtime", "sc-cron", "sc-cron-commands"}
ECOSYSTEM = {"rmcp", "utoipa", "utoipa-axum"}


def check_daemon_closure(metadata, *, registry):
    """Follow only normal reachable edges; dev-only identities cannot conceal collisions."""
    packages = {item["id"]: item for item in metadata["packages"]}
    nodes = {item["id"]: item for item in metadata["resolve"]["nodes"]}
    daemon = next(item["id"] for item in packages.values() if item["name"].endswith("-daemon"))
    pending, seen = [daemon], set()
    while pending:
        identity = pending.pop()
        if identity in seen:
            continue
        seen.add(identity)
        pending.extend(dep["pkg"] for dep in nodes[identity]["deps"]
                       if any(kind["kind"] in (None, "normal") for kind in dep["dep_kinds"]))
    reachable = [packages[identity] for identity in seen]
    for name in ECOSYSTEM:
        identities = {(p["name"], p["version"], p.get("source")) for p in reachable if p["name"] == name}
        assert len(identities) == 1, (name, identities)
    for name in PUBLIC:
        matches = [p for p in reachable if p["name"] == name]
        assert len(matches) == 1, (name, matches)
        assert matches[0]["version"] == "0.2.0", matches[0]
        if registry:
            assert (matches[0].get("source") or "").startswith("registry+"), matches[0]
    return {p["name"]: p["version"] for p in reachable if p["name"] in PUBLIC | ECOSYSTEM}


class IdentityGuardTests(unittest.TestCase):
    def test_reachable_duplicate_fails_even_when_a_root_identity_is_valid(self):
        packages = [{"id": "daemon", "name": "sample-daemon", "version": "0.1.1", "source": None}]
        packages += [{"id": name, "name": name, "version": "0.2.0", "source": "registry+index"}
                     for name in PUBLIC | ECOSYSTEM]
        packages.append({"id": "old-rmcp", "name": "rmcp", "version": "0.8.0", "source": "registry+index"})
        nodes = [{"id": p["id"], "deps": []} for p in packages]
        nodes[0]["deps"] = [{"pkg": p["id"], "dep_kinds": [{"kind": None}]} for p in packages[1:]]
        with self.assertRaises(AssertionError):
            check_daemon_closure({"packages": packages, "resolve": {"nodes": nodes}}, registry=True)
