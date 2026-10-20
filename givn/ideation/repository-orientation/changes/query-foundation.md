# Change seed: query-foundation

Status: ready

- Use-case IDs: UC6 (symbol and lexical modes), UC7.1, UC7.2, UC7.3, UC8.1,
  UC8.2
- Topic: `repository-orientation`
- Personas: agent--coding-harness, developer--agent-first
- Suggested start: 3; depends on `workspace-index`.
- Handoff: ready.

## Accepted decisions

### MCP is the only query surface

- Context: the opening said "not only an MCP"; no attachment expanded it.
- Options: MCP only with an administrative CLI; MCP plus a query CLI; MCP plus
  an LSP bridge; an embeddable core.
- Chosen: **MCP only; the CLI stays administrative**. Rejected: a query CLI
  and an LSP bridge because they add a second actor and output contract
  without a confirmed need.

### Identity and precision

- A symbol identity is `{qualified_name, short_name, kind, container,
  language, repository, signature}`; `qualified_name` is container-qualified.
- Every navigation result states `precision` (`precise` or `syntactic`) and
  `index_level`; a precise response is preferred and a syntactic fallback is
  marked with a reason.
- Ranges are 1-based and inclusive; columns count Unicode code points; paths
  are repository-relative.
- Each result is paginated under a 4,000-token default budget with a hard cap
  and a cursor bound to the input, filters, page parameters, and index
  revision.

### Query shape

- Every outcome uses one envelope with `status`, `errors`, `resolution`,
  `freshness`, `budget`, `cursor`, `empty_reason`, and counts.
- Match bases are `exact-symbol`, `typo`, `lexical`, `synonym`, `translation`,
  and `meaning-level`; a candidate may carry several.
- Typo resolution uses Damerau-Levenshtein over the case-folded last segment,
  distance at most two; exact matches suppress typo candidates.

## Open questions

### MCP tool schemas

The tool names and schemas are a design concern. The use cases define the
domain result shapes, not the transport serialization.

### Precise indexer coverage

Which languages have a precise index, and how is a missing precise level
reported without hiding a syntactic definition?

### Syntactic reference accuracy

Syntactic references are name-matched and may include unrelated same-named
symbols; the accepted precision and the agent guidance are design and benchmark
concerns.
