# 1. Introduction and Goals

## Requirements overview

Ortsinn is an MCP-only code-knowledge product. This change adds the
project-level benchmark harness and report contract for measuring the nearest
existing tools against the product's acceptance criteria before any product-path
decision is taken. The benchmark is not part of the product runtime.

The essential requirements of this change are:

- one `benchmark run` command verifies a fixed manifest (revision, candidate
  identity, gold version and fingerprint, cassette fingerprints, gold path and
  line ranges at the fixed checkout) and refuses a mismatch with exit code 2;
- a committed fixture — `fixture-repo` at revision `fixture-1` with three
  fixture candidate identities — proves every observable behaviour with exact
  expected values;
- the harness scores typo recall (rank one and five plus a separate no-match
  rate), natural-language recall and MRR, explanation correctness and evidence
  grounding, and concept and concept-equivalence recall with default and
  no-expansion arms;
- every metric cell is `measured`, `unavailable` with a reason, or
  `not_applicable` with a reason; coverage always carries its numerator and
  denominator, and a missing measurement is never a zero;
- recorded result and model cassettes replay without an external model call
  and without a candidate install, reproducing the measurements and ordering;
- the report names the fixed inputs, candidate setup, measured values,
  coverage, capability gaps, and measured gaps, and explicitly defers the
  build/contribute/compose decision to the maintainer and
  `docs/decisions/benchmark-path.md`.

The real candidate adapters (open-codebase-index, CKB, Serena), the public
reference repositories, the real gold sets, and the real record run are the
follow-up change `benchmark-run`'s. This change fixes and proves the contract
they must satisfy.

The authoritative behaviour is specified in the change-local
`givn/changes/benchmark-gate/specs/benchmark-comparison/benchmark-comparison.feature`,
which merges to `givn/specs/benchmark-comparison/` on archive.

## Quality goals

| Priority | Quality attribute | Motivation |
|---|---|---|
| 1 | Reproducibility | The same manifest, gold set, fixture identities, and cassettes must produce the same values and ordering; a comparison that cannot be rerun is not evidence. |
| 2 | Honesty of measurement | A missing capability, unstartable candidate, or index timeout is `unavailable` with a reason, never a zero; coverage is reported with its denominator. |
| 3 | Contract fidelity | The fixture record shape is the contract the real adapters must produce; gold paths and line ranges are validated at the fixed checkout, and every failure family has a scenario. |
| 4 | Transparency | The report labels proposed targets as proposed, separates rank from no-match, and states its limitations (rubric surface forms, the fixture CKB Rust Tier 3 note, Serena NL unavailability). |
| 5 | Maintainability | The harness is Rust in the project's workspace and reuses one toolchain; recording/replay keeps tests free and deterministic. |

## Stakeholders

| Role | Expectation |
|---|---|
| Ortsinn maintainer (`maintainer--ortsinn`) | One reproducible command, an auditable report contract, and a defensible basis for the product-path decision once `benchmark-run` publishes the real measurement. |
| Upstream project maintainer (`upstream--maintainer`, conditional) | If the path is a contribution, the measured gaps that concern the upstream project, with the exact revisions and evidence. |
| Future product user (beneficiary) | That no duplicate product is built, and that a contribution or composition is chosen on measured gaps rather than README claims. |
