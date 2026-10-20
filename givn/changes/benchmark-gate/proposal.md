# Proposal: benchmark-gate

## Release note

> Adds the reproducible benchmark harness and report contract for the
> code-knowledge comparison: a Rust CLI that verifies fixed inputs, replays
> recorded candidate results, scores typo, natural-language, explanation, and
> concept recall with coverage and measured gaps, and publishes
> `report.json`/`report.md`. The harness is proven on a committed fixture;
> the real candidate measurement on the fixed reference repositories is
> delivered by the follow-up change `benchmark-run`.

## Use-Case Context

- Use-case ID: `benchmark-comparison` — a new project-gate use case. It is not
  part of the product's runtime cut; the seed records no product use-case ID.
  This change delivers the harness and report contract; `benchmark-run`
  extends the same capability with the real-candidate measurement.
- Ideation topic: `repository-orientation`
- Seed paths: `givn/ideation/repository-orientation/changes/benchmark-gate.md`
  and the split seed
  `givn/ideation/repository-orientation/changes/benchmark-run.md`
- Confirmed Personas: `maintainer--ortsinn`; `upstream--maintainer` is
  conditional on the benchmark leading to a contribution and is not exercised
  by this change.

## Problem / Opportunity

The market survey found a dense field of code-knowledge tools, but none of the
nearest projects has been measured against Ortsinn's acceptance criteria. The
maintainer cannot tell whether the combination gap that would justify a new
product actually exists: open-codebase-index already combines a Rust core,
Tree-sitter, SQLite, usearch, BM25, a watcher, and a call graph; CKB
orchestrates SCIP, LSP, and Git; Serena is LSP-based. Their README claims do
not establish typo tolerance, meaning-level retrieval, explanation grounding,
or concept recall on real repositories.

A measurement is only usable if it is reproducible, free to rerun, and honest
about unavailable capabilities. Building that machinery is the prerequisite
for any comparison: fixed inputs and gold sets must be verifiable, recorded
model responses must replay without an external call, a missing measurement
must never become a zero score, and the report must state its coverage and
gaps. This change builds and proves that harness. The real candidate adapters,
fixed repositories, gold sets, and record run are delivered by
`benchmark-run`; until then the harness is proven against a committed fixture
whose every expected value is asserted.

## Proposed Solution

The maintainer runs one benchmark command and receives a report that follows
one contract for every input and candidate:

- The fixed-input manifest names each repository and revision, each
  candidate's version and commit, each gold set's version and fingerprint, and
  the cassette fingerprints. A mismatch stops the run with exit code 2 and
  publishes no report.
- Candidate results are normalized to ranked locations and recorded in result
  cassettes; model requests and responses are recorded in model cassettes.
  Replay reads the cassettes, starts no candidate, and makes no external call,
  so the same manifest reproduces the same measurements and order.
- The harness scores typo recall at ranks one and five with a separate
  no-match rate, natural-language recall at five and mean reciprocal rank at
  ten, explanation correctness with evidence grounding, and concept and
  concept-equivalence recall with default and no-expansion arms.
- Every metric cell is `measured`, `unavailable` with a reason, or
  `not_applicable` with a reason. Metric/task coverage carries its numerator
  and denominator. A missing capability is never a zero score, and a metric
  with no measured candidate reports an unavailable gap.
- The report names the fixed inputs, candidate setup, index status and
  deadlines, measured values, coverage, capability gaps, and measured gaps.
  It states the recording mode, the fingerprints, and the exact replay
  command. It pre-registers no build threshold and records the product-path
  decision as deferred to `maintainer--ortsinn` and
  `docs/decisions/benchmark-path.md`.
- The harness is proven on a committed fixture: `fixture-repo` at revision
  `fixture-1` with three fixture candidate identities, fixed gold task counts,
  fixed cassette fingerprints, and exact expected metric values. The regular
  scenarios and two `@e2e` CLI scenarios assert those values.

`benchmark-run` then adds the real adapters, the pinned candidate installs,
the fixed reference repositories and their gold sets, the record run, and the
published measured report. This change fixes the contract those steps must
satisfy.

The benchmark is a project gate, not a runtime capability of the product, and
the product remains MCP-only.

## Capability Routing

`givn spec route --change benchmark-gate` reports no signal: the change has no
lexical, token, or command overlap with an existing capability. The decision
is recorded directly.

| Capability | Decision | Rationale |
|---|---|---|
| `benchmark-comparison` | `NEW in benchmark-comparison` | No permanent capability covers a maintainer-run comparison of external tools. This change adds the harness and report contract; `benchmark-run` extends the same capability with the real-candidate measurement. It is a project gate, not a product runtime use case. |

## Out of Scope

- Real candidate adapters: open-codebase-index, CKB, and Serena, the `rmcp`
  stdio client, and candidate setup/indexer probes. `benchmark-run` owns them.
- The real fixed inputs: `benchmark/manifest.yaml`, `benchmark/targets.yaml`,
  `benchmark/fetch-repos.sh`, the public reference repositories, the real
  gold sets, and the synthetic-monorepo production run. `benchmark-run` owns
  them.
- The real record run and the published real measured report.
  `benchmark-run` owns them.
- Product runtime behavior: the MCP query surface, workspace configuration,
  indexing, concept vocabulary, summaries, and impact are not designed or
  changed here.
- The build/contribute/compose decision itself. The report publishes the
  measured gaps; the decision is carried to the maintainer and
  `docs/decisions/benchmark-path.md`.
- Any contribution plan or upstream change.
- Q6 (owner visibility into external model calls and resource consumption) for
  the product. The benchmark's recorded model responses are a project-tool
  reproducibility artifact, not the product's owner-facing usage record.
- Q3 (name availability) and any release or packaging decision.
- Patching, optimizing, or reconfiguring a candidate beyond the documented
  setup normalization needed for a fair comparison.
- Throughput, memory, and cost benchmarking as product acceptance criteria.
  Indexing time and index status are recorded only to explain an unavailable
  measurement.

## Open Questions

### D1 — Benchmark decision rule (seed Q5)

- Question: what evidence decides between building new, contributing, and
  composing after the benchmark?
- Options:
  1. **Strict gap rule** — build only if the nearest project misses at least
     one confirmed acceptance threshold on a differentiator; gain: objective
     and protective against a duplicate; cost: one small miss can force a
     large build; in practice: a typo Recall@1 of 0.88 against 0.90 would
     force a new product.
  2. **Material gap rule** — build only if the nearest project misses by a
     stated margin or cannot close the gap without architectural change;
     gain: avoids building for marginal differences; cost: the margin is
     arbitrary and can hide a strategically important small gap; in practice:
     0.88 against 0.90 would not justify a build, 0.60 would.
  3. **Combination rule** — build if no single project can host the missing
     pieces without architectural change, otherwise contribute or compose;
     gain: focuses on architecture rather than scores; cost: requires a deep
     reading of each candidate and is slower; in practice: a storage choice
     that blocks multi-language precise navigation would justify a build even
     if scores are close.
  4. **Decide after the report** — run the benchmark, publish the measured
     gaps, and choose the path in a second decision with the data in hand;
     gain: no premature rule and thresholds can be calibrated; cost: the path
     is not pre-registered, so the report must define its own stopping point
     and the decision must name its owner and artifact; in practice: the
     maintainer reads the report and records the path afterwards.
- Disposition: answered — the user chose option 4 on 2026-10-20. The benchmark
  publishes measured gaps and pre-registers no build threshold; its scope
  (fixed inputs, candidates, metrics, report) is its stopping point. The
  resulting path decision is carried to `maintainer--ortsinn` and the
  post-report decision record `docs/decisions/benchmark-path.md`, which cites
  the published report.

### D2 — Reference repositories and gold sets (seed open item)

- Question: which exact repositories and commits are measured, and who
  produces the gold sets?
- Options:
  1. **Fix public repositories and curated gold sets** — pin three public
     repositories and the synthetic monorepo; the maintainer authors and
     reviews committed gold sets for each metric; gain: reproducible inputs
     and checkable expected answers; cost: substantial gold-set authoring;
     in practice: the TypeScript, Python, and Rust repositories are measured
     at recorded commits with a fixed number of reviewed typo tasks each.
  2. **Reuse external benchmark gold sets** — adopt an existing suite;
     gain: less authoring; cost: the repositories, languages, and metric
     definitions do not match the acceptance criteria, and the expected
     answers cannot be checked against the fixed revisions; in practice: an
     external suite's Python tasks would not cover the Rust reference.
  3. **Judge on the fly without gold sets** — score candidate output with a
     model or a human in each run; gain: no gold-set build; cost: the rerun is
     not deterministic, not free, and not comparable across runs; in practice:
     the same candidate output could score differently on two runs.
- Disposition: carried — owned by `benchmark-run`, owner
  `maintainer--ortsinn`. This change fixes the manifest/gold shape and the
  path/line validation the real inputs must satisfy; it does not author the
  real gold sets.

### D3 — Competitor versions and setup normalization (seed open item)

- Question: which versions of open-codebase-index, CKB, and Serena are
  compared, and how are their setup differences normalized?
- Options:
  1. **Latest at run time** — install whatever is current; gain: always
     current; cost: the run is not reproducible and two runs are not
     comparable; in practice: a candidate released during the benchmark
     changes the comparison between runs.
  2. **Pin releases and commits with documented normalization** — fix each
     candidate version and commit, install in an isolated toolchain, use each
     tool's documented indexing setup and a common query shape, and record
     every deviation; gain: deterministic and fair; cost: setup work and a
     containerized runner; in practice: open-codebase-index v0.37.0 and CKB
     v9.3.1 are installed in the runner image and the report records their
     commits.
  3. **Use opaque vendor images** — compare prebuilt images; gain: least
     setup; cost: the internal version, configuration, and capability set are
     not auditable and cannot be normalized transparently; in practice: a
     vendor image's internal configuration cannot be compared with the
     documented setup.
- Disposition: carried — owned by `benchmark-run`, owner
  `maintainer--ortsinn`. This change fixes the candidate/setup fields and the
  unavailable-never-zero rule the real candidates must populate.

### D4 — Fuzzy behavior of open-codebase-index (seed open item)

- Question: does open-codebase-index support typo-tolerant symbol search, and
  how is that established rather than assumed from the survey?
- Options:
  1. **Assume absence from the survey** — mark the capability missing without
     a run; gain: no work; cost: a false negative can justify a duplicate
     product; in practice: the report would mark typo search absent even if
     the tool returns the target at rank two.
  2. **Read its documentation and source** — inspect the implementation;
     gain: cheap; cost: the documented behavior may not match retrieval
     results on fixed tasks; in practice: documentation may mention substring
     matching, but the fixed typo tasks decide whether that is enough.
  3. **Measure typo tasks and report the observed behavior** — run the same
     typo gold set as every candidate and report rank-one recall, top-five
     recall, and the no-match rate; gain: behavioral evidence; cost: gold
     tasks and an adapter that can submit the queries; in practice: a typo
     query such as `getUserProfle` is scored at rank one and rank five, and a
     no-result is counted separately.
- Disposition: carried — owned by `benchmark-run`, owner
  `maintainer--ortsinn`. This change implements the typo metric (rank-one,
  top-five, and no-match rate) and proves it on the fixture; the real
  open-codebase-index measurement runs in `benchmark-run`.
