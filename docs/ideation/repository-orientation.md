# Repository orientation — Ortsinn ideation

- Topic: `repository-orientation`
- Status: handoff complete; confirmed 2026-10-20
- Topic state: `givn/ideation/repository-orientation/`
- Product path: build, contribute, or compose remains pending the benchmark
  (Q2); this document describes the build-new branch and the shared domain.

## Problem

A developer or coding agent working in a large, multi-language repository
cannot reliably establish context — locating the right symbol despite typos,
paraphrases, translations, or inconsistent naming, proving who uses it, and
judging what a change breaks — because no local source answers location,
relation, and meaning together with verifiable evidence.

## Vision

Ortsinn gives its consumer a sense of place in a repository. It answers where a
thing is, how it relates, what it does, and what depends on it, from an index
that stays fresh as files change. It treats different words that name the same
thing — synonyms, translations, naming variants — as one meaning, not as
separate vocabulary. Answers are grounded in the repository; the consumer can
check them. It runs locally and is open source.

The source material proposes Rust, precision levels in every navigation answer,
file:line evidence, a bounded number of tool calls, typo-tolerant symbol
search, concept-level retrieval, query expansion, and a project glossary.
These are requirements, not frame facts.

## Scope

In scope: a local knowledge layer over a set of independent repositories;
read-only locate, navigate, understand, and impact; cross-repository
references; meaning-level retrieval; a project-level term record; MCP as the
only query surface; evidence-bearing answers.

Out of scope: hosted or multi-tenant service; code editing, refactoring, or
execution; replacing an editor or LSP server; explanations asserted without
evidence; direct human query from a CLI or editor.

## Stakeholders and personas

| Persona | Role | Authority boundary | Success criterion |
|---|---|---|---|
| Coding agent in an agent harness (`agent--coding-harness`) | Autonomous agent answering via tools | Queries and reads; never edits the repository or index | Correct, evidence-bearing location in few calls without exhausting context |
| Developer operating an agent harness (`developer--agent-first`) | Developer working through an IDE or terminal agent | Owns local setup and questions, not team policy | Verified location quickly; can tell whether the agent guessed |
| Repository owner / platform lead (`repository-owner--platform-lead`) | Accountable for repositories and developer machines | Decides workspace membership, ignore rules, external model use, indexing time | Current index without machine cost or code leaving the approved boundary |
| Open-source adapter implementer (`contributor--adapter-implementer`) | External contributor adding language or store adapters | Implements against ports and conformance suite; cannot change contracts or releases | Adapter passes conformance and improves coverage |
| Ortsinn maintainer (`maintainer--ortsinn`) | Owns direction, releases, licence | Sets scope and releases; cannot decide a user's repository policy | Sustainable adoption with a measured reason to exist |
| Model endpoint operator (`provider--model-endpoint`) | Operates the external OpenAI-compatible endpoint | Sets models, rate limits, pricing, retention; does not see the repository | Predictable, policy-compliant usage with no surprise cost |
| Upstream project maintainer (`upstream--maintainer`) | Maintainer of a project that may receive a contribution | Accepts or rejects contributions; sets upstream architecture and licence | Accepted contribution that closes a measured gap without raising maintenance burden |

The upstream maintainer is conditional on the benchmark leading to a
contribution.

## Research protocol

The project had no changes, no specs, an empty durable glossary, and only
arc42 templates when this topic started. The source material is seven user
attachments: a requirements and architecture proposal, a storage decision,
five usage scenarios, a component map, a name selection, a market survey, and
a semantic concept analysis. They are recorded as proposal material, not as
confirmed decisions. The market claims are from the survey and were not
independently re-verified. Benchmark evidence and name availability remain
open.

## Event storming

32 events across five runtime contexts: workspace, indexing, vocabulary,
query, and summary. Two aggregates:

| Aggregate | Invariant |
|---|---|
| Indexed file | Every symbol, chunk, and edge derived from a file carries that file's current content hash; no entry points outside its range; removal removes all entries. |
| Concept vocabulary | Each accepted term maps to at most one concept; every mapping cites evidence; merging preserves all terms. |

Workspace configuration, the index, symbol table, text/vector indexes,
dependency graph, precise index, freshness status, error list, glossary, and
summary cache are explicitly non-aggregates: configuration, read models, or
cache records. The repository, model endpoint, and precise indexer are
external systems. The full canvas is in
`givn/ideation/repository-orientation/eventstorming.md`.

## Use cases

15 leaf `!` use cases and 2 `+` summaries. Each leaf has an elaborated contract
under `givn/ideation/repository-orientation/use-cases/`.

| ID | Goal | Actor | Kind |
|---|---|---|---|
| UC1 | The workspace defines repositories, ignore rules, endpoint, offline mode, schedule, and test paths. | Repository owner | changing |
| UC2 | The index reflects the current content of the workspace repositories. | Repository owner | changing |
| UC3 | The owner knows which files failed to index and why. | Repository owner | observing |
| UC4 | Every accepted term maps to one concept with evidence. | Developer | changing |
| UC5 | The developer sees concepts with undeclared term variants. | Developer | observing |
| UC6 | The agent gets ranked candidate locations with match basis, expanded terms, and evidence. | Agent for the Developer | observing |
| UC7 | Navigate code: definition, references, or implementations. | Agent for the Developer | observing (`+`) |
| UC7.1 | Get a symbol's definition with its precision level. | Agent for the Developer | observing |
| UC7.2 | Get a symbol's references with their precision level. | Agent for the Developer | observing |
| UC7.3 | Get a symbol's implementations. | Agent for the Developer | observing |
| UC8 | Understand code: enough context for a file or symbol. | Agent for the Developer | observing (`+`) |
| UC8.1 | Get the symbols of a file. | Agent for the Developer | observing |
| UC8.2 | Get a symbol's source with surrounding context. | Agent for the Developer | observing |
| UC8.3 | Get an evidence-backed explanation of a symbol, file, or module. | Agent for the Developer | observing |
| UC9 | Get the transitive dependents affected by a change. | Agent for the Developer | observing |
| UC10 | Read the glossary: concepts and accepted terms. | Agent for the Developer | observing |
| UC11 | Know whether the index is current and which level is stale. | Agent for the Developer | observing |

### Use-case to permanent mapping

Each leaf maps to its own permanent capability; UC7 and UC8 map to summary
capabilities over their children.

| Use case | Permanent capability |
|---|---|
| UC1 | `workspace-configuration` |
| UC2 | `index-freshness` |
| UC3 | `index-failure-report` |
| UC4 | `concept-vocabulary` |
| UC5 | `naming-inconsistency-report` |
| UC6 | `code-location` |
| UC7 | `code-navigation` (summary) |
| UC7.1 | `definition-navigation` |
| UC7.2 | `reference-navigation` |
| UC7.3 | `implementation-navigation` |
| UC8 | `code-understanding` (summary) |
| UC8.1 | `file-symbol-listing` |
| UC8.2 | `symbol-source` |
| UC8.3 | `code-explanation` |
| UC9 | `change-impact` |
| UC10 | `glossary-read` |
| UC11 | `index-status` |

## Confirmed terms

All 18 terms are confirmed by the user; no term was rejected.

Workspace, Repository, Index, Indexed file, Symbol, Reference, Precision
level, Freshness, Chunk, Concept, Term, Concept vocabulary, Glossary, Query
expansion, Evidence, Impact, Summary, Naming inconsistency.

Definitions, contrasts, first uses, and collision checks are in
`givn/ideation/repository-orientation/domain-terms.md`.

## Open questions

| Question | Status | Blocks |
|---|---|---|
| Q1 Human query surface | Closed: MCP only; CLI administrative | — |
| Q2 Build, contribution, or composition | Closed: benchmark first | Product path |
| Q3 Name availability | Open | First public release |
| Q4 One or several repositories per index | Closed: several; one workspace index | — |
| Q5 Benchmark decision rule | Open | Stopping condition of the benchmark and the path choice |
| Q6 Owner visibility into model calls and resource consumption | Open | Owner's success criterion and audit/retention design |

## Change seeds

| Seed | Guarantee | Use cases | Depends on |
|---|---|---|---|
| `benchmark-gate` | A measured comparison of the nearest tools against the acceptance criteria, with a pre-registered path decision. | none (project gate) | — |
| `workspace-index` | A local workspace over several repositories keeps a fresh index, reports failures, and reports freshness. | UC1, UC2, UC3, UC11 | benchmark-gate (build-new) |
| `query-foundation` | An agent locates, navigates, and reads code with evidence, without semantic retrieval. | UC6 (symbol/lexical), UC7.1-7.3, UC8.1-8.2 | workspace-index |
| `concept-retrieval` | An agent retrieves by meaning across synonyms and translations and gets grounded explanations. | UC4, UC5, UC10, UC6 (meaning-level), UC8.3 | query-foundation |
| `impact-graph` | An agent gets the transitive dependents of a change across repositories. | UC9 | workspace-index, query-foundation |

Seeds are inputs to `/givn-propose`, not proposals.

## Handoff record

- Frame: confirmed 2026-10-20.
- Event storming and term check: confirmed 2026-10-20.
- Use-case cut: confirmed 2026-10-20.
- Use cases: elaborated; phase review applied. Residual contract polish is
  recorded for the proposal and design steps.
- Personas: confirmed; all seven stay topic-local. Promotion remains offered
  per file but was not performed.
- Terms: 18 confirmed and added to the durable glossary
  (`docs/arc42/12-glossary.md`); no rejected terms.
- Seeds: ready; none started. Handoff confirmed 2026-10-20.
