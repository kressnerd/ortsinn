# Change seed: benchmark-run

Status: proposed by benchmark-run

- Use-case IDs: `benchmark-comparison` (real-candidate measurement)
- Topic: `repository-orientation`
- Personas: `maintainer--ortsinn`; `upstream--maintainer` is conditional on
  the path outcome
- Suggested start: after `benchmark-gate`; depends on the fixture harness,
  replay contract, and report model
- Handoff: ready

## Accepted decisions

### Split from benchmark-gate

- Context: the review found that `benchmark-gate` implemented only the
  fixture harness; the real candidate adapters, fixed inputs, gold sets, and
  record run were missing.
- Options:
  1. **Implement the real path inside `benchmark-gate`** — one change; long
     setup and record run.
  2. **Split the real path into `benchmark-run`** — archive the harness now;
     the real measurement follows.
- Chosen: **Split**. Rejected: one change because the harness is a complete,
  testable deliverable on its own and the real run needs large external
  inputs and candidate setup.

## Open questions

### Real candidate setup

Which candidate versions, indexers, language servers, embedding
configuration, and fixed repository commits are used? Decided in
`benchmark-run`'s design.

### Gold sets and record-run gate

Who authors the real gold sets, and which real record run is required before
publishing? Decided in `benchmark-run`'s design/tasks.
