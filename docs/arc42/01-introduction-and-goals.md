# 1. Introduction and Goals

## Requirements overview

Ortsinn is an MCP-only code-knowledge product. The project-level benchmark
harness and report contract were delivered by `benchmark-gate`; the
real-candidate path extends them and publishes the measured comparison that the
product-path decision consumes. The benchmark is not part of the product
runtime.

The essential requirements of the benchmark are:

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
  `docs/decisions/benchmark-path.md`;
- a real record run covers every manifest candidate and repository, records
  setup deviations and unavailable candidates, attempts the synthetic monorepo
  within its 30-minute deadline, and publishes the measured comparison with its
  committed cassettes, provenance record, and report only when the per-family
  measurement floor is met.

The real candidate adapters (open-codebase-index, CKB, Serena), the public
reference repositories, the reviewed gold sets, the hybrid model endpoints, and
the real record run extend the fixture harness. The fixture harness remains the
executable contract test; the real run publishes the measured comparison.

The authoritative behaviour is specified in
`givn/specs/benchmark-comparison/benchmark-comparison.feature`; the
`benchmark-run` delta that extends it lives under
`givn/changes/benchmark-run/specs/` until archive.

## Quality goals

| Priority | Quality attribute | Motivation |
|---|---|---|
| 1 | Reproducibility | The same manifest, gold set, candidate identities, and cassettes must produce the same values and ordering; candidate pins are authored in the manifest and the provenance record binds the committed evidence to them. |
| 2 | Honesty of measurement | A missing capability, unstartable candidate, or index timeout is `unavailable` with a reason, never a zero; coverage is reported with its denominator. |
| 3 | Contract fidelity | The fixture record shape is the contract the real adapters must produce; gold paths and line ranges are validated at the fixed checkout, and every failure family has a scenario. |
| 4 | Transparency | The report labels proposed targets as proposed, separates rank from no-match, labels secondary repositories lower-power, and states its limitations (rubric surface forms, the fixture CKB Rust Tier 3 note, Serena NL unavailability, endpoint localities). |
| 5 | Maintainability | The harness is Rust in the project's workspace and reuses one toolchain; recording/replay keeps tests free and deterministic. |
| 6 | Evidence completeness | A real report publishes only when at least one measured cell exists for every metric family and every manifest repository; it names setup deviations, unavailable reasons, endpoints, and provenance digests, and commits cassettes plus the report. |

## Stakeholders

| Role | Expectation |
|---|---|
| Ortsinn maintainer (`maintainer--ortsinn`) | One reproducible command, an auditable report contract, and a defensible basis for the product-path decision including the real measurement the benchmark publishes. |
| Upstream project maintainer (`upstream--maintainer`, conditional) | If the path is a contribution, the measured gaps that concern the upstream project, with the exact revisions and evidence. |
| Future product user (beneficiary) | That no duplicate product is built, and that a contribution or composition is chosen on measured gaps rather than README claims. |
