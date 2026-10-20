# 4. Solution Strategy

## Key decisions

- **Harness before the real measurement.** This change builds and proves the
  reproducible harness and report contract; `benchmark-run` adds the real
  adapters and fixed inputs and publishes the measured comparison. The
  product-path decision stays deferred.
- **Record once, replay always.** A recorded fixture run captures raw model
  responses and normalized results; replay then reproduces the report without
  a candidate install and without an external call.
- **The normalized record is the contract.** Results enter the harness as
  ordered locations (and optional explanations with citations); the real
  adapters are `benchmark-run`'s, and no capability is inferred from a README.
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
| Candidate data | Normalized result cassettes plus model cassettes | The fixture proves the contract; `benchmark-run` records real candidate results in the same shape. |
| Recording boundary | `axum` + `tokio` local OpenAI-compatible server | Records and replays model calls with no external service on replay. |
| Data formats | YAML manifest and gold metadata, JSONL cassettes, JSON/Markdown report | Human-editable inputs, streaming-friendly cassettes, one report model. |
| Isolation | Read-only fixture checkout; no candidate process; no container | Works on the target workstation and keeps replay network-free. |
| Coverage | `cargo-llvm-cov` with LLVM 23 and two Cobertura reports merged by `benchmark/tools/cobertura.py` | Instruments both the runner and the CLI child; the regular scope excludes `@e2e` and uses a distinct target directory. |

There is no qualified architecture decision record for these choices: they are
single-change, single-component implementation decisions whose complete
rationale lives in `design.md`. See chapter 9.

## Approach to quality goals

| Quality attribute | Approach |
|---|---|
| Reproducibility | Fixed fixture manifest, candidate identities, gold versions and fingerprints, cassette fingerprints, sorted maps, fixed task ordering. |
| Honesty of measurement | Closed status model, coverage denominators, separate no-match and error totals, proposed-vs-confirmed target labels. |
| Contract fidelity | Gold path/line validation at the checkout, typed missing-response failure, fixture identities for every failure family, and exact Then-step values. |
| Transparency | Report limitations; Serena NL `unavailable`; a regex proxy is labelled and excluded from the NL aggregate; the real measurement is explicitly `benchmark-run`'s. |
| Maintainability | One Rust crate and toolchain; one report model renders Markdown and the terminal; shared `env.sh` for all scripts. |
