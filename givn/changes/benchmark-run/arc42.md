# arc42 Documentation Update: benchmark-run

## Impact assessment

This change extends the benchmark harness with the real-candidate path:
adapters, fixed public inputs, reviewed gold sets, per-family minimums, setup
and fetch scripts, the hybrid model endpoints, the real record run, its
provenance record, the query digest, and the real report. All twelve chapters
are affected because the harness moves from a fixture-only contract to a
measured comparison with external candidates and a manual evidence gate.

| # | Chapter | Affected? | Reason | Summary of change (if Yes) |
|---|---|---|---|---|
| 1 | 01 introduction-and-goals | Yes | The project goal now includes the real measured comparison, and the quality goals add real-setup honesty, per-power gold review, publication floors, and evidence publication. | Repointed the authoritative spec path at the permanent capability, separated the benchmark-gate fixture contract from this change's additions, and added the real-run requirements and stakeholder expectation. |
| 2 | 02 architecture-constraints | Yes | Manifest-authored candidate pins, the setup lock, the hybrid endpoints, per-power gold minimums, the 1800-second synthetic deadline, and the manual evidence gate are new constraints. | Replaced setup-time version resolution with manifest-authored pins, added the hybrid endpoint and budget constraints, the per-power gold counts, and the fixture-versus-real test boundary. |
| 3 | 03 context-and-scope | Yes | External candidates, reference repositories, the synthetic monorepo, project setup scripts, the two endpoint localities, and the provenance record enter the context. | Added the real context, separated project setup from the consumer CLI, and added the endpoint/provenance interfaces. |
| 4 | 04 solution-strategy | Yes | The strategy adds a candidate-adapter session boundary, manifest-authored pins, the setup lock, reviewed-gold power routing, the hybrid endpoints, the publication floor, the query-digest ADR, and a record-once evidence run. | Updated the real-path key decisions, added the fingerprint/provenance strategy, and reconciled the ADR wording with chapter 9. |
| 5 | 05 building-block-view | Yes | New adapter/session, gold proposer/minimums, setup validation, targets loading, provenance writing, cassette migration, and the real record path are building blocks. | Added the provenance writer, one-shot migration, targets loader, floor check, and setup-validation responsibilities. |
| 6 | 06 runtime-view | Yes | New real setup-validation, real record, provenance, floor, and real replay flows; the existing record/replay flow gains the adapter branch, hybrid endpoints, and the query-digest key. | Added the real record/replay sequence with both endpoints, the floor gate, and the provenance write, and updated the failure notes. |
| 7 | 07 deployment-view | Yes | Real candidate tools, per-candidate work copies, the setup lock, both model endpoints, the provenance record, and the evidence directories are deployment elements. | Added the endpoint localities/models, the provenance path, and the manifest-as-pin-source deployment facts. |
| 8 | 08 crosscutting-concepts | Yes | Setup deviations, setup probes, per-power reviewed gold, publication floor, query digest, hybrid endpoints, and the adapter-session contract are cross-cutting. | Added the endpoint, power, floor, and provenance concepts and corrected the unreviewed-gold semantics. |
| 9 | 09 architecture-decisions | Yes | Five design candidates were tested independently; the query-digest cassette-key decision qualifies as one ADR, and the register gains it. | Added the ADR-0001 register row, the individual ADR, the ADR directory index, and the per-candidate qualification verdicts. |
| 10 | 10 quality-requirements | Yes | Real setup, per-power gold review, query-digest replay, full manifest coverage, the publication floor, and provenance honesty need quality scenarios. | Fixed QS-018 and added QS-022 through QS-024. |
| 11 | 11 risks-and-technical-debt | Yes | External installs, gold authoring, the bounded synthetic run, cassette migration, provenance fabrication, external explanation cost, and the manual evidence gate are new risks; two old debts are addressed. | Updated R-009/R-012 through R-016 and added R-017 for the endpoint/budget consequence. |
| 12 | 12 glossary | Yes | `query digest`, `setup lock`, `provenance record`, `publication floor`, `power label`, and `targets file` are new terms; the gold-set and candidate entries changed. | Added the terms and updated the gold-set/review semantics. |

## ADR qualification

Five candidate decisions were tested independently with the complete
qualification procedure. Only the query-digest cassette-key decision passes all
five mandatory dimensions.

### Query digest in the model-cassette key — QUALIFIED

```text
ADR action: NEW_ADR
ADR target: docs/arc42/adr/ADR-0001-query-digest-model-cassette-key.md

qualification: QUALIFIED
alternatives: PASS
architectural_impact: PASS
durable_consequence: PASS
lower_level_artifact: PASS
existing_adr_check: PASS
must_be_shared: SUPPORTING
routing: NEW_ADR
canonical_artifact: null
target_adr: null
replacement_adr: null
```

Evidence: real alternatives exist (keep a query-free cassette key and document
the limitation, versus key every model entry by the normalized query digest);
architectural impact is a durable on-disk contract and migration boundary
shared by record, replay, the result cassette, and future gold versions;
durable consequence is that reversal requires migrating committed cassettes and
recomputing manifest and report fingerprints; `lower_level_artifact` passes
because a storage format is not owned by a Gherkin specification and the
change-scoped design is not the durable architecture home; the existing
register and `docs/arc42/adr/` were searched and hold no prior benchmark
cassette-format decision.

### Candidate setup and manifest pins — NOT_QUALIFIED

```text
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

Real alternatives exist (setup-time resolution versus manifest-authored pins),
but the choice is reversed by editing the manifest and re-running setup with no
migration, and its complete rationale belongs to `design.md` and the project
documentation. Routed once to `design.md`.

### Per-power gold minimums and review — NOT_QUALIFIED

```text
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

The counts and the review requirement are authoring policy plus observable
workflow behavior owned by the use case and Gherkin; changing a minimum does
not require a migration. Routed once to `design.md`.

### Record gate and per-family publication floor — NOT_QUALIFIED

```text
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

The floor is a publication rule observable through the CLI and specified by
the use case and Gherkin. It fixes no component or deployment boundary and is
reversible within this change. Routed once to `design.md`.

### Adapter/session boundary — NOT_QUALIFIED

```text
qualification: NOT_QUALIFIED
alternatives: PASS
architectural_impact: PASS
durable_consequence: FAIL
lower_level_artifact: FAIL
existing_adr_check: PASS
must_be_shared: SUPPORTING
routing: CANONICAL_ARTIFACT
canonical_artifact: design.md
target_adr: null
replacement_adr: null
```

The session fixes a component boundary (real alternatives, `FakeSession`
twin), but it is reversible within one change and its complete interface
rationale belongs to `design.md`. Routed once to `design.md`.

## Notes

- The benchmark remains project tooling outside product runtime; the product is
  MCP-only and the product-path decision remains deferred.
- The real record run is a manual evidence task; the fixture suite remains the
  CI gate and starts no real candidate.
- The real run's pins are authored in the manifest; the setup lock records
  probes and toolchain versions, and the provenance record binds the committed
  cassettes and report to the installed candidates and endpoint localities.
- All diagrams are Mermaid fenced blocks; no ASCII art is used.
- Chapter 09 stays a register plus the ADR-0001 row; the ADR body lives in
  `docs/arc42/adr/ADR-0001-query-digest-model-cassette-key.md`.

---

## Status

STATUS: DONE
