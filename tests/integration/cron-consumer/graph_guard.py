"""Normal dependency graph proof, kept separate from dev test feature unification."""
import json
import pathlib
import subprocess

manifest = pathlib.Path(__file__).with_name("Cargo.toml")
for features in ([], ["--features", "monty"]):
    result = subprocess.run(["cargo", "tree", "--manifest-path", str(manifest), "--edges", "normal", "--prefix", "none", "--format", "{p}", "--offline", *features], capture_output=True, text=True, check=True)
    names = {line.split()[0] for line in result.stdout.splitlines() if line.strip()}
    forbidden = {"axum", "rmcp", "clap", "sc-observability", "sc-command-server", "sc-runtime", "atm", "jev"}
    assert not names & forbidden, names & forbidden
    assert ("monty" in names) == bool(features)
    metadata = json.loads(subprocess.run(["cargo", "metadata", "--manifest-path", str(manifest), "--no-deps", "--format-version", "1", "--offline"], capture_output=True, text=True, check=True).stdout)
    consumer = metadata["packages"][0]
    sdk_edges = [edge["name"] for edge in consumer["dependencies"] if edge["name"].startswith("sc-")]
    assert set(sdk_edges) == {"sc-cron"}, sdk_edges
print("PASS default and Monty normal graphs; only sc-cron is a direct SDK edge")
