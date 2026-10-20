# Use Case Definition — repository orientation

- Topic: `repository-orientation`
- Status: confirmed 2026-10-20 by the user; phase review findings applied;
  elaboration complete.
- Source: the 32-event chain in `eventstorming.md`; the confirmed Frame and
  the 18 confirmed terms.
- The benchmark (Q5) is a project gate, not a product use case. It is excluded
  from this cut and belongs to the Handoff seeds.

## Boundaries

| Boundary | Event chain | Use cases |
|---|---|---|
| Workspace | E1-E3 | UC1 |
| Indexing | E4-E12, E28 | UC2, UC3, UC11 |
| Vocabulary | E13-E18, E29 | UC4, UC5 |
| Query | E19-E24, E27, E30-E32 | UC6, UC7, UC8, UC9, UC10 |
| Summary | E25-E26 | UC8.3, supporting context |

Term collisions mark the boundaries:

- **Workspace vs repository**: one workspace covers several repositories;
  UC1 configures the workspace, not a repository's content.
- **Index vs indexed file**: UC2 is the workspace-level goal; Indexed file is
  the per-file consistency boundary, not a use case.
- **Concept vocabulary vs glossary**: UC4 changes the vocabulary; UC10 reads
  the glossary; UC5 reads the glossary and the symbol table.
- **Locate vs navigate**: UC6 starts from a question; UC7 starts from a known
  symbol.
- **Understand**: UC8 starts from a chosen file or symbol; UC6 only produces
  candidates.
- **Freshness**: UC11 observes it; UC2 changes it.
- **Index level vs precision level**: freshness is reported per index level
  (syntactic, semantic, precise); a navigation answer carries a precision
  level (precise or syntactic).
- **Impact**: UC9 reads the dependency graph; it never writes it.

## Use cases

The Personas column names runtime review lenses. The Ortsinn maintainer, the
adapter implementer, and the upstream maintainer review the cut at the project
gate; they are not runtime actors.

| ID | Level | Goal (observable outcome) | Primary actor | Boundary | Kind | Personas |
|---|---|---|---|---|---|---|
| UC1 | `!` | The workspace defines the repositories, ignore rules, model endpoint, offline mode, indexing schedule, and test paths that the index covers. | Repository owner | Workspace | changing | repository-owner, developer (supporting), provider |
| UC2 | `!` | The index reflects the current content of the workspace repositories. | Repository owner | Indexing | changing | repository-owner, developer (supporting), provider |
| UC3 | `!` | The owner knows which files failed to index and why. | Repository owner | Indexing | observing | repository-owner, developer (supporting) |
| UC4 | `!` | Every accepted term maps to one concept with evidence in the glossary. | Developer | Vocabulary | changing | developer, provider |
| UC5 | `!` | The developer sees concepts with undeclared term variants. | Developer | Vocabulary | observing | developer |
| UC6 | `!` | The agent for the developer gets a ranked set of candidate code locations for a name or meaning query, with match basis, expanded terms, and evidence. | Agent for the Developer | Query | observing | agent, developer, provider |
| UC7 | `+` | The agent for the developer resolves a symbol to its definition, references, or implementations. | Agent for the Developer | Query | observing | agent, developer |
| UC7.1 | `!` | The agent for the developer gets the definition of a symbol with its precision level. | Agent for the Developer | Query | observing | agent, developer |
| UC7.2 | `!` | The agent for the developer gets the references of a symbol with their precision level. | Agent for the Developer | Query | observing | agent, developer |
| UC7.3 | `!` | The agent for the developer gets the implementations of a symbol. | Agent for the Developer | Query | observing | agent, developer |
| UC8 | `+` | The agent for the developer gets enough context to understand a file or symbol. | Agent for the Developer | Query | observing | agent, developer |
| UC8.1 | `!` | The agent for the developer gets the symbols of a file. | Agent for the Developer | Query | observing | agent, developer |
| UC8.2 | `!` | The agent for the developer gets the source of a symbol with surrounding context. | Agent for the Developer | Query | observing | agent, developer |
| UC8.3 | `!` | The agent for the developer gets an evidence-backed explanation of a symbol, file, or module. | Agent for the Developer | Query | observing | agent, developer, provider |
| UC9 | `!` | The agent for the developer gets the transitive dependents affected by a change to a symbol. | Agent for the Developer | Query | observing | agent, developer |
| UC10 | `!` | The agent for the developer gets the glossary: the project's concepts and accepted terms. | Agent for the Developer | Query | observing | agent, developer |
| UC11 | `!` | The agent for the developer knows whether the index is current and which index level is stale. | Agent for the Developer | Indexing | observing | agent, developer, repository-owner (supporting) |

## Use-case detail (definition level)

### UC1 — Configure the workspace

- Goal: the workspace defines the repositories, ignore rules, model endpoint,
  offline mode, indexing schedule, and test paths that the index covers.
- Primary actor: Repository owner. Supporting: Developer applies the local
  model endpoint under the owner's policy.
- Trigger: the owner requests a workspace configuration change.
- Preconditions: none.
- Success guarantee: a workspace definition exists with the chosen
  repositories, ignore rules, model endpoint, offline mode, indexing
  schedule, and test paths.
- Minimal guarantee: an invalid repository or rule is rejected with the
  previous workspace unchanged.
- Out of scope: changing repository content; indexing the files (UC2).
- Includes: none. Extends: none.

### UC2 — Restore index freshness

- Goal: the index reflects the current content of the workspace repositories.
- Primary actor: Repository owner.
- Trigger: the file watcher observes a file change (E4), or the owner
  requests a rebuild (E28).
- Preconditions: a workspace is configured.
- Success guarantee: every eligible file in the workspace is indexed from its
  current content; freshness is current.
- Minimal guarantee: a failed file does not abort the run; it is recorded in
  the error list, and untouched files keep their previous index.
- Out of scope: changing repository content; deciding ignore rules (UC1).
- Includes: none. Extends: none.

### UC3 — Diagnose indexing failures

- Goal: the owner knows which files failed to index and why.
- Primary actor: Repository owner. Supporting: Developer reads the same
  report on the local machine.
- Trigger: the owner opens the error report after a run.
- Preconditions: a workspace is configured; the failure log may be empty.
- Success guarantee: the report names each failed file, the failure kind, and
  the time; no mutating interaction.
- Minimal guarantee: one failure does not hide the others.
- Out of scope: fixing the files; rebuilding (UC2).
- Includes: none. Extends: none.

### UC4 — Curate the concept vocabulary

- Goal: every accepted term maps to one concept with evidence in the glossary.
- Primary actor: Developer.
- Trigger: the developer requests a vocabulary change.
- Preconditions: the syntactic index is built for bootstrap, declare, and
  map; merge and reject do not require it.
- Success guarantee: each accepted term maps to one concept with evidence.
- Rule: merging two concepts preserves all their terms (the Concept vocabulary
  invariant); it is not a second outcome of this goal.
- Minimal guarantee: an invalid or rejected mapping leaves the previous
  vocabulary intact.
- Out of scope: editing repository identifiers; UC5 reports inconsistencies.
- Includes: none. Extends: none.

### UC5 — Inspect naming inconsistencies

- Goal: the developer sees concepts with undeclared term variants.
- Primary actor: Developer.
- Trigger: a term is mapped (E15); the inconsistency detection policy reports
  an undeclared variant (E18); the developer reviews the report.
- Preconditions: a workspace is configured; the symbol level may be missing
  or stale; a vocabulary exists, possibly empty.
- Success guarantee: each reported concept names its variants and their
  evidence; no mutating interaction.
- Minimal guarantee: an empty vocabulary yields an empty report, not an error.
- Out of scope: mapping the variants (UC4).
- Includes: none. Extends: none.

### UC6 — Locate code

- Goal: the agent for the developer gets a ranked set of candidate code
  locations for a name or meaning query, with match basis, expanded terms, and
  evidence.
- Primary actor: Agent for the Developer.
- Trigger: the agent receives a question that needs a location.
- Preconditions: a workspace is configured; the index may be empty or stale.
- Success guarantee: a ranked, paginated set under a stated token budget, with
  match basis (exact-symbol, typo, lexical, synonym, translation,
  meaning-level), the expanded terms, and evidence; no mutating interaction.
- Minimal guarantee: no candidate yields an empty result with the query and
  search modes stated, plus index freshness; not an error.
- Out of scope: resolving a known symbol (UC7); understanding a chosen
  location (UC8).
- Includes: none. Extends: none.
- Capabilities: typo-tolerant symbol search, lexical search, meaning-level
  search, hybrid fusion, query expansion.

### UC7 — Navigate code (`+`)

- Goal: the agent for the developer resolves a symbol to its definition,
  references, or implementations. The coverage is the union of the children.
- Primary actor: Agent for the Developer.
- Kind: observing.
- Includes: UC7.1, UC7.2, UC7.3.
- Direct capabilities: none.
- Success guarantee: UC7.1, UC7.2, and UC7.3 each complete for the requested
  symbol; a child with no matches returns its empty result under its minimal
  guarantee.
- Flow rule: the query dispatches to the requested child; the parent never
  fabricates a child result.
- Minimal guarantee: a missing precise index yields a syntactic result marked
  as such, not an error.
- Extends: none.

### UC7.1 — Resolve a definition (`!`)

- Goal: the agent for the developer gets the definition of a symbol with its
  precision level.
- Trigger: the agent requests the definition for a symbol name or location.
- Success guarantee: definition locations with precision level and evidence.
- Minimal guarantee: no definition yields an empty result marked syntactic,
  not a fabricated location.
- Observing.

### UC7.2 — Find references (`!`)

- Goal: the agent for the developer gets the references of a symbol with their
  precision level.
- Trigger: the agent requests references for a symbol identity.
- Success guarantee: a paginated set of reference locations under a stated
  token budget, with precision level and evidence.
- Minimal guarantee: no references yields an empty set, not an error.
- Observing.

### UC7.3 — Find implementations (`!`)

- Goal: the agent for the developer gets the implementations of a symbol.
- Trigger: the agent requests implementations for an interface or abstract
  symbol.
- Success guarantee: implementation locations with precision level and
  evidence.
- Minimal guarantee: no implementations yields an empty set; a language
  without implementation support says so.
- Observing.

### UC8 — Understand code (`+`)

- Goal: the agent for the developer gets enough context to understand a file
  or symbol. The coverage is the union of the children.
- Primary actor: Agent for the Developer.
- Kind: observing.
- Includes: UC8.1, UC8.2, UC8.3.
- Direct capabilities: none.
- Success guarantee: UC8.1, UC8.2, and UC8.3 each complete for the requested
  file or symbol; a child with no result returns its minimal-guarantee result.
- Flow rule: the query dispatches to the requested child; the parent never
  fabricates a child result.
- Minimal guarantee: when the semantic index level is missing or stale, the
  structural result is returned.
- Extends: none.

### UC8.1 — Inspect file symbols (`!`)

- Goal: the agent for the developer gets the symbols of a file.
- Trigger: the agent requests the symbols of a file path.
- Success guarantee: the symbols of the file with kinds, ranges, and
  precision level.
- Minimal guarantee: an unparseable file yields the error with its path, not
  an empty result presented as complete.
- Observing.

### UC8.2 — Retrieve symbol source (`!`)

- Goal: the agent for the developer gets the source of a symbol with
  surrounding context.
- Trigger: the agent requests the source of a symbol.
- Success guarantee: the source range with evidence and the symbol's location.
- Minimal guarantee: a missing symbol yields an error naming it, not another
  symbol's source.
- Observing.

### UC8.3 — Explain code (`!`)

- Goal: the agent for the developer gets an evidence-backed explanation of a
  symbol, file, or module directory.
- Trigger: the agent requests an explanation.
- Success guarantee: an explanation with at least one file:line evidence; no
  mutating interaction by the actor.
- Minimal guarantee: when the semantic index level or the model endpoint is
  unavailable, the answer says the explanation is unavailable; structural
  answers still work.
- Subfunction: summary generation and caching run as a system policy inside
  the flow; they are not a mutating interaction of this observing use case.
- Observing.

### UC9 — Assess change impact

- Goal: the agent for the developer gets the transitive dependents affected by
  a change to a symbol.
- Primary actor: Agent for the Developer.
- Trigger: the agent requests the impact of a proposed change.
- Preconditions: a workspace is configured; the dependency graph may be
  missing or partial.
- Success guarantee: a depth-bounded set of dependents with evidence and edge
  kinds; no mutating interaction.
- Minimal guarantee: a missing graph yields an empty set with the reason, not
  a false "no impact".
- Out of scope: editing call sites; the product is read-only.
- Includes: none. Extends: none.

### UC10 — Read the glossary

- Goal: the agent for the developer gets the glossary: the project's concepts
  and accepted terms.
- Primary actor: Agent for the Developer. Boundary: Query; the Glossary is the
  read model (event E32).
- Trigger: the agent requests the glossary.
- Preconditions: a vocabulary exists, possibly empty.
- Success guarantee: the glossary with concepts, terms, languages,
  identifiers, and evidence; no mutating interaction.
- Minimal guarantee: an empty vocabulary yields an empty glossary, not an
  error.
- Out of scope: curating it (UC4).
- Includes: none. Extends: none.

### UC11 — Check index freshness

- Goal: the agent for the developer knows whether the index is current and
  which index level is stale.
- Primary actor: Agent for the Developer. Supporting: Repository owner reads
  the same status for operations.
- Trigger: the agent requests index status.
- Preconditions: a workspace is configured.
- Success guarantee: freshness per index level (syntactic, semantic, precise)
  and the age; no mutating interaction.
- Minimal guarantee: an index that has never run reports "not built", not
  "fresh".
- Out of scope: rebuilding (UC2).
- Includes: none. Extends: none.

## Subfunctions (`-`, not permanent use cases)

These remain inside the flows above and never become their own use-case
directories:

- Index file, remove file, load precise index, produce embeddings, update
  dependency graph, invalidate summary — inside UC2.
- Expand query, fuse results — inside UC6.
- Generate summary — a system subfunction inside UC8.3.
- Detect inconsistency — inside UC5.

## Excluded goals

- Benchmark the nearest tools against the acceptance criteria — project gate
  (Q5), not a product use case.
- Query the index directly from a CLI or editor — Q1 closed MCP-only; the CLI
  is administrative.
- Edit or refactor code — out of product scope.
- Hosted or multi-tenant service — out.
- Replace an editor or LSP server — out.
- Change a repository's shared conventions — the product reports
  inconsistency (UC5); it does not edit the repository.
- Configure the agent harness — the harness is external.
- Review adapter conformance — contributor project activity, not runtime.
- Release the product — maintainer project activity, not runtime.

## Sizing checks

- **One goal, one primary actor**: each `!` row names one actor and one
  outcome. Supporting actors are marked and do not become a second goal.
- **Goal-shaped**: no goal names a technical component, mechanism, or channel.
  Search modes are capabilities inside UC6, not separate use cases.
- **Level discipline**: UC7 and UC8 are `+` with zero direct capabilities and
  only child `!` via Includes. Every other use case is `!`.
- **Mutation containment**: observing cases (UC3, UC5, UC6, UC7, UC8, UC9,
  UC10, UC11) own no mutating interaction by the actor. Changing cases are
  UC1, UC2, UC4. The summary cache write in UC8.3 is a system subfunction, not
  an actor interaction.
- **Extension discipline**: no extensions at definition time. Recovery paths
  in minimal guarantees become step-referenced extensions at elaboration.
- **Precondition and flow consistency**: preconditions are not established by
  the flow; triggers are actor requests or observed events.
- **Implementation independence**: no SCIP, tree-sitter, MCP, tantivy, usearch,
  or SQLite in a goal, trigger, or guarantee.
- **Relationship semantics**: Includes forms a two-level acyclic graph; `+`
  use cases include only `!` children; the parent guarantee is the union of
  the child guarantees.

## Persona value ranking (advisory)

Ranking from the confirmed personas; 1 = highest value. The maintainer,
adapter implementer, and upstream maintainer are project-gate reviewers, not
runtime actors.

| Persona | Ranking |
|---|---|
| Agent | UC6, UC7, UC8, UC9, UC11, UC10 |
| Developer | UC6, UC7, UC1, UC4, UC8, UC9, UC2, UC5, UC3, UC11, UC10 |
| Repository owner | UC2, UC1, UC3, UC11 |
| Ortsinn maintainer | UC6, UC2, UC9, UC4, UC10 (adoption/differentiation view; project gate) |
| Adapter implementer | UC7, UC8.1, UC11, UC2, UC9, UC6 (contract view; project gate) |
| Model endpoint operator | UC2, UC8.3, UC6, UC4, UC1 (request-volume view) |
| Upstream maintainer | none while contribution is conditional |

Persona findings applied:

- The agent's output bounding is in the success guarantees of UC6, UC7.2, and
  UC8.
- The developer is a supporting actor on UC1, UC2, and UC3.
- The provider is tagged on every model-calling use case.
- The repository owner is a supporting actor on UC11.
- The maintainer and contributor tags are project-level, not runtime.

Open question: the repository owner's visibility into external model calls and
resource consumption (Q6). It blocks the owner's success criterion; the cut
does not invent a runtime use case for it.

## Review corrections applied

- UC4's conjunctive success guarantee split; the merge rule moved to Rules.
- UC1's indexing precondition removed from its success guarantee and
  supporting actor.
- UC7/UC8 parent guarantees now match Includes and the union of child
  guarantees.
- UC11 now reports freshness per index level, not per precision level.
- UC9/UC11 preconditions no longer make their minimal guarantees unreachable.
- Boundary table: E28/E12 moved to Indexing; E26 moved to Summary; E32 anchors
  UC10 in Query.
- Triggers rewritten as requests or observed events.
- Non-human actor notation "Agent for the Developer" applied.
- UC8.3 stays observing; summary generation/caching is a system subfunction.
- UC8.1 renamed to "Inspect file symbols"; "meaning-level" replaces
  "semantic" as a match basis.
- Persona tags reconciled with the runtime/project split.
