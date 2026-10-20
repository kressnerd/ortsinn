# Session state — repository orientation

- Topic: `repository-orientation`
- Updated: 2026-10-20
- Current phase: Handoff complete (user confirmed 2026-10-20)
- Next: the user starts a change with `/givn-propose` per seed. Q5 (benchmark
  decision rule) is decided before the benchmark seed starts; Q6 (owner
  visibility) before the workspace-index seed starts; Q3 (name availability)
  before the first release.

## Progress

- Inventory complete: the project has no changes and no specs; every arc42
  chapter is a template; the durable glossary is empty; `README.md` carries
  only the name and tagline.
- The six user attachments (design specification, storage decision, five
  scenarios, component map, name selection, market survey) are recorded as
  proposal material, not as confirmed decisions.
- `frame.md` drafted with problem, vision, stakeholders, scope, research
  protocol, and gaps.
- `questions.md` opened with four questions: Q1 human query surface,
  Q2 build/contribute/compose, Q3 name availability, Q4 repositories per
  index.
- `domain-terms.md` holds candidate terms only; none is confirmed.
- Frame quality review run; findings applied. Q2 is the frame-level fork and
  is asked before Q1. The project licence is MPL-2.0, not MIT/Apache-2.0.
- Q2 closed 2026-10-20: benchmark first. The first deliverable is the
  comparative benchmark; build, contribute, and compose remain open until its
  report. Q5 records the missing decision rule.
- Semantic requirement recorded 2026-10-20: retrieval must return targets that
  name the same thing in different words (synonyms, translations, naming
  variants), not only typo-close matches. Query expansion, project glossary,
  and chunk enrichment are proposal material. AC-21 and AC-22 are proposed,
  not confirmed.
- New candidate terms recorded: concept, concept equivalence, project
  glossary, query expansion, naming inconsistency. None is confirmed.
- Q1 closed 2026-10-20: MCP is the only query surface; the CLI stays
  administrative. The "not only MCP" statement is a future direction, not a
  first-deliverable commitment.
- Q4 closed 2026-10-20: one index spans several independent repositories;
  cross-repository references and impact are in scope. Workspace is recorded
  as a candidate term for the set of repositories.

- Frame confirmed 2026-10-20 by the user as the basis for Event Storming.
- Persona inventory: no project-level personas, no sibling ideation topics.
  Seven personas proposed, one per stakeholder type; the upstream maintainer
  is conditional on the benchmark leading to contribution.

- Personas confirmed 2026-10-20 by the user. All seven are review lenses for
  the topic; the upstream maintainer remains conditional on the benchmark.

- Event Storming drafted 2026-10-20: 32 events across workspace, indexing,
  vocabulary, query, summary, and impact; two aggregates (Indexed file,
  Concept vocabulary). The phase review applied: answer assembly policy added,
  impact event added, E10/E11/E12 routing fixed, inconsistency and evidence
  commands added to the canvas.
- Event Storming term check closed 2026-10-20: all 18 terms confirmed by the
  user, including the two aggregate roots. The generic-noun concern for
  `concept` was raised and the name confirmed.

- Use Case Definition cut drafted 2026-10-20: 15 `!` use cases (3 changing,
  12 observing) and 2 `+` summary use cases, covering all 32 events.
- Phase review applied: UC4 conjunction split, UC7/UC8 parent guarantees
  aligned with Includes, UC11 index-level split, UC9/UC11 preconditions fixed,
  E26/E28/E32 boundaries corrected, triggers rewritten as requests/events,
  "Agent for the Developer" notation applied, UC8.3 kept observing, persona
  tags reconciled.
- Persona value rankings recorded (agent, developer, repository owner,
  maintainer; reviewer also ranked contributor/provider/upstream).
- Q6 opened: owner visibility into external model calls and resource
  consumption.

- Use Case Definition cut confirmed 2026-10-20 by the user.
- Use Case Elaboration started with UC6 Locate code, the top value for the
  agent and developer personas. Elaboration order: UC6, UC7, UC8, UC9, UC10,
  UC11, UC1, UC2, UC4, UC5, UC3.

- UC6 elaborated 2026-10-20. Persona critiques applied: `repository_scope`
  (hard filter) split from `focus_path` (soft boost); match basis is a set
  (`exact-symbol`, `typo`, `lexical`, `synonym`, `translation`,
  `meaning-level`); default 4,000-token budget with a hard cap and atomic
  candidates; per-mode status (`ran`, `skipped`, `failed`, `unavailable`);
  expansion provenance (original, glossary, model); offline makes no external
  request; one expansion attempt at a deadline; query-embedding call named;
  data boundary limited to query text, language, and kind; candidate
  provenance, role, origin, and stale flag; determinism tied to index
  revision, glossary snapshot, and expansion content.

- UC6 confirmed 2026-10-20 by the user.
- UC7.1 drafted 2026-10-20: definition request by name or location, resolution
  basis (exact, typo, location), ambiguity candidates, precise lookup with
  syntactic fallback, multiple definitions, scope filter, one result envelope.

- UC7.1 confirmed 2026-10-20 by the user.
- UC7.2 drafted 2026-10-20: references by symbol identity, candidate
  identifier, or location; precise lookup with syntactic fallback; enclosing
  symbol, roles, test marker, cross-repository references, scope filters, one
  result envelope.

- UC7.2 confirmed 2026-10-20 by the user. The user directed the workflow to
  proceed through the remaining use cases without per-case confirmation,
  interrupting only for decisions costly to reverse.

- Batch 1 (UC7.3, UC7, UC8.1, UC8.2, UC8.3, UC8) reviewed by agent, developer,
  and provider personas; high/medium findings applied: identity inputs,
  one-level fallback, `resolution`/`freshness` shapes, counts, source
  revisions, explanation cache/context digests, data boundary, budgets.
- All remaining use cases drafted: UC1, UC2, UC3, UC4, UC5, UC9, UC10, UC11.
- Batch 2 review completed for UC1-UC5 and UC9-UC11 with agent, developer,
  owner, and provider personas.

- Batch 2 (UC1, UC2, UC3, UC4, UC5, UC9, UC10, UC11) reviewed by agent,
  developer, owner, and provider personas. High findings applied: config vs
  content revision, scope eviction, expected-revision conflicts, embedding
  batching/dedup/caps, external-transfer gating, job schema, failure
  vocabulary (`failures`, not `errors`), outstanding-failure scope, closed
  failure kinds, vocabulary status/envelope shapes, candidate provenance,
  rejection suppression, variant match bases and coverage, impact graph
  identity/counts/coverage matrix, glossary revision and caps,
  summary/entries freshness detail.
- Medium and low findings that only affect polish or later design detail are
  left for the phase review and the proposal/design steps.

- Handoff published 2026-10-20: `docs/ideation/repository-orientation.md`;
  five change seeds under `changes/` (benchmark-gate, workspace-index,
  query-foundation, concept-retrieval, impact-graph).
- 18 confirmed terms proposed for the durable glossary; no rejected terms.
- Seven personas stay topic-local; promotion is offered but not performed.
- Residual contract polish from the phase review is recorded for the
  proposal/design steps.

- Handoff confirmed 2026-10-20 by the user. The 18 terms are added to the
  durable glossary (`docs/arc42/12-glossary.md`); the seven personas stay
  topic-local; the five seeds are ready.

## Next question

None. Handoff is complete. The user starts a change with `/givn-propose` per
seed; Q5 and Q6 are decided before the seeds they affect.

## Phase ladder

Frame → Event Storming → Use Case Definition → Use Case Elaboration → Handoff.
No later phase started.
