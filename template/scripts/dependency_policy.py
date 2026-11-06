"""Check the daemon's resolved normal dependency identities and ancestry.

Only the actual direct ``sc_cron`` SDK binding can own the approved private
Python scratch dependency. A second path to the same tempfile identity still
fails; package-name occurrence or global reachability is not an exemption.
"""

from pathlib import Path


def _normal_edges(node: dict) -> list[dict]:
    edges = []
    for dependency in node["deps"]:
        kinds = dependency["dep_kinds"]
        if not kinds or any(item["kind"] not in (None, "dev", "build") for item in kinds):
            raise ValueError("Cargo metadata has an invalid dependency kind")
        if any(item["kind"] is None for item in kinds):
            edges.append(dependency)
    return edges


def _active_normal_edges(packages: dict, nodes: dict, root: str, tree: str) -> dict:
    # Metadata can unify dev-only features into normal edges. Cargo tree -e
    # normal selects the daemon build graph; map its identities back to metadata
    # rather than trusting those extra edges or collapsing by package name.
    active = {}
    stack = []
    for line in tree.splitlines():
        depth_text, separator, display = line.partition("|")
        if not separator or not depth_text.isdecimal():
            raise ValueError("invalid machine-formatted normal dependency tree")
        depth = int(depth_text)
        display = display.removesuffix(" (*)")
        if depth == 0:
            candidates = [root]
        elif depth > len(stack):
            raise ValueError("normal dependency tree skipped an ancestry level")
        else:
            parent = stack[depth - 1]
            candidates = {edge["pkg"] for edge in _normal_edges(nodes[parent])}
        matched = [identity for identity in candidates if display ==
                   f"{packages[identity]['name']} v{packages[identity]['version']}"
                   or display.startswith(
                       f"{packages[identity]['name']} v{packages[identity]['version']} ("
                   )]
        if len(matched) > 1:
            matched = [identity for identity in matched if display.endswith(
                f" ({Path(packages[identity]['manifest_path']).parent})"
            )]
        if len(matched) != 1:
            raise ValueError("normal dependency package identity is missing or ambiguous")
        identity = matched[0]
        if depth:
            edges = active.setdefault(stack[depth - 1], {})
            for edge in _normal_edges(nodes[stack[depth - 1]]):
                if edge["pkg"] == identity:
                    edges[(edge["pkg"], edge["name"])] = edge
        active.setdefault(identity, {})
        stack[depth:] = [identity]
    if root not in active:
        raise ValueError("normal dependency tree omitted the daemon")
    return {identity: list(edges.values()) for identity, edges in active.items()}


def daemon_scratch_errors(metadata: dict, daemon_manifest: Path, tree: str) -> list[str]:
    """Return violations; dev/build edges cannot authorize a normal path."""
    packages = {package["id"]: package for package in metadata["packages"]}
    nodes = {node["id"]: node for node in metadata["resolve"]["nodes"]}
    roots = [
        package["id"] for package in packages.values()
        if Path(package["manifest_path"]).resolve() == daemon_manifest.resolve()
    ]
    if len(roots) != 1:
        raise ValueError("Cargo metadata must identify exactly one daemon manifest")
    root = roots[0]
    active = _active_normal_edges(packages, nodes, root, tree)
    root_edges = active[root]
    sdk_declared = any(
        dependency["name"] == "sc-cron"
        and dependency["kind"] is None
        and (dependency.get("rename") or "sc-cron").replace("-", "_") == "sc_cron"
        for dependency in packages[root]["dependencies"]
    )
    qualified = {
        edge["pkg"] for edge in root_edges
        if sdk_declared and edge["name"] == "sc_cron"
        and packages[edge["pkg"]]["name"] == "sc-cron"
    }
    reachable = set()
    pending = [root]
    errors = []
    while pending:
        parent = pending.pop()
        if parent in reachable:
            continue
        reachable.add(parent)
        for edge in active[parent]:
            child = edge["pkg"]
            package = packages[child]
            if package["name"] == "tempdir":
                errors.append("tempdir is forbidden in the daemon normal dependency graph")
            elif package["name"] == "tempfile" and parent not in qualified:
                owner = packages[parent]
                errors.append(
                    "unapproved normal tempfile ancestry: "
                    f"{owner['name']} {owner['version']} -> tempfile {package['version']}"
                )
            pending.append(child)
    return sorted(set(errors))
