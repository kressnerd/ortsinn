# Open questions — repository orientation

## Q1 — Human query surface beyond MCP

- Status: closed 2026-10-20 (user)
- Blocks: the Frame's consumer list and every later use case whose primary
  actor is a person; decides whether the first deliverable is agent-only or
  also human-facing.
- Context: the design material names the consumer as "Agent über MCP" and
  gives the CLI only administrative commands (index, status, bench). The
  opening statement says Ortsinn is "not only an MCP for agentic harnesses",
  but no attachment expands that. Every use case needs one primary actor. If
  people query the same index directly, their goals must be modeled; if not,
  the person stays a beneficiary and the agent is the only actor. The answer
  does not decide storage, transport, or language. It is asked for the
  build-new branch: if the benchmark leads to a contribution, the upstream
  project's surface decides and this answer is revisited.
- Question: which cumulative set of consumer surfaces beyond MCP is in the
  first scope of Ortsinn?
- Options:
  1. **MCP only; CLI stays administrative.**
     In practice: `ortsinn index` builds the index; the developer asks the
     agent "where is getUserProfile used?" and the agent calls the MCP tools.
     Gain: one query contract, one actor, smallest first scope.
     Cost: a person cannot query without an agent; the "not only MCP" claim
     stays a future statement.
  2. **MCP plus a query CLI (adds to option 1).**
     In practice: `ortsinn search getUserProfile`, `ortsinn outline
     src/user.rs`, and `ortsinn impact <symbol>` print the same ranked results
     in a terminal.
     Gain: direct human orientation and verification; scriptable in CI; the
     core is exercised without an agent.
     Cost: a second output contract (formats, pagination, exit codes) and
     human use cases must be modeled and tested; a CI operator becomes a
     stakeholder.
  3. **MCP plus a query CLI plus an LSP bridge (adds to option 2).**
     In practice: an editor asks the bridge for go-to-definition and
     find-references; the bridge answers from the same index.
     Gain: catches developers who do not use an agent; reuses the precise
     index.
     Cost: a third interface, editor-specific behavior, and direct comparison
     with existing LSP tooling; an editor user becomes a stakeholder.
- Deferred: whether the core is an embeddable library with a public API is a
  design decision, not a consumer-surface cut; it is not part of this
  question.
- Resolution: **MCP only; CLI stays administrative** because the first scope
  keeps one query contract and one actor; the opening's "not only MCP" is a
  future direction, not a first-deliverable commitment. Rejected: **MCP plus a
  query CLI** because it adds a second output contract and human use cases
  without a confirmed need; **MCP plus query CLI plus LSP bridge** because it
  adds a third interface and competes with existing LSP tooling.

## Q2 — New build, contribution, or composition

- Status: closed 2026-10-20 (user)
- Blocks: whether this topic produces use cases for a new product or a
  contribution plan, and whether upstream maintainers become stakeholders;
  shapes every change seed at Handoff.
- Context: the market survey found open-codebase-index (Rust core with
  Tree-sitter, SQLite, usearch, BM25, watcher, call graph; MIT), CKB
  (SCIP/LSP/Git orchestration), Serena (LSP), and others. None of the surveyed
  projects documents the full combination: explicit precision levels in every
  answer, multi-language SCIP, typo-tolerant symbol search with measured
  recall, and hierarchical cached summaries. The survey recommends
  benchmarking before building and names three options.
- Question: should the first deliverable be a new Rust product, a contribution
  to an existing MIT project, or a composition of existing tools?
- Options:
  1. **Build new.**
     In practice: Ortsinn ships its own daemon and index; success is measured
     against the attachment's AC-01, AC-03, and AC-12 on the three reference
     repositories.
     Gain: full control over precision levels, fuzzy metrics, and summaries;
     pure Rust.
     Cost: high effort in a dense field; every component must beat the nearest
     project on a measured criterion to justify itself.
  2. **Contribute to open-codebase-index.**
     In practice: fuzzy symbol search, multi-language SCIP, and summaries are
     added to the existing MIT project.
     Gain: fastest path to user value; existing watcher, stores, and hybrid
     retrieval are reused.
     Cost: orchestration stays TypeScript; the Rust-only preference is not
     met; upstream review and direction apply. The repository is MPL-2.0,
     while the upstream project is MIT: contributed code is offered under the
     upstream terms, so the project's own licence does not carry over to it.
  3. **Compose.**
     In practice: a small Rust binary adds only fuzzy symbol search, SCIP
     aggregation, and summaries, and delegates vector and graph retrieval to
     an existing MCP server.
     Gain: least effort; each component stays with its owner.
     Cost: the user runs two MCP servers; cross-tool fusion and consistent
     evidence are harder.
  4. **Benchmark first, then commit.**
     In practice: the typo-recall, natural-language retrieval, and
     explanation tests from the attachment run against open-codebase-index,
     CKB, and Serena on the three reference repositories; the result decides
     between options 1, 2, and 3.
     Gain: the build/contribute/compose choice rests on measured gaps instead
     of README claims; no duplicate product is started without evidence.
     Cost: the domain work pauses while the benchmark and its gold sets are
     built; the benchmark is itself substantial work.
- Resolution: **Benchmark first, then commit** because the market survey found
  a dense field and the combination gap is unproven; a measured comparison
  prevents building a duplicate. Rejected: **Build new** because no benchmark
  yet shows a gap that justifies a new product; **Contribute to
  open-codebase-index** because its fuzzy and concept-level behavior and the
  full combination were not tested; **Compose** because which gaps are
  separable without architectural change is unknown before the benchmark.

## Q3 — Name availability before publishing

- Status: open
- Blocks: the first public release and any package or repository name
  reservation; not the domain model.
- Context: the naming analysis found no crates.io data for `ortsinn` (the
  request failed, which is consistent with absence but not proof), did not
  evaluate GitHub search, and rejected `codeknow`, `RepoMind`,
  `CodeCartographer`, `cairn`, `lodestar`, and `wayfinder` as taken or
  colliding. The name `lotse` is the fallback but was not checked either.
- Question: should the name be verified now, or is `ortsinn` a working name
  until the first release?
- Options:
  1. **Verify now.**
     In practice: `cargo search ortsinn`, GitHub repository and organisation
     search, npm, and PyPI are checked before any artifact uses the name.
     Gain: no late rename; the published document and crates can use one name.
     Cost: a short detour before use cases.
  2. **Working name.**
     In practice: the topic and use cases say "Ortsinn"; verification happens
     before the first release.
     Gain: no detour; the domain work proceeds.
     Cost: a collision may force a rename across the topic document and seeds.
- Resolution: pending

## Q4 — One repository or several per index

- Status: closed 2026-10-20 (user)
- Blocks: the Frame's in-scope statement, the Repository candidate definition,
  and whether cross-repository references are a use case.
- Context: the design material says "beliebig große, mehrsprachige
  Repositories" and stores the index under the repository; the market survey
  lists a multi-repo tool. The Frame currently says "one or more
  repositories" because the question was never asked. If several repositories
  share one index, find-references can cross repository boundaries, and the
  index location, freshness, and ownership model change.
- Question: does one Ortsinn index cover exactly one repository, or several
  independent repositories?
- Options:
  1. **One repository per index.**
     In practice: `.ortsinn/` lives in the repository; queries never return
     paths from another checkout.
     Gain: simple ownership, freshness, and ignore rules; the index follows
     the repository.
     Cost: cross-repository navigation needs a separate index and no shared
     references.
  2. **Several repositories per index.**
     In practice: a workspace lists `api/`, `web/`, and `shared/`;
     find-references returns hits in all three.
     Gain: cross-repository navigation and impact.
     Cost: workspace configuration, shared freshness, and a larger index;
     ignore and ownership rules become workspace-level.
- Resolution: **Several repositories per index** because the product must
  answer cross-repository references and impact for a multi-repository
  product; one query must return hits across `api/`, `web/`, and `shared/`.
  Rejected: **One repository per index** because it duplicates indexes for
  shared libraries and cannot answer cross-repository references.

## Q5 — Benchmark decision rule

- Status: closed 2026-10-20 (user)
- Blocks: the stopping condition of the benchmark and the final
  build/contribute/compose choice; the benchmark report cannot conclude
  without it.
- Context: Q2 resolved to benchmark first. The typo-recall criterion (AC-01),
  natural-language retrieval criterion (AC-03), explanation criterion
  (AC-12), and the proposed concept-recall criteria (AC-21, AC-22) give
  thresholds, but not a rule that maps a competitor's result to build,
  contribute, or compose. Without a rule stated before the run, the same data
  can justify any path.
- Question: what evidence should decide between building new, contributing,
  and composing after the benchmark?
- Options:
  1. **Strict gap rule.**
     In practice: build only if the nearest project misses at least one
     confirmed acceptance threshold on a differentiator (typo-tolerant symbol
     search, multi-language precise navigation, concept-level retrieval, or
     hierarchical summaries); otherwise contribute to the nearest project or
     compose the separable gaps. Example: open-codebase-index reaches
     Recall@1 0.88 on the typo-recall criterion (AC-01, threshold 0.90) and
     has no concept-level retrieval, so build new.
     Gain: objective; protects against building a duplicate.
     Cost: a single threshold miss can force a large build even when the gap
     is small.
  2. **Material gap rule.**
     In practice: build only if the nearest project misses by a stated margin
     (for example at least 10 points on two criteria) or the gap cannot be
     closed upstream without architectural change. Example: 0.88 against 0.90
     does not justify a new product; 0.60 against 0.90 does.
     Gain: avoids building for marginal differences.
     Cost: the margin is arbitrary and may hide a strategically important
     small gap.
  3. **Combination rule.**
     In practice: build if no single project can host the missing pieces
     without architectural change; otherwise contribute to the closest
     project. Example: if open-codebase-index lacks precise navigation but its
     store and watcher can host it, contribute; if its storage choice blocks
     multi-language precise navigation, build.
     Gain: focuses on architecture, not scores.
     Cost: requires a deep reading of each candidate's architecture; slower.
  4. **Decide after the report.**
     In practice: run the benchmark, publish the measured gaps, and choose the
     path in a second decision with the data in hand.
     Gain: no premature rule; thresholds can be calibrated.
     Cost: the decision is not pre-registered; motivated reasoning is
     possible, and the benchmark needs a defined stopping point.
- Resolution: **Decide after the report** (option 4) because no rule should be
  pre-registered before the measured gaps exist; the benchmark publishes the
  measured gaps, and the maintainer records the build/contribute/compose
  decision in `docs/decisions/benchmark-path.md` afterwards. The benchmark's
  fixed inputs, candidates, metrics, and report are its stopping point.
  Source: user decision, 2026-10-20. Rejected: **Strict gap rule** because one
  threshold miss can force a large build on an unconfirmed target;
  **Material gap rule** because the margin is arbitrary and can hide a
  strategic small gap; **Combination rule** because it requires a deep
  architectural reading before any measurement exists.
  Note: closing this question does not confirm AC-01's 0.90 threshold. That
  number appears only as an illustrative example in option 1 above; the
  benchmark records it as a proposed target, not a confirmed one.

## Q6 — Owner visibility into model calls and resource consumption

- Status: open
- Blocks: whether the use-case cut covers the repository owner's success
  criterion ("current index without machine cost or code leaving the approved
  boundary"); shapes the read models and the offline policy.
- Context: the owner persona has authority over whether an external model
  endpoint may receive code and an interest in resource consumption. The event
  chain calls the endpoint for embeddings, vocabulary bootstrap, query
  expansion, and summaries, but records no recipient, payload class, duration,
  index size, or cost. The cut covers indexing failures (UC3) and freshness
  (UC11), not what left the machine or what a run consumed. This is a
  read-model gap, not only a missing use-case name.
- Question: which owner-visible record should the product keep for external
  model calls and resource consumption?
- Options:
  1. **Record every external call and indexing run.**
     In practice: the owner opens a usage report and sees "embeddings: 12,400
     chunks, 3.1M tokens, endpoint X; summaries: 84 calls"; each file's
     payload class is attributable.
     Gain: full audit and cost control; proves what left the machine.
     Cost: a new event, read model, and retention policy; the local index
     grows.
  2. **Record aggregates only.**
     In practice: the owner sees per-run and per-endpoint totals, not which
     file's content was sent.
     Gain: low overhead; enough for cost trends.
     Cost: no per-call audit; harder to prove the data boundary.
  3. **Keep no local record; rely on the endpoint provider's logs.**
     In practice: the owner checks the provider console.
     Gain: no product work.
     Cost: the owner cannot verify from the local machine; offline runs leave
     no trace.
- Resolution: pending
