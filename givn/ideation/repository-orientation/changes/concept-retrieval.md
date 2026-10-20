# Change seed: concept-retrieval

Status: ready

- Use-case IDs: UC4, UC5, UC10, UC6 (meaning-level and hybrid), UC8.3
- Topic: `repository-orientation`
- Personas: agent--coding-harness, developer--agent-first,
  provider--model-endpoint
- Suggested start: 4; depends on `query-foundation`. The glossary curation
  (UC4/UC5) can start in parallel with `query-foundation` after
  `workspace-index`.
- Handoff: ready.

## Accepted decisions

### Meaning-level retrieval is a requirement, not a typo variant

- Context: the user stated that semantic search must return results for
  semantically close words of the same concept, not only typo variants.
- Chosen: retrieval by meaning across synonyms, translations, and naming
  variants; typo tolerance remains a separate match basis.
- The result states the match basis and the expanded terms with their sources:
  original, glossary, or model. Model variants are never presented as
  glossary terms.

### Project glossary

- The glossary is the read model of the concept vocabulary; it maps each
  accepted term to one concept with evidence.
- A term maps to at most one concept; merging preserves terms; rejected terms
  are suppressed until re-proposed with new evidence.
- Bootstrap proposes candidates; the developer confirms them. Bootstrap makes
  one external attempt within a deadline.

### External transfer and offline

- External transfer requires `offline: false` and
  `model_endpoint.permitted: true`.
- Offline issues no request; glossary and cached results still serve.
- One expansion or explanation generation makes one attempt within the
  deadline, with no automatic retry; failures carry a reason and retryability.
- Explanation cache keys include the target content digest, a context digest
  (glossary, docs, index revisions), prompt version, and model identity.

## Open questions

### Q6 — Owner visibility into model calls and resource consumption

The audit and retention choice is costly to reverse. Options are in
`givn/ideation/repository-orientation/questions.md` (Q6).

### Embedding model

Which model is used for meaning-level retrieval, and does it handle German and
English identifiers well enough for AC-21? The benchmark must measure, not
assume.

### Glossary curation burden

How much developer confirmation does bootstrap require before the glossary is
useful, and what is the fallback when no glossary exists?

### Explanation evidence threshold

How many valid, target-relevant citations are required before an explanation
is returned? The use case requires at least one inside the target; the
benchmark's AC-12 may justify a stronger rule.
