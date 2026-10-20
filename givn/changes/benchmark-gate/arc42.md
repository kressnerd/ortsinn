# arc42 Documentation Update: benchmark-gate

## Impact assessment

| # | Chapter | Affected? | Reason | Summary of change (if Yes) |
|---|---|---|---|---|
| 1 | 01 introduction-and-goals | Yes | The change adds a project goal (a reproducible benchmark gate before the product-path decision) and quality goals (reproducibility, honesty, fairness, transparency) for the harness. | Added the benchmark requirements overview, five quality goals, and the maintainer/upstream/future-user stakeholders. |
| 2 | 02 architecture-constraints | Yes | It fixes the gnu toolchain and zlib, no container, workspace-local tool prefixes, pinned candidate/repository revisions, a loopback model boundary, the `harness = false` runner, and the proposed-only AC-01 rule. | Added technical, organisational, and convention constraints for the harness. |
| 3 | 03 context-and-scope | Yes | It introduces a new user-facing CLI surface and external candidates/model endpoint, and records that the harness is outside the product runtime. | Added business and technical context Mermaid diagrams, the interface table, and the explicit scope boundary. |
| 4 | 04 solution-strategy | Yes | It establishes benchmark-first, record-once/replay-always, behavioral measurement through documented interfaces, the closed status model, and a workspace-local Rust harness. | Added key decisions, a technology-choice table, and the quality-goal approach. |
| 5 | 05 building-block-view | Yes | It adds the `benchmark` crate with CLI, runner, manifest, adapters, MCP transport, recorder, metrics, report, gold, and generator. | Added a level-1 component diagram and level-2 views for runner, adapters, recording, report, and gold/generator. |
| 6 | 06 runtime-view | Yes | It adds replay, record, index-timeout/unavailable, and fixed-input/missing-cassette flows. | Added sequence and flow diagrams with steps and error semantics for the four flows. |
| 7 | 07 deployment-view | Yes | It adds a local toolchain, Nix zlib, tool prefixes, worktrees, committed cassettes, and a loopback endpoint. | Added the deployment diagram and level-2 node, prefix, and evidence details. |
| 8 | 08 crosscutting-concepts | Yes | It introduces the recording/replay boundary, canonical keys and fingerprints, the closed status model, metric/task coverage, determinism, exit codes, and target labeling. | Added the crosscutting concepts with the coverage/code-coverage distinction and gap labeling. |
| 9 | 09 architecture-decisions | Yes | A structured ADR verdict and the existing-ADR/duplicate check must be recorded even though no ADR qualifies. | Added `ADR action: NONE`, routing `CANONICAL_ARTIFACT` to `design.md`, the five-gate verdict, and the empty-register check. |
| 10 | 10 quality-requirements | Yes | It adds measurable quality scenarios for replay determinism, unavailable-is-not-zero, coverage, width, runner strictness, record evidence, and coverage instrumentation. | Added a quality tree and QS-001 through QS-014. |
| 11 | 11 risks-and-technical-debt | Yes | It adds the top implementation risk (real adapters/record path untested), the broken musl toolchain, index time, candidate drift, setup degradation, and the proposed-target risk. | Added risks R-001 through R-010 and technical debt TD-001 through TD-005. |
| 12 | 12 glossary | Yes | It introduces benchmark, candidate, cassette, replay, capability gap, measured gap, no-expansion arm, fixed input, gold set, metric/task coverage, recording boundary, and adds the confirmed concept-equivalence term. | Added the terms with contrast/anti-terms and the metric/task-versus-code-coverage distinction. |

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
register at `docs/arc42/09-architecture-decisions.md` and
`docs/arc42/adr/README.md` were searched; no existing ADR covers the harness,
so there is no duplicate, amendment, or supersession.

## Notes

- The benchmark is a project-level tool, not product runtime; chapters document
  it because its report gates the next product decision.
- All diagrams are Mermaid fenced code blocks; no ASCII art is used.
- Chapter 09 stays a register; no MADR body was created.

---

## Status

STATUS: DONE
