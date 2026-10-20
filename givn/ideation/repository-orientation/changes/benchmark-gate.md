# Change seed: benchmark-gate

Status: proposed by benchmark-gate

- Use-case IDs: none — this is a project gate, not a product use case.
- Topic: `repository-orientation`
- Personas: maintainer--ortsinn; upstream--maintainer (conditional on the
  outcome)
- Suggested start: 1, first deliverable; no dependency.
- Handoff: ready.

## Accepted decisions

### Benchmark before building

- Context: the market survey found a dense field. open-codebase-index already
  combines a Rust core, Tree-sitter, SQLite, usearch, BM25, a watcher, and a
  call graph under MIT; CKB orchestrates SCIP, LSP, and Git; Serena is
  LSP-based. None documents the full combination: explicit precision levels,
  multi-language SCIP, typo-tolerant symbol search with measured recall, and
  hierarchical cached summaries.
- Options:
  1. **Build new** — full control; high effort; every component must beat the
     nearest project on a measured criterion.
  2. **Contribute to open-codebase-index** — fastest user value; TypeScript
     orchestration; upstream direction; MPL-2.0 code offered under MIT.
  3. **Compose** — a small Rust binary for fuzzy search, precise aggregation,
     and summaries; two MCP servers; harder fusion.
  4. **Benchmark first** — measure the nearest tools against the acceptance
     criteria, then choose.
- Chosen: **Benchmark first**. Rejected: build new without evidence;
  contribute without testing the nearest project's fuzzy and concept behavior;
  compose before knowing which gaps are separable.

### Benchmark basis

- Compare on three public reference repositories (TypeScript, Python, Rust;
  at least 200k LOC each) and one synthetic 10M-LOC monorepo.
- Measure the typo-recall criterion (AC-01), natural-language retrieval
  (AC-03), explanation correctness (AC-12), and concept recall (AC-21/AC-22).
- Fix repository commits, record embedding calls, and replay them so the
  comparison is deterministic and free to rerun.
- Include open-codebase-index, CKB, and Serena; add another tool only if it
  closes a coverage gap in the comparison.

## Open questions

### Q5 — Benchmark decision rule

What result decides between building new, contributing, and composing? The
options and their consequences are in
`givn/ideation/repository-orientation/questions.md` (Q5). Without a rule
stated before the run, the same data can justify any path.

Resolution note: Q5 is closed with option 4 (decide after the report; source
user/2026-10-20). The proposal's D1 disposition and the design's `## Decisions`
ledger record the outcome. The seed text above is kept for provenance.

### Reference repositories and gold sets

Which exact repositories and commits, and who produces the gold sets? The
acceptance criteria name sizes and languages but not the repositories.

Resolution note: resolved by proposal D2 and `design.md` ("Fixed inputs",
"Gold sets"); the manifest pins the repositories, and the maintainer owns the
reviewed gold sets.

### Competitor versions

Which versions of open-codebase-index, CKB, and Serena are compared, and how
are their setup differences normalized?

Resolution note: resolved by proposal D3 and `design.md` ("Candidates and
setup normalization"); candidate releases and commits are pinned and every
setup deviation is recorded.

### Fuzzy behavior of open-codebase-index

The survey records no explicit fuzzy-symbol search and no test of its
behavior; the benchmark must test it rather than assume its absence.

Resolution note: resolved by proposal D4 and `design.md` ("Metrics"); every
candidate is measured on the same typo tasks, with rank-one recall, top-five
recall, and the no-match rate.
