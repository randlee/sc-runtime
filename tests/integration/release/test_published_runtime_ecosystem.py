"""Release-fixture identity checks shared by checkout and registry qualification."""
import json
import unittest

PUBLIC = {"sc-config", "sc-transport", "sc-command", "sc-runtime", "sc-cron", "sc-cron-commands"}
CRATES_IO_SOURCE = "registry+https://github.com/rust-lang/crates.io-index"
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
            assert matches[0].get("source") == CRATES_IO_SOURCE, matches[0]
    return {p["name"]: p["version"] for p in reachable if p["name"] in PUBLIC | ECOSYSTEM}


class IdentityGuardTests(unittest.TestCase):
    def test_reachable_duplicate_fails_even_when_a_root_identity_is_valid(self):
        packages = [{"id": "daemon", "name": "sample-daemon", "version": "0.1.1", "source": None}]
        packages += [{"id": name, "name": name, "version": "0.2.0", "source": CRATES_IO_SOURCE}
                     for name in PUBLIC | ECOSYSTEM]
        packages.append({"id": "old-rmcp", "name": "rmcp", "version": "0.8.0", "source": CRATES_IO_SOURCE})
        nodes = [{"id": p["id"], "deps": []} for p in packages]
        nodes[0]["deps"] = [{"pkg": p["id"], "dep_kinds": [{"kind": None}]} for p in packages[1:]]
        with self.assertRaises(AssertionError):
            check_daemon_closure({"packages": packages, "resolve": {"nodes": nodes}}, registry=True)

    @staticmethod
    def metadata(source=CRATES_IO_SOURCE):
        packages = [{"id": "path+file:///consumer#sample-daemon@0.1.1",
                     "name": "sample-daemon", "version": "0.1.1", "source": None}]
        packages += [{"id": f"{source}#{name}@0.2.0", "name": name,
                      "version": "0.2.0", "source": source} for name in sorted(PUBLIC | ECOSYSTEM)]
        nodes = [{"id": p["id"], "deps": []} for p in packages]
        nodes[0]["deps"] = [{"pkg": p["id"], "dep_kinds": [{"kind": None}]} for p in packages[1:]]
        return {"packages": packages, "resolve": {"nodes": nodes}}

    def test_actual_crates_io_cargo_source_is_accepted(self):
        identities = check_daemon_closure(self.metadata(), registry=True)
        self.assertEqual(set(identities), PUBLIC | ECOSYSTEM)

    def test_alternate_lookalike_path_git_and_missing_sources_are_rejected(self):
        for source in ("registry+https://registry.example/index",
                       CRATES_IO_SOURCE + ".evil.example",
                       "registry+https://github.com/rust-lang/crates.io-index/",
                       "path+file:///checkout", "git+https://github.com/randlee/sc-runtime", None):
            with self.subTest(source=source), self.assertRaises(AssertionError):
                check_daemon_closure(self.metadata(source), registry=True)

    def test_one_source_mismatch_cannot_hide_among_valid_registry_packages(self):
        for name in PUBLIC:
            metadata = self.metadata()
            next(p for p in metadata["packages"] if p["name"] == name)["source"] = "registry+https://mirror.example/index"
            with self.subTest(name=name), self.assertRaises(AssertionError):
                check_daemon_closure(metadata, registry=True)
