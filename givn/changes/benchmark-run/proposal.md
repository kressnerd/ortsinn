# Proposal: benchmark-run

## Release note

> Runs the benchmark harness against the real candidate tools on the fixed reference repositories and publishes the measured gaps for the product-path decision.

## Use-Case Context

- Use-case ID: `benchmark-comparison` (the real-candidate measurement; the
  fixture harness and replay contract were delivered by `benchmark-gate`).
- Ideation topic: `repository-orientation`
- Seed path: `givn/ideation/repository-orientation/changes/benchmark-run.md`
  (split: this change owns the real-candidate portion).
- Confirmed Personas: `maintainer--ortsinn`; `upstream--maintainer` is
  conditional on the benchmark leading to a contribution.

## Problem / Opportunity

`benchmark-gate` delivered the Rust harness, fixture contract, replay engine,
metric scorers, report model, CLI, and fixture/e2e evidence. It deliberately
does not install or run the real candidates, does not clone the fixed
reference repositories, and does not publish a real measured report. The
proposal therefore still lacks the data that the product-path decision needs:
real typo, natural-language, explanation, and concept-recall measurements for
open-codebase-index, CKB, and Serena on the fixed inputs.

## Proposed Solution

Extend the fixture harness with the real candidate path:

- Add adapters for open-codebase-index, CKB, and Serena behind the existing
  candidate contract, including the `rmcp` stdio client and per-candidate
  setup probes.
- Add the real `benchmark/manifest.yaml`, `targets.yaml`, `fetch-repos.sh`,
  and maintainer-reviewed gold sets for the fixed reference repositories and
  the synthetic monorepo.
- Install and pin the candidate toolchains and their indexers/language
  servers in `benchmark/setup.sh`.
- Run a real record pass on the fixed inputs, store the model and result
  cassettes, replay it, and publish `report.json`/`report.md` with the
  measured gaps and the deferred product-path decision.
- Keep the fixture harness and its e2e scenarios as the deterministic
  contract test of the reporting path.
- Add a query digest to the model cassette key before reusing a recorded
  model response across gold versions; the fixture harness documents this
  omission as a limitation.

## Capability Routing

`givn spec route --change benchmark-run` reports no signal; the existing
`benchmark-comparison` capability owns the real-run scenarios.

| Proposed capability | Decision | Rationale |
|---|---|---|
| `benchmark-comparison` | `EXTEND benchmark-comparison` | The real adapters, fixed inputs, gold sets, and record run extend the same maintainer goal and report contract delivered by `benchmark-gate`. |

## Carried ledger crosswalk

`benchmark-gate` carries three decisions into this change; this proposal's
local question IDs are their new owners:

| benchmark-gate decision | benchmark-run question |
|---|---|
| D2 (reference repositories and gold sets) | D2 (gold set production) |
| D3 (competitor versions and setup normalization) | D1 (real candidate setup and availability) |
| D4 (fuzzy behavior of open-codebase-index) | D4 (single ledger entry; fuzzy behavior decided once) |

## Out of Scope

- Harness internals already delivered by `benchmark-gate`: workspace, runner,
  fixture/replay engine, report model, metric formulas, CLI, and fixture e2e.
- Product runtime behavior: the product remains MCP-only.
- The product-path build/contribute/compose choice itself. The report
  publishes the measured gaps; the maintainer records the path afterwards in
  `docs/decisions/benchmark-path.md`.

## Open Questions

### D1 — Real candidate setup and availability

- Question: which candidate versions, indexers, language servers, and
  embedding configuration are installed, and how are unavailable capabilities
  recorded?
- Options:
  1. **Install and pin every documented candidate dependency** — CKB SCIP
     indexers, Serena Python 3.13 and language servers, ocbi custom embedding
     endpoint; gain: fair measurement; cost: large setup and index time.
  2. **Record basic-mode/no-LSP deviations** — gain: less setup; cost: the
     report compares degraded candidates and must mark every affected cell.
- Disposition: carried — owner `maintainer--ortsinn`, decided in this
  change's design; the benchmark-gate design already requires the deviations
  to be visible in every affected cell.

### D2 — Gold set production

- Question: who authors and reviews the real gold sets, and at which task
  counts?
- Options:
  1. **Maintainer-reviewed gold sets at the design's minimum counts** —
     gain: checkable expected answers; cost: substantial authoring.
  2. **External benchmark sets** — gain: less authoring; cost: repositories
     and languages do not match the fixed inputs.
- Disposition: carried — owner `maintainer--ortsinn`, decided in this
  change's design. The `benchmark-gate` fixture contract and the ideation
  seed set the minimum counts.

### D3 — Real record-run gate

- Question: which real record run is required before the benchmark report can
  be published?
- Options:
  1. **At least one real candidate on one fixed reference repository, plus a
     replay probe per installed candidate** — gain: proves the real path;
     cost: install/index time.
  2. **All three candidates on all fixed inputs** — gain: complete comparison;
     cost: the full benchmark budget.
- Disposition: carried — owner `maintainer--ortsinn`, decided in this
  change's design/tasks.
