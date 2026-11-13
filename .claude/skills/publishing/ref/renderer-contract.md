# Renderer Contract

- Direct agent template rendering uses the `sc-compose` **CLI** (≥ repo floor in
  `.sc/generation-tools.json` when present; newer releases OK). Do not import
  `sc_compose` from an agent prompt or an inline Python command.
- **Python publish renderer** (separate): read exact `SC_COMPOSE_VERSION` in
  `.github/scripts/bootstrap_sc_compose.py`; the bootstrapped wheel must match.
  That pin is not the generation CLI floor.
- Package Python code (`install.py`, integration examples, and tests) uses only
  the interpreter printed by `bootstrap_sc_compose.py`. That bootstrapper
  installs or replaces the wheel until it exactly matches
  `SC_COMPOSE_VERSION`.
- Homebrew and Scoop channel workflows bootstrap that same pinned renderer on
  the runner via `.github/actions/setup-renderer`. They must not extract or
  execute the consumer product binary as a template renderer.
