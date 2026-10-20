# Change seed: impact-graph

Status: ready

- Use-case IDs: UC9
- Topic: `repository-orientation`
- Personas: agent--coding-harness, developer--agent-first
- Suggested start: 5; depends on `workspace-index` and
  `query-foundation`.
- Handoff: ready.

## Accepted decisions

### Cross-repository impact

- The graph spans the workspace; dependents can be in another repository.
- Direction `in` returns dependents (impact); `out` returns dependencies and
  is labeled as such, not as impact.
- Edge kinds: `imports` and `contains` from the syntactic graph; `calls` and
  `implements` from the precise graph, with a syntactic name-matched `calls`
  fallback.
- The response may mix edge precisions; it reports precise and syntactic edge
  counts, and each edge carries its own level.
- Cycles are detected and reported with node identifiers; the repeated node
  is not re-expanded.
- Depth defaults to 2 and is bounded to 5; a depth-bounded or filtered result
  reports `truncated` and boundary nodes.

## Open questions

### Graph storage

Which store implements the graph port: an embedded graph library, SQLite
edges, or a graph database? The source material proposes SQLite plus a graph
library; the choice belongs to `/givn-design`.

### Precise edge coverage

Which languages provide precise call and implementation edges, and what does
the result report when they do not?

### Default depth

Is 2 the right default for the agent's context budget, or should the default
depend on the edge kind?
