# 11. Risks and Technical Debt

## Risks

| ID | Risk | Probability | Impact | Mitigation |
|---|---|---|---|---|
| R-001 | The fixture contract drifts from what a real candidate record can express; a real adapter needs a shape or failure the scorer does not handle while the fixture suite passes. | Medium | High | The normalized record is the reviewed contract; every failure family (missing response, mismatch, timeout, unstartable, query error, all-unavailable gap) has a scenario; `benchmark-run` must record real results in the same shape before the path decision. |
| R-002 | The split leaves the real comparison unpublished until `benchmark-run` completes. | High | High | The split is the recorded operator decision; the review carries the `DEFERRAL` line and `benchmark-run` owns the real record gate. The deferral is never presented as a real measurement. |
| R-003 | The default musl Rust toolchain is broken and gnu `rustc` needs zlib. | High | Medium | `env.sh` pins `stable-x86_64-unknown-linux-gnu` and resolves zlib via system or Nix; every script sources it. |
| R-004 | The Nix LLVM 21.1.8 package cannot merge rustc's raw profile version 11. | High | Medium | `env.sh` resolves `nixpkgs#llvmPackages_23.llvm` from rustc's LLVM major and exports `LLVM_COV`/`LLVM_PROFDATA`; the coverage scripts fail loudly when the tools are absent. |
| R-005 | Candidate interfaces drift between pinned versions. | Medium | Medium | `benchmark-run` pins versions and records normalized results in cassettes, so a report stays reproducible even when an upstream tool changes. |
| R-006 | A proposed AC-01 target is read as an accepted threshold. | Medium | Medium | `status: proposed` in the manifest targets, "gap to proposed target" labeling, no path choice in the report. |
| R-007 | CKB's Rust Tier 3 support makes a Rust comparison look like a quality difference. | Medium | Medium | The fixture records the tier as a capability/coverage note; `benchmark-run` records it for the real candidate. |
| R-008 | Serena's regex pattern search is mistaken for natural-language retrieval. | Medium | Medium | NL cells `unavailable` with `no_natural_language_search`; any regex proxy is labelled and excluded from the NL aggregate. |
| R-009 | The report looks complete while no measured value exists. | Low | High | Closed status model, coverage denominators, and specs that assert concrete values; `benchmark-run` owns the real measurement. |
| R-010 | Recorded cassettes are missing, stale, or accidentally overwritten. | Low | High | Cassette fingerprints in the report, typed exit 2 on a miss, committed fixture cassettes, and a temporary output directory in the record E2E. |
| R-011 | The coverage merge accepts a copied report and reports one scope twice. | Low | Medium | `merge-coverages.sh` refuses byte-identical inputs and probes a file reachable only from each scope. |

## Technical debt

| ID | Item | Impact | Plan to address |
|---|---|---|---|
| TD-001 | Fixture and replay evidence do not prove real candidate behavior. | A report could rest on fixtures only. | Owned by `benchmark-run`: its real record-run gate and replay probe are the evidence before the path decision. |
| TD-002 | The real gold sets and public fixed inputs do not exist yet. | No real comparison is possible. | `benchmark-run` authors the reviewed gold sets and adds `benchmark/manifest.yaml`, `targets.yaml`, and `fetch-repos.sh`. |
| TD-003 | Coverage merge depends on the LLVM 23 tools and the Python helper. | The archive coverage gate can fail on a host without them. | `env.sh` resolves both; the scripts fail loudly rather than copy a report. |
| TD-004 | The benchmark is project tooling outside the product runtime. | Its chapters may drift from a future product architecture. | Each affected chapter states the harness/project-tool boundary; the product chapters are updated when product crates exist. |
| TD-005 | A regex proxy arm for Serena adds report surface. | Readers may confuse the proxy with NL recall. | The proxy is rendered separately, marked `excluded_from_aggregate`, and documented in limitations. |
