# Change seed: workspace-index

Status: ready

- Use-case IDs: UC1, UC2, UC3, UC11
- Topic: `repository-orientation`
- Personas: repository-owner--platform-lead,
  developer--agent-first (supporting), provider--model-endpoint (supporting)
- Suggested start: 2; depends on the benchmark gate choosing the build-new
  branch (or on an equivalent contribution plan).
- Handoff: ready.

## Accepted decisions

### One index over several repositories

- Context: the product must answer cross-repository references and impact for
  a multi-repository product.
- Options:
  1. **One repository per index** — simple ownership; duplicates indexes for
     shared libraries; no cross-repository references.
  2. **Several repositories per index** — one workspace index; cross-repository
     navigation; workspace-level configuration and freshness.
- Chosen: **Several repositories per index**. Rejected: one repository per
  index because it cannot answer cross-repository references.

### Index levels and freshness

- The index reports freshness per repository, language, and level:
  `syntactic`, `semantic`, `precise`.
- Incremental file changes update the syntactic level immediately; semantic
  and precise work follow and report their own state.
- Per-file consistency: a file's derived entries share its content hash.
- A failed file does not abort the run; it is recorded and its previous index
  is not marked fresh.

### Configuration is not content

- Workspace configuration advances a `config_revision`; the index content
  advances a `content_revision`.
- Freshness is judged against the content revision, so adding a repository or
  changing the schedule does not make the index stale.
- Removing a repository or adding an ignore rule marks affected entries for
  removal; UC2 purges them before freshness advances.

## Open questions

### Q6 — Owner visibility into model calls and resource consumption

Which owner-visible record should the product keep: every external call and
run, aggregates only, or none? This is costly to reverse because it shapes
the read model and retention. Options are in
`givn/ideation/repository-orientation/questions.md` (Q6).

### Embedding model identity

Which embedding model is configured, and how is a model change detected and
re-embedded within the per-run cap?

### Store adapters

Which embedded stores implement the index, symbol, vector, and graph ports?
The source material proposes SQLite, tantivy, and usearch; the choice belongs
to `/givn-design`.

### Precise indexer coverage

Which languages have precise indexers, and what does the workspace report
when a language has none?
