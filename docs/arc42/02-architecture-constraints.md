# 2. Architecture Constraints

## Technical constraints

| Constraint | Motivation |
|---|---|
| Rust gnu toolchain `stable-x86_64-unknown-linux-gnu` (cargo/rustc 1.99.1) | The workstation's default musl toolchain is not runnable; all project scripts source `env.sh`, which sets `RUSTUP_TOOLCHAIN`. |
| `libz.so.1` on the loader path | The gnu `rustc` needs zlib; `env.sh` uses a system zlib or the Nix `zlib` path from `nix build --no-link --print-out-paths nixpkgs#zlib`. |
| No container runtime | Isolation is workspace-local: per-candidate worktrees, tool prefixes under `benchmark/.tools/`, and a loopback recorder. |
| Pinned candidate versions and commits | A comparison is only valid if the candidate identity is fixed; the manifest records version, commit, and adapter version. |
| Pinned reference revisions | Every query names a repository at a fixed commit; `git rev-parse HEAD` is verified before scoring. |
| Loopback-only model boundary | Replay serves recorded responses and makes no external request; record mode forwards to a configured local OpenAI-compatible endpoint. |
| One `benchmark run` command | The consumer surface is the proposed benchmark command; there is no separate `verify` or `report` subcommand. |
| `cucumber` 0.23 runner with `harness = false` and `filter_run_and_exit()` | Libtest would consume `--name`/`--tags` and `run()` would not fail the process; the strict runner is a hard test-infrastructure constraint. |
| Package versions pinned in `Cargo.lock` and the manifest | Reproducible builds; `rmcp` uses the `client` and `transport-child-process` features explicitly. |

## Organisational constraints

| Constraint | Motivation |
|---|---|
| The maintainer owns the gold sets and reviews every public task | Gold is frozen before the first record run; tuning expected answers after seeing results is prohibited. |
| The benchmark pre-registers no build threshold | The product-path decision is made after the report, in `docs/decisions/benchmark-path.md`, and the report contains no chosen path. |
| AC-01's 0.90 is recorded as `proposed`, never `confirmed` | The acceptance thresholds in the proposal attachments are not confirmed; presenting 0.90 as accepted would overstate the evidence. |
| Target numbers for AC-03, AC-12, AC-21, and AC-22 stay `unstated` | No confirmed numeric target exists in the repository; inventing one would be fabricated acceptance. |
| A real record run is a manual acceptance gate | Fixture-only evidence cannot prove three heterogeneous real adapters work. |

## Conventions

| Convention | Motivation |
|---|---|
| Gherkin scenarios assert concrete fixture values, not section presence | A report that prints headings with arbitrary numbers must not pass. |
| Every metric reports value, numerator, denominator, and coverage; missing cells are `unavailable` or `not_applicable` | Prevents a small or missing sample from reading as a strong result. |
| "Metric/task coverage" always means attempted tasks; "code coverage" always means instrumented lines | The two concepts must not collide in the report or the coverage scripts. |
| Diagrams are Mermaid fenced blocks | ASCII-art diagrams are not permitted in the architecture documentation. |
| The report states the exact replay command and every fingerprint | The result must be independently rerunnable. |
