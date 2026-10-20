# 4. Solution Strategy

## Key decisions

- **Benchmark before the product path.** The project measures the nearest tools
  on fixed inputs before choosing build, contribute, or compose; the report
  publishes the gaps and the path decision is deferred.
- **Record once, replay always.** A recorded run captures raw model responses
  and normalized candidate results; replay then reproduces the report without a
  candidate install and without an external call.
- **Behavioral measurement through documented interfaces.** Every candidate is
  driven through its own CLI/MCP surface, probed for capabilities at the pinned
  version, and never credited with an operation it does not expose.
- **Closed status model instead of zeros.** Each cell is `measured`,
  `unavailable` with a reason, or `not_applicable` with a reason; coverage is
  attempted divided by attempted plus unavailable, excluding not-applicable.
- **Rust workspace-local harness.** One Rust crate in the project workspace,
  one toolchain, no container; isolation comes from per-candidate worktrees,
  tool prefixes, and a loopback recorder.

## Technology choices

| Concern | Choice | Rationale (see ADR) |
|---|---|---|
| Harness language | Rust, edition 2024 | Same language as the future product; one workspace and toolchain. |
| Gherkin runner | `cucumber` 0.23, `harness = false`, `filter_run_and_exit()` | Runs the same specs in Rust and fails the process on any undefined, pending, skipped, or failed step. |
| Candidate transport | `rmcp` 3.5.1 with `client` and `transport-child-process`; CLI and stdio JSON-RPC fallback | Uses each candidate's documented surface; a failed handshake is an unavailable candidate, not a crash. |
| Recording boundary | `axum` + `tokio` local OpenAI-compatible server | Records and replays model calls with no external service on replay. |
| Data formats | YAML manifest and gold metadata, JSONL cassettes, JSON/Markdown report | Human-editable inputs, streaming-friendly cassettes, one report model. |
| Isolation | Workspace-local worktrees and tool prefixes; no container | Works on the target workstation and keeps replay network-free. |
| Coverage | `cargo llvm-cov` with `llvm-profdata` merge, Cobertura per scope | Instrument both the runner and the CLI child; the non-E2E scope excludes `@e2e`. |

There is no qualified architecture decision record for these choices: they are
single-change, single-component implementation decisions whose complete
rationale lives in `design.md`. See chapter 9.

## Approach to quality goals

| Quality attribute | Approach |
|---|---|
| Reproducibility | Fixed manifest, candidate pins, gold versions and fingerprints, canonical cassette keys, sorted maps, fixed task ordering, timestamps excluded from measurement identity. |
| Honesty of measurement | Closed status model, coverage denominators, separate no-match and error totals, proposed-vs-confirmed target labels. |
| Fairness and auditability | Pinned indexers and language servers, recorded setup deviations, `ckb doctor` output, `.git` present in candidate copies, CKB Rust Tier 3 recorded. |
| Transparency | Report limitations section; Serena NL `unavailable`; a regex proxy is labelled and excluded from the NL aggregate. |
| Maintainability | One Rust crate and toolchain; one report model renders Markdown and the terminal; shared `env.sh` for all scripts. |
