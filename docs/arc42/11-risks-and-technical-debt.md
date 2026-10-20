# 11. Risks and Technical Debt

## Risks

| ID | Risk | Probability | Impact | Mitigation |
|---|---|---|---|---|
| R-001 | The real candidate adapters and the record path are never exercised end to end; three heterogeneous adapters can all return empty while fixture tests pass. | High | High | Record-mode E2E with a fake endpoint, plus the manual record-run acceptance gate (one real candidate/repository evidence file and a replay probe per installed candidate). |
| R-002 | The default musl Rust toolchain is broken and gnu `rustc` needs zlib. | High | Medium | `env.sh` pins `stable-x86_64-unknown-linux-gnu` and resolves zlib via system or Nix; every script sources it. |
| R-003 | A 10M-line monorepo exceeds a candidate's practical index time. | Medium | Medium | Manifest index deadlines; breach is an explicit `index_timeout` capability gap, never an invisible failure or zero. |
| R-004 | Candidate interfaces drift between the pinned version and current behavior. | Medium | Medium | Pinned versions/commits, adapter versions, recorded result cassettes, and a documented-interface probe. |
| R-005 | Candidate setup is degraded (missing SCIP indexer, language server, or embedding configuration), making the comparison unfair. | Medium | High | `setup.sh` installs and pins indexers/servers, configures `embeddingProvider: "custom"`, runs `ckb doctor`, records every version/deviation, and marks affected cells unavailable rather than degrading silently. |
| R-006 | A proposed AC-01 target is read as an accepted threshold. | Medium | Medium | `status: proposed` in `targets.yaml`, "gap to proposed target" labeling, no path choice in the report. |
| R-007 | CKB's Rust Tier 3 support makes a Rust comparison look like a quality difference. | Medium | Medium | Tier recorded as a capability/coverage note on every CKB Rust cell. |
| R-008 | Serena's regex pattern search is mistaken for natural-language retrieval. | Medium | Medium | NL cells `unavailable` with `no_natural_language_search`; any regex proxy is labelled and excluded from the NL aggregate. |
| R-009 | The report looks complete while no real measurement exists. | Low | High | Closed status model, coverage denominators, the manual record-run gate, and specs that assert concrete values. |
| R-010 | Recorded cassettes are missing, stale, or accidentally overwritten. | Low | High | Cassette fingerprints in the report, exit 2 on a miss, committed cassettes, temporary directories in the record E2E. |

## Technical debt

| ID | Item | Impact | Plan to address |
|---|---|---|---|
| TD-001 | Fixture and replay evidence do not prove real candidate behavior. | A published report could rest on fixtures only. | The manual record-run acceptance gate is a task gate; it is replaced only by a future CI job with live candidate installs. |
| TD-002 | `gold/propose.rs` drafts gold that a human must review. | Unreviewed proposals would weaken the expected answers. | Every committed task carries `reviewed_by: maintainer--ortsinn`; gold is frozen before the first record run. |
| TD-003 | Coverage merge depends on `llvm-profdata` availability. | The archive coverage gate can fail on a host without it. | `setup.sh` installs `cargo-llvm-cov` and the matching LLVM tools; the scripts fail loudly rather than copy a report. |
| TD-004 | The benchmark is project tooling outside the product runtime. | Its chapters may drift from a future product architecture. | Each affected chapter states the harness/project-tool boundary; the product chapters are updated when product crates exist. |
| TD-005 | A regex proxy arm for Serena adds report surface. | Readers may confuse the proxy with NL recall. | The proxy is rendered separately, marked `excluded_from_aggregate`, and documented in limitations. |
