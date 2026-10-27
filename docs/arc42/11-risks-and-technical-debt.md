# 11. Risks and Technical Debt

## Risks

| ID | Risk | Probability | Impact | Mitigation |
|---|---|---|---|---|
| R-001 | The real adapter needs a shape or failure the scorer does not handle while the fixture suite passes. | Medium | High | The normalized record is the reviewed contract; the `CandidateSession` boundary plus `FakeSession` tests cover mapping, and every failure family (setup probe, deviation, missing response, mismatch, timeout, query error, all-unavailable gap) has a scenario. |
| R-002 | The real comparison stays unpublished because the external repositories, candidate installs, or model endpoints are unavailable. | High | High | The real record run is an explicit evidence task and a gate for the measured comparison; a missing dependency is recorded or the task is blocked. Fixture values are never substituted for measured values. |
| R-003 | The default musl Rust toolchain is broken and gnu `rustc` needs zlib. | High | Medium | `env.sh` pins `stable-x86_64-unknown-linux-gnu` and resolves zlib via system or Nix; every script sources it. |
| R-004 | The Nix LLVM 21.1.8 package cannot merge rustc's raw profile version 11. | High | Medium | `env.sh` resolves `nixpkgs#llvmPackages_23.llvm` from rustc's LLVM major and exports `LLVM_COV`/`LLVM_PROFDATA`; the coverage scripts fail loudly when the tools are absent. |
| R-005 | Candidate interfaces drift between pinned versions. | Medium | Medium | The setup lock and manifest pin the resolved versions and commits; the setup probe records declared operations; normalized cassettes keep the report reproducible even when an upstream tool changes. |
| R-006 | A proposed AC-01 target is read as an accepted threshold. | Medium | Medium | `status: proposed` in the manifest targets, "gap to proposed target" labeling, no path choice in the report. |
| R-007 | CKB's Rust Tier 3 support makes a Rust comparison look like a quality difference. | Medium | Medium | The setup lock records the real language-support tier; the report names it on every affected cell as a capability/coverage note. |
| R-008 | Serena's regex pattern search is mistaken for natural-language retrieval. | Medium | Medium | NL cells `unavailable` with `no_natural_language_search`; any regex proxy is labelled and excluded from the NL aggregate. |
| R-009 | The report looks complete while no measured value exists. | Low | High | Closed status model, coverage denominators, and a per-family publication floor that exits 2 before writing either report file; every manifest cell is measured, unavailable, or not applicable. |
| R-010 | Recorded cassettes are missing, stale, or accidentally overwritten. | Low | High | Cassette fingerprints in the report, typed exit 2 on a miss, committed fixture cassettes, and a temporary output directory in the record E2E. |
| R-011 | The coverage merge accepts a copied report and reports one scope twice. | Low | Medium | `merge-coverages.sh` refuses byte-identical inputs and probes a file reachable only from each scope. |
| R-012 | The external repositories, candidate installs, or model endpoints are unavailable when the evidence run is attempted. | High | High | Candidate pins are authored in the manifest with evidence; the setup lock records probes and toolchains; a missing dependency is a setup deviation or unavailable cell. The evidence task is blocked rather than filled with fixture values. |
| R-013 | Gold authoring at the per-power minimums is too large or contains an unreviewed task. | Medium | High | The primary repository needs 170 tasks and each secondary 85; the draft proposer reduces authoring, and the loader rejects an unreviewed referenced set or a below-minimum count before scoring. |
| R-014 | The 10M-line synthetic run exceeds its deadline or disk budget. | Medium | Medium | The generator runs under `benchmark/work/`; the real attempt is bounded by the 1,800-second deadline; a breach marks the cell `index_timeout` with elapsed time and the report remains complete. |
| R-015 | The query-digest key migration leaves the committed fixture cassettes stale or report fingerprints inconsistent. | Low | High | ADR-0001's one-shot result-cassette migration and model-cassette regeneration update both cassettes and the manifest fingerprint; replay fails loudly on a missing key rather than reusing a query-free entry, and the `@givn.modified` scenario asserts the digest shape. |
| R-016 | The real record run is manual and its committed evidence could be fabricated. | Medium | High | The committed provenance record binds the cassettes and report to the manifest pins, setup-lock probes, endpoint localities, index outcomes, and digests; scenarios 7, 11, and 15 cross-check it, so a hand-authored artifact fails. |
| R-017 | The external explanation endpoint costs money or leaks code questions. | Medium | Medium | Only the at-most-80 reviewed gold explanation queries leave the machine under a recorded operator decision; candidate-internal embeddings stay on loopback and are bounded by the index deadline; the report states both localities, models, and the budget. |

## Technical debt

| ID | Item | Impact | Plan to address |
|---|---|---|---|
| TD-001 | Fixture and replay evidence do not by themselves prove real candidate behavior. | A report could rest on fixtures only. | The real record run and replay probe are the evidence before the path decision; the fixture suite remains the contract test. |
| TD-002 | Real gold sets and public fixed inputs need maintenance as upstream revisions change. | A stale pin makes the comparison less relevant. | The manifest, setup lock, fetch/setup scripts, and draft proposer are shipped; a new comparison is a new pin and record run. |
| TD-003 | Coverage merge depends on the LLVM 23 tools and the Python helper. | The archive coverage gate can fail on a host without them. | `env.sh` resolves both; the scripts fail loudly rather than copy a report. |
| TD-004 | The benchmark is project tooling outside the product runtime. | Its chapters may drift from a future product architecture. | Each affected chapter states the harness/project-tool boundary; the product chapters are updated when product crates exist. |
| TD-005 | A regex proxy arm for Serena adds report surface. | Readers may confuse the proxy with NL recall. | The proxy is rendered separately, marked `excluded_from_aggregate`, and documented in limitations. |
