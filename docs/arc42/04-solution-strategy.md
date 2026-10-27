# 4. Solution Strategy

## Key decisions

- **Harness contract plus the real measurement.** The fixture harness remains
  the executable contract test; this change adds the real adapters, pinned
  fixed inputs, reviewed gold sets, and the record run that publish the
  measured comparison. The product-path decision stays deferred.
- **Record once, replay always.** A recorded run captures raw model responses
  and normalized results; replay then reproduces the report without a
  candidate install and without an external call. The real run records once
  and commits its cassettes; the fixture run uses the loopback fake endpoint.
- **The normalized record is the contract.** Results enter the harness as
  ordered locations (and optional explanations with citations); the real
  adapters implement the same shape behind a session boundary and never infer
  a capability from a README.
- **Adapter session boundary.** CKB and Serena are queried through a
  `CandidateSession` backed by `rmcp` stdio MCP; open-codebase-index uses its
  documented CLI/HTTP interface. A `FakeSession` covers the mapping in tests
  and the real session is exercised by the evidence run.
- **Pin in the manifest, record probes in the lock.** Candidate and repository
  pins are authored in `benchmark/manifest.yaml` with their evidence; the
  setup script installs exactly those and exits non-zero on drift. The setup
  lock records probe results and resolved toolchain versions, never the pin
  origin.
- **Hybrid, budgeted model endpoints.** Benchmark embeddings use a loopback
  endpoint and configure candidate-internal embeddings there; only the
  at-most-80 reviewed gold explanation queries reach an external endpoint under
  a recorded operator decision. Both localities, models, and the budget appear
  in the setup lock, provenance record, and report.
- **Reviewed gold with per-power minimums.** A draft proposer writes unreviewed
  candidates; only manifest-referenced files with `reviewed_by:
  maintainer--ortsinn` and the count for the repository's power are loaded. The
  primary repository carries the full minimums, each secondary half, and
  secondary cells are labelled lower-power and excluded from cross-repository
  aggregate claims.
- **Publish only above a per-family floor.** At least one measured cell for
  each of typo, natural-language, explanation, and concept, and at least one
  measured cell for every manifest repository; otherwise the run exits 2 before
  writing either report file.
- **Targets, provenance, and reproduction identity.** The real targets live in
  `benchmark/targets.yaml` with a recorded fingerprint; the committed
  provenance record binds the cassettes and report to the manifest pins,
  setup-lock probes, endpoint localities, index outcomes, and digests.
- **Query digest in the model key.** A recorded model response is keyed by
  the normalized query digest, so a changed query is a missing response, not a
  reused answer. This is the change's one qualified decision (ADR-0001).
- **Closed status model instead of zeros.** Each cell is `measured`,
  `unavailable` with a reason, or `not_applicable` with a reason; coverage is
  attempted divided by attempted plus unavailable, excluding not-applicable.
- **Validate every fixed input.** Candidate version/commit, repository commit,
  gold and committed-cassette SHA-256 digests, every expected path and line
  range, explanation target, and acceptable-evidence range are checked before
  scoring; a mismatch fails the run with exit code 2 and writes no report.
- **Rust workspace-local harness.** One Rust crate in the project workspace,
  one toolchain, no container; isolation comes from the read-only fixture
  checkout, the caller-owned report directory, and a loopback recorder.

## Technology choices

| Concern | Choice | Rationale (see ADR) |
|---|---|---|
| Harness language | Rust, edition 2024 | Same language as the future product; one workspace and toolchain. |
| Gherkin runner | `cucumber` 0.23, `harness = false`, `filter_run_and_exit()` | Runs the same specs in Rust and fails the process on any undefined, pending, skipped, or failed step. |
| Candidate data | Normalized result cassettes plus model cassettes | The fixture proves the contract; the real adapters record real candidate results in the same shape. |
| Recording boundary | `axum` + `tokio` local OpenAI-compatible server | Records and replays model calls with no external service on replay. |
| Data formats | YAML manifest and gold metadata, JSONL cassettes, JSON/Markdown report | Human-editable inputs, streaming-friendly cassettes, one report model. |
| Isolation | Read-only fixture checkout for tests; per-candidate work copies and installed tools for the evidence run; no container | Keeps the test suite network-free and makes the real run local and repeatable. |
| Coverage | `cargo-llvm-cov` with LLVM 23 and two Cobertura reports merged by `benchmark/tools/cobertura.py` | Instruments both the runner and the CLI child; the regular scope excludes `@e2e` and uses a distinct target directory. |
| Candidate interfaces | `rmcp` stdio MCP for CKB and Serena; the documented CLI/HTTP interface for open-codebase-index | Each candidate is measured through its own interface, behind one `CandidateSession` normalization boundary. |
| Setup lock | `benchmark/setup.lock.json` with manifest-authored pins | Proves what was probed and which toolchains were used; the manifest remains the only pin origin. |
| Gold review | Draft directory, manifest `reviewed_by`, per-power minimums | Keeps unreviewed or below-minimum referenced sets out of scoring while allowing a draft proposer to reduce authoring. |
| Real evidence | Committed result/model cassettes, provenance record, and report; manual record run | Replay is free and deterministic; the provenance record makes the measurement auditable, and fixture values are never substituted. |

One decision qualifies as an architecture decision record: the query-digest
cassette-key format (ADR-0001). The remaining choices are change-scoped
implementation or workflow decisions whose complete rationale lives in the
change's `design.md`; chapter 9 records each verdict.

## Approach to quality goals

| Quality attribute | Approach |
|---|---|
| Reproducibility | Fixed manifest, candidate pins, gold versions and fingerprints, cassette fingerprints, targets fingerprint, provenance digests, sorted maps, fixed task ordering. |
| Honesty of measurement | Closed status model, coverage denominators, separate no-match and error totals, proposed-vs-confirmed target labels, and a per-family publication floor. |
| Contract fidelity | Gold path/line validation at the checkout, typed missing-response and gold-review failures, fixture identities for every failure family, and exact Then-step values. |
| Transparency | Report limitations; Serena NL `unavailable`; a regex proxy is labelled and excluded from the NL aggregate; secondary repositories are labelled lower-power; endpoint localities and the real record run are stated. |
| Maintainability | One Rust crate and toolchain; one report model renders Markdown and the terminal; shared `env.sh` for all scripts. |
