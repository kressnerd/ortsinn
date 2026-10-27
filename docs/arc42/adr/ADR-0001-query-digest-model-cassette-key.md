---
status: accepted
date: 2026-10-27
decision-makers: maintainer--ortsinn
consulted: none
informed: none
---

# Query digest in the model-cassette key

## Qualification

ADR-QUALIFICATION: PASS

### Qualification verdict

```text
qualification: QUALIFIED
alternatives: PASS
architectural_impact: PASS
durable_consequence: PASS
lower_level_artifact: PASS
existing_adr_check: PASS
must_be_shared: SUPPORTING
routing: NEW_ADR
canonical_artifact: null
target_adr: null
replacement_adr: null
```

### MADR checklist evidence

- [x] **Hard to reverse** — Reverting later would cost significant effort
      because: the committed result and model cassettes, their manifest
      fingerprints, and the report's reproduction identity would all have to be
      migrated back; the digested keys are part of the committed evidence, not
      a local cache.
- [x] **Shapes quality attributes** — It materially affects reproducibility and
      evidence honesty because: replay either returns the response recorded for
      that exact query or fails as a missing response; it can never silently
      return another query's answer.
- [x] **Structural** — It defines a boundary, contract, or dependency direction
      because: it fixes the durable on-disk model-cassette key format shared by
      the recorder, replay, the result cassette's `model_requests`, and every
      future gold version.
- [x] **Must be shared** — Developers unfamiliar with it would systematically
      violate the architecture because: adding a gold query edit without
      re-recording would look correct in a diff while replay silently reused a
      stale answer under the old query-free key.

The four checklist items are evidence prompts, not independent qualification
criteria. All five mandatory dimensions pass; `must_be_shared` is supporting
only.

## Context and Problem Statement

The benchmark records raw model requests and responses in a JSONL model
cassette and normalized candidate results in a result cassette, then replays
the cassettes to score without any model call. Before this decision the model
key identified the candidate, task or model, and layer, but not the query text.
Two gold versions that changed a query's wording for the same candidate and
task therefore collided: replay reused the earlier query's response and the
report looked correct. The cassette format is durable evidence, so the fix is a
format decision rather than a local implementation detail.

## Decision Drivers

- A replay must never reuse another query's recorded answer.
- Committed cassettes and their fingerprints must be independently verifiable.
- A gold-set revision must be safe without invalidating unrelated responses.
- The migration must be one bounded step with a verifiable result.

## Considered Options

- Option 1: keep the query-free key and document the limitation.
- Option 2: include the SHA-256 digest of the normalized query text in every
  model key.
- Option 3: include a digest of the entire gold set in the key.

## Decision Outcome

Chosen option: "Option 2", because it makes a changed query a distinct model
request and therefore a typed replay miss, with the smallest possible
invalidation unit.

### Consequences

- Good, because a gold query edit can no longer reuse a stale recorded answer;
  the failure names the missing response and the candidate.
- Good, because the cassettes are self-describing evidence: the query digest is
  part of the key and can be recomputed from the gold set.
- Bad, because every existing result and model cassette must be migrated once.
  The result cassette is a record-path input, so its `model_requests` keys need
  a one-shot migration and a recomputed fingerprint; the manifest and report
  fingerprints change.
- Bad, because changing a gold query now requires re-recording its explanation
  response instead of reusing the previous run.

### Confirmation

The scenario "A changed query does not reuse a recorded model response" edits
one explanation query, recomputes the gold fingerprint in a temporary manifest,
replays, and asserts exit 2 with the typed missing response for the new digest
and no reuse of the old key. The `@givn.modified` replay-fingerprint scenario
asserts that every committed model-cassette key carries the query digest and
that the recomputed file digest equals the manifest-declared fingerprint; the
replay model-dependency check enforces the same invariant on every run.

## Pros and Cons of the Options

### Option 1: keep the query-free key

- Good, because no cassette migration is required.
- Bad, because a changed query silently reuses another query's answer, which is
  fabricated evidence.
- Bad, because the limitation would have to be carried indefinitely and would
  undermine the real report's replay claim.

### Option 2: digest the normalized query

- Good, because a changed query is a distinct key and a typed replay failure.
- Good, because the invalidation unit is exactly the edited query.
- Neutral, because the key grows by one SHA-256 segment.
- Bad, because it requires a one-time migration of committed cassettes and
  their fingerprints.

### Option 3: digest the whole gold set

- Good, because any gold change is detected.
- Bad, because editing one task invalidates every recorded response, making
  incremental gold review expensive and replay coverage fragile.

## More Information

- Design: `givn/changes/benchmark-run/design.md` — "Recording, replay, and the
  query digest" and "Fixture migration".
- Prior limitation: the `benchmark-gate` design (F-11) documented the
  query-free key as a limitation that `benchmark-run` must remove before reuse
  across gold versions.
- Related chapters: Arc42 chapters 6 (runtime), 8 (cross-cutting concepts), and
  11 (cassette migration risk R-015).
