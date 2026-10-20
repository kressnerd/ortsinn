# 1. Introduction and Goals

## Requirements overview

Ortsinn is an MCP-only code-knowledge product. This change adds a project-level
benchmark gate that measures the nearest existing tools against the product's
acceptance criteria before any product-path decision is taken. The benchmark is
not part of the product runtime.

The essential requirements of the gate are:

- one `benchmark run` command runs on fixed inputs: three pinned public
  reference repositories (TypeScript, Python, Rust) and one deterministic
  10-million-line synthetic monorepo;
- three pinned candidates — open-codebase-index, CKB, and Serena — are measured
  through their own documented interfaces for typo recall, natural-language
  retrieval, explanation correctness, and concept recall, including the
  no-expansion arm where a candidate supports it;
- the report names every candidate and revision, the value, numerator,
  denominator, and coverage of every metric, the capability gaps, and the
  measured gaps against targets;
- a recorded run replays without an external model call and without candidate
  installs, and reproduces the measurements and ordering;
- the report publishes the gaps and explicitly defers the
  build/contribute/compose decision to the maintainer and
  `docs/decisions/benchmark-path.md`.

The authoritative behaviour is specified in
`givn/specs/benchmark-comparison/benchmark-comparison.feature` (merged on
archive).

## Quality goals

| Priority | Quality attribute | Motivation |
|---|---|---|
| 1 | Reproducibility | The same manifest, gold set, candidate pins, and cassettes must produce the same values and ordering; a comparison that cannot be rerun is not evidence. |
| 2 | Honesty of measurement | A missing capability, unstartable candidate, or index timeout is `unavailable` with a reason, never a zero; coverage is reported with its denominator. |
| 3 | Fairness and auditability | Candidate setup is normalized and pinned, deviations are recorded, and every query goes through the candidate's documented interface. |
| 4 | Transparency | The report labels proposed targets as proposed, separates rank from no-match, and states its limitations (rubric surface forms, CKB Rust Tier 3, Serena NL unavailability). |
| 5 | Maintainability | The harness is Rust in the project's workspace and reuses one toolchain; recording/replay keeps tests free and deterministic. |

## Stakeholders

| Role | Expectation |
|---|---|
| Ortsinn maintainer (`maintainer--ortsinn`) | One reproducible command, an auditable report, and a defensible basis for the product-path decision. |
| Upstream project maintainer (`upstream--maintainer`, conditional) | If the path is a contribution, the measured gaps that concern the upstream project, with the exact revisions and evidence. |
| Future product user (beneficiary) | That no duplicate product is built, and that a contribution or composition is chosen on measured gaps rather than README claims. |
