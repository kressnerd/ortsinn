# 10. Quality Requirements

## Quality tree

```mermaid
flowchart LR
    Q[Benchmark quality] --> R[Reproducibility]
    Q --> H[Honesty]
    Q --> F[Fairness]
    Q --> T[Transparency]
    R --> R1[Deterministic replay]
    R --> R2[Fixed inputs and fingerprints]
    H --> H1[Unavailable is not zero]
    H --> H2[Coverage with denominators]
    F --> F1[Pinned setup and recorded deviations]
    F --> F2[Documented interfaces only]
    T --> T1[Proposed targets labelled]
    T --> T2[Limitations stated]
```

## Quality scenarios

| ID | Quality attribute | Scenario | Metric / Acceptance criterion |
|---|---|---|---|
| QS-001 | Reproducibility | The maintainer reruns the same manifest, gold, pins, and cassettes in replay. | Identical metric values, coverage, ordering, and fingerprints; no external request; no candidate install. |
| QS-002 | Reproducibility | A repository or gold fingerprint does not match the manifest. | Exit 2 naming the mismatch; no `report.json`/`report.md` published. |
| QS-003 | Honesty | A candidate has no explanation operation. | Explanation cells `unavailable` with `no_explanation_operation`; coverage 0/4 for that candidate; aggregate remains measured from other candidates; capability gap listed. |
| QS-004 | Honesty | A metric has no measured candidate at all. | `to_best` and `to_target` have status `unavailable`, reason `no_measured_candidate`, and no numeric value. |
| QS-005 | Honesty | A candidate returns no result for some typo tasks. | No-match rate reported separately from recall, with its numerator and denominator. |
| QS-006 | Fairness | CKB's Rust support is Tier 3 and Serena has no NL search. | Tier and `no_natural_language_search` recorded on the cells; no silent quality difference and no zero. |
| QS-007 | Transparency | The report shows a gap to AC-01's 0.90. | Gap labelled "gap to proposed target" with `target_status: proposed`; never "confirmed". |
| QS-008 | Determinism | The synthetic monorepo is regenerated from the fixed seed and generator version. | Same content fingerprint and same gold locations; ten million code lines across three languages. |
| QS-009 | Fairness | A candidate exceeds its index deadline. | `index_timeout` with elapsed time; every metric for that candidate/repository unavailable and never zero. |
| QS-010 | Transparency | A query errors during a metric run. | Error counted as a miss, recall denominator unchanged, error total reported separately. |
| QS-011 | Usability | The terminal summary is rendered at 80 and 160 columns. | No metric label/value pair wraps or splits across lines at either width. |
| QS-012 | Testability | A step is undefined or skipped. | The `harness = false` cucumber runner exits non-zero; the run cannot report GREEN with nothing executed. |
| QS-013 | Evidence | A real candidate adapter returns empty results while fixtures pass. | The manual record-run gate fails the task: no committed record-run evidence and replay probe, no accepted report. |
| QS-014 | Code coverage | Both the cucumber runner and CLI child run under instrumentation. | Both Cobertura reports exist from distinct profiles; the named production probes are non-zero. |
