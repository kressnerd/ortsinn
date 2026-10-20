# arc42 Documentation Update: benchmark-gate

## Impact assessment

The change is split: this change delivers the fixture-backed harness and report
contract. `benchmark-run` owns the real adapters, public fixed inputs, real
gold sets, candidate installs, and the real record run. The chapters describe
this change's harness and name the split where the real measurement belongs.

| # | Chapter | Affected? | Reason | Summary of change (if Yes) |
|---|---|---|---|---|
| 1 | 01 introduction-and-goals | Yes | The change adds a project goal (a reproducible harness and report contract before the product-path decision) and quality goals (reproducibility, honesty, contract fidelity, transparency, maintainability) for the harness. | Added the harness requirements overview, five quality goals, and the maintainer/upstream/future-user stakeholders. |
| 2 | 02 architecture-constraints | Yes | It fixes the gnu toolchain, zlib, LLVM 23, the fixture pins, the checkout path, the loopback boundary, the `harness = false` runner, two separate coverage scopes, and the operator split. | Added technical, organisational, and convention constraints for the harness and the split. |
| 3 | 03 context-and-scope | Yes | It introduces the CLI surface, the fixture checkout, cassettes, and the fake endpoint, and records that the harness is outside the product runtime and that real candidates are `benchmark-run`'s. | Added business and technical context Mermaid diagrams, the interface table, and the scope boundary. |
| 4 | 04 solution-strategy | Yes | It establishes harness-first, record-once/replay-always, the normalized record contract, gold validation, the closed status model, and a workspace-local Rust harness. | Added key decisions, a technology-choice table, and the quality-goal approach. |
| 5 | 05 building-block-view | Yes | It adds the `benchmark` crate with CLI, runner, manifest, gold, cassettes, metrics, report, generator, and the Cobertura tool. It does not add candidate adapters or MCP transport. | Added a level-1 component diagram and level-2 views for runner, recording, metrics, report, and gold/generator. |
| 6 | 06 runtime-view | Yes | It adds replay, record, index-timeout/unavailable, and fixed-input/missing-response flows over recorded fixture data. | Added sequence and flow diagrams with steps and error semantics for the four flows. |
| 7 | 07 deployment-view | Yes | It adds the fixture checkout, separate coverage targets, LLVM 23, the loopback endpoint, and committed transcripts. | Added the deployment diagram and level-2 fixture, recording, and coverage details. |
| 8 | 08 crosscutting-concepts | Yes | It introduces the recording/replay boundary, canonical keys and fingerprints, gold validation, the closed status model, metric/task versus code coverage, exit codes, and the fixture-candidate distinction. | Added the crosscutting concepts with the coverage/code-coverage distinction and gap labeling. |
| 9 | 09 architecture-decisions | Yes | A structured ADR verdict and the existing-ADR/duplicate check must be recorded even though no ADR qualifies. | Added `ADR action: NONE`, routing `CANONICAL_ARTIFACT` to `design.md`, the five-gate verdict, and the empty-register check. |
| 10 | 10 quality-requirements | Yes | It adds measurable quality scenarios for replay determinism, unavailable-is-not-zero, gold validation, coverage, width, runner strictness, contract fidelity, and coverage instrumentation. | Added a quality tree and QS-001 through QS-015. |
| 11 | 11 risks-and-technical-debt | Yes | It adds the contract-drift risk, the split/unpublished-comparison risk, the broken musl toolchain, the LLVM profile mismatch, candidate drift, setup degradation, and the proposed-target risk. | Added risks R-001 through R-011 and technical debt TD-001 through TD-005. |
| 12 | 12 glossary | Yes | It adds benchmark, candidate, capability gap, cassette, code coverage, concept equivalence, fixed input, fixture candidate, gold set, index deadline, measured gap, metric/task coverage, no-expansion arm, path decision, query error, recording boundary, replay, target, and the candidate/fixture-candidate distinction. | Added the terms with contrast/anti-terms and the metric/task-versus-code-coverage distinction. |

## ADR qualification

```text
ADR action: NONE
ADR target: none

qualification: NOT_QUALIFIED
alternatives: PASS
architectural_impact: FAIL
durable_consequence: FAIL
lower_level_artifact: FAIL
existing_adr_check: PASS
must_be_shared: SUPPORTING
routing: CANONICAL_ARTIFACT
canonical_artifact: design.md
target_adr: null
replacement_adr: null
```

The harness language, workspace membership, recording/replay boundary, and
no-container isolation have real alternatives, but they are single-component,
single-change decisions whose complete rationale is owned by `design.md`. The
register at `docs/arc42/09-architecture-decisions.md` was searched;
`docs/arc42/adr/` contains only the ADR template. No existing ADR covers the
harness, so there is no duplicate, amendment, or supersession.

## Notes

- The benchmark is a project-level tool, not product runtime; chapters document
  it because its report contract gates the next product decision.
- The real candidate adapters, public fixed inputs, real gold sets, and real
  record run are `benchmark-run`'s and are marked as its interface where they
  appear in the constraints and scope chapters.
- All diagrams are Mermaid fenced code blocks; no ASCII art is used.
- Chapter 09 stays a register; no MADR body was created.

---

## Status

STATUS: DONE
