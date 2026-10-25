# Intent - phase-current

Written 2026-10-25 by randlee. This page frames the anchors. Jev judges the anchors, not this page.

## What this plan is for

sc-runtime is a set of crates and a template for deploying a new application that can use its database immediately. General database commands are the bootstrap. The consumer disables that public surface and adds application commands that call the same backend operations.

## What success looks like

- A generated project exposes general database commands on the CLI, on MCP, and over HTTP, including `delete row <table> <id>`.
- The consumer can turn that surface off.
- An application command calls the same backend operation through its own CLI command, MCP tool, and HTTP route.

## What is deliberately out of scope

- A command registry inside the four library crates.
- Store-to-store cache sync inside the sc-runtime crates. That forwarding lives in the consumer adaptor.

## Anchors

Anchors for this plan live in `anchors.json` (record of truth) and are rendered to `anchors.md`. Requirements and ADRs that govern this plan are listed there too, under their own IDs.
