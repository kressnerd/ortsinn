# Frame — repository orientation

- Topic: `repository-orientation`
- Status: confirmed 2026-10-20 by the user
- Source material: user input of 2026-10-19 plus seven attachments (design
  specification, storage decision, five scenarios, component map, name
  selection, market survey, semantic concept analysis of 2026-10-20)

## Problem

A developer or coding agent working in a large, multi-language repository
cannot reliably establish context — locating the right symbol despite typos,
paraphrases, translations, or inconsistent naming, proving who uses it, and
judging what a change breaks — because no local source answers location,
relation, and meaning together with verifiable evidence.

## Vision

Proposed vision, derived from the source material and conditional on Q2:

Ortsinn gives its consumer a sense of place in a repository: it answers where a
thing is, how it relates, what it does, and what depends on it, from an index
that stays fresh as files change. Answers are grounded in the repository, so
the consumer can check them instead of trusting them. It treats different
words that name the same thing — synonyms, translations, naming variants — as
one meaning, not as separate vocabulary. It runs locally and is open source.

The source material adds concrete mechanisms: Rust, an explicit precision
level and file:line evidence in every answer, a bounded number of tool calls,
typo-tolerant symbol search, concept-level retrieval, query expansion, and a
project glossary. Those are requirements to confirm in later phases, not frame
facts.

| Whose work changes | Today | With Ortsinn |
|---|---|---|
| Coding agent | Reads whole files to find a location; guesses at call sites; loses context to large outputs | A few bounded tool calls to a location it can verify |
| Developer operating the agent | Cannot tell whether the agent's answer is grounded | Reads the same evidence the agent used; can query directly (surface open: Q1) |
| Repository owner | Code leaves the machine only if an external model is configured | Can keep the index and queries local; external model calls are optional |

## Stakeholders

| Stakeholder type | Interest in the outcome |
|---|---|
| Coding agent (primary consumer) | Compact, unambiguous, machine-readable answers; stable tool contracts; pagination and token bounds |
| Developer operating an agent harness | Correct location with few calls; trust through evidence; no surprise resource use |
| Repository owner / team | Fresh index after saves; no disruptive writes; control over what leaves the machine |
| Open-source contributor / adapter implementer | Clear ports, conformance suite, language-coverage path |
| Ortsinn maintainer | Adoption in a crowded field; sustainable maintenance; differentiation that survives technology change |
| Upstream project maintainer (only if Q2 chooses contribution) | Accepts, reviews, and maintains the contributed capability; sets direction and licence |
| Model endpoint operator (external) | Receives code when embeddings or summaries are requested; rate limits and cost apply |

Q1 may add a direct human user (CLI user or editor user) as a stakeholder type;
the list is extended when Q1 closes. The primary consumer is the coding agent
under every option considered so far.

## Scope

In scope for this topic:

- A local knowledge layer over a set of independent repositories indexed
  together, multi-language, incrementally updated. Cross-repository references
  and impact are in scope (Q4 closed).
- Read-only queries: locate, navigate, understand, assess impact.
- Retrieval by meaning: a query returns targets that express the same thing in
  different words (synonyms, translations, naming variants), not only literal
  or typo-close matches.
- A project-level record of accepted terms per meaning, so the consumer can see
  the repository's vocabulary and query with it.
- Agent consumption over MCP as the only query surface; the CLI stays
  administrative (Q1 closed).
- Evidence-bearing answers with an explicit precision level.

Out of scope for this topic:

- A hosted or multi-tenant service.
- Code editing, refactoring, or execution.
- Replacing an editor or an LSP server.
- Guaranteed correctness of LLM-generated explanations without evidence.

Open scope forks:

- The benchmark outcome between build, contribution, and composition (Q2) —
  determines whether this topic is a new product at all; the vision and
  stakeholder list are conditional on it. No other Frame fork is open.

## Research protocol

| Source | What it contributes | Status |
|---|---|---|
| `README.md` | Name and tagline: "A sense of place for code agents." | present |
| `docs/arc42/` | Durable architecture record | templates only; no decisions recorded |
| `docs/arc42/12-glossary.md` | Durable ubiquitous language | empty |
| `givn/` | Changes and specs | no changes, no specs |
| `LICENSE` | Project licence: Mozilla Public License 2.0 | present |
| User attachment 1 | Requirements F1–F9, N1–N6, architecture, algorithms, AC-01–AC-20 | proposal, not confirmed |
| User attachment 2 | Storage decision: SQLite + tantivy + usearch, no graph database | proposal, not confirmed |
| User attachment 3 | Five usage scenarios | proposal, not confirmed |
| User attachment 4 | Component-to-question mapping | proposal, not confirmed |
| User attachment 5 | Name selection: Ortsinn | proposal; availability unverified |
| User attachment 6 | Market survey and build/contribute/compose options | proposal; README claims not re-verified |
| User attachment 7 | Semantic concept analysis: embeddings, query expansion, project glossary, chunk enrichment, AC-21, AC-22 | proposal, not confirmed |

Gaps this frame does not close:

- No personas.
- No event storming, use cases, or confirmed domain terms.
- No benchmark evidence for the acceptance thresholds in attachments 1 and 7.
- No decision rule for the benchmark (Q5).
- No verified name availability.
- The term for the meaning shared by different words is a candidate; it is not
  confirmed.

## Derived constraints (from the material, not yet confirmed as decisions)

- Project licence is MPL-2.0, as shipped in `LICENSE`. The source material
  proposes MIT or Apache-2.0 for dependencies, not for the project itself.
- Rust; local developer machine; open source.
- MCP consumer over stdio and Streamable HTTP.
- External OpenAI-compatible model endpoint; offline mode.
- Tree-sitter for breadth and immediacy; SCIP for precise navigation where an
  indexer exists.
- SQLite, tantivy, and usearch as embedded stores; no separate database
  service.
- Incremental update on file change; syntactic freshness within 2 s, semantic
  within 30 s.

These constrain later design; they do not define the domain.

## Confirmation record

- 2026-10-19 — frame drafted.
- 2026-10-19 — Frame review applied: Q2 (build/contribute/compose) is the
  frame-level fork and is asked before Q1; the vision and the upstream
  maintainer entry are conditional on Q2; Q4 (repositories per index) added;
  the project licence corrected to MPL-2.0.
- 2026-10-20 — Q2 resolved: benchmark first. The first deliverable is the
  comparative benchmark, not the product. Build, contribute, and compose
  remain open until the benchmark report; the vision and the upstream
  maintainer entry stay conditional on that result.
- 2026-10-20 — the semantic requirement was added: retrieval must return
  targets that name the same thing in different words, not only typo-close
  matches. Query expansion, project glossary, and chunk enrichment are
  recorded as proposal material, not as confirmed design.
- 2026-10-20 — Q1 closed: the only query surface is MCP; the CLI stays
  administrative. The "not only MCP" statement is a future direction, not a
  first-deliverable commitment.
- 2026-10-20 — Q4 closed: one index spans several independent repositories;
  cross-repository references and impact are in scope.

- 2026-10-20 — Frame confirmed by the user as the basis for Event Storming.
  The product path remains conditional on the benchmark; Q5 (benchmark
  decision rule) is logged for its seed.
