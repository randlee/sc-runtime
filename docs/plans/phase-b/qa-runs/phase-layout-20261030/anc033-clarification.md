# ANC-033 clarification after relocation verification

At pushed head `28ae7369e326534d5b27e1943c7078ff6ade1357`, quality-mgr confirmed the relocation preserved all approved content. Fresh live planning gates passed b-13, b-14, b-15, b-17 and b-18. The b-16 run `plan-20261030T175925Z-b-16` returned ANC-033 `underspecified`, leaning satisfied, unstable: addressed=yes at 1.0; contradicted=no at 0.67 and 0.58, below the unchanged 0.6 threshold.

Triage action: replace “Monty source is application-supplied in-process source. MVP has no signing, separate script approval mechanism, or consumer option enabling one.” with “MVP executes application-supplied in-process Monty source without signature validation or separate script approval; consumers cannot enable either gate.”

Authority predating the failed run: locked ANC-033 version 3 requires MVP execution without signing or separate approval; its deterministic check requires neither signatures nor approval configuration. ADR-RUN-0013 already requires execution without signatures. “No signing” could otherwise describe signature generation without expressly ruling out execution-time signature validation.

Quality-mgr independently recommended this shorter clarification as preservation of approved intent. The handler execution-risk boundary and non-goals remain unchanged. No anchors, quotes, scopes, exclusions, question overrides, fixtures, versions, thresholds, sprint ownership or acceptance requirements change. This is the sole approved-plan prose change after the relocation review. Preserve the failing run as evidence; verify the new pushed head with the same complete sprint and all 44 locked anchors.
