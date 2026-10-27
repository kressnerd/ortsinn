# 12. Glossary

<!-- Domain and technical terms used in this project. Entries are alphabetically
     ordered. Use the same terms consistently throughout all documentation. -->

## Ubiquitous language

| Term | Definition | Contrast / anti-term |
|---|---|---|
| Benchmark | The reproducible measurement of pinned candidate tools against the acceptance criteria on fixed inputs, published as one report. | Product use case, not benchmark: the benchmark is a project gate outside product runtime; it changes no product behavior. |
| Candidate | An external code-knowledge tool measured through its own documented interface at a pinned version and commit. The fixture harness uses committed fixture candidates; the real comparison uses the external candidates. | Dependency, not candidate: a candidate is measured and never linked into the product. Fixture candidate, not candidate: a fixture candidate is a committed test double. |
| Capability gap | A candidate's missing operation or support level, recorded with a reason (for example `no_explanation_operation` or CKB Rust Tier 3) and excluded from scoring. | Zero score, not capability gap: a gap states that a capability is absent; a zero would claim it was measured and failed. |
| Cassette | A recorded JSONL file of normalized candidate results or raw model requests/responses, keyed by a canonical SHA-256 fingerprint. | Live run, not cassette: a cassette is evidence from one recorded run; a live run calls the candidate and the endpoint. |
| Chunk | A retrievable text segment bounded by a symbol or a size limit. | Symbol, not chunk: a symbol is a named code entity; a chunk is a retrieval unit that may span or split symbols. |
| Code coverage | The instrumented-line coverage produced by `cargo-llvm-cov` for one scope and merged into a Cobertura report. | Metric/task coverage, not code coverage: metric/task coverage counts scored tasks; code coverage counts executed lines. |
| Concept | A unit of meaning that several terms and identifiers may express (for example user, visitor, Benutzer, Besucher). | Term, not concept: a term is one spelling; a concept is the meaning several terms share. |
| Concept equivalence | The measured property that different surface variants expressing one concept retrieve the same expected target. | Term equality, not concept equivalence: identical spelling is not required; equivalence is judged by the retrieved target. |
| Concept vocabulary | The consistency boundary that maps each accepted term to at most one concept and records the evidence for each mapping. | Glossary, not concept vocabulary: the glossary is the persistent read model; the vocabulary enforces the mappings. |
| Evidence | The file and line citation that backs an answer. | Explanation, not evidence: an explanation interprets; evidence is checkable in the repository. |
| File resolution | A candidate answer that identifies a file without a line range, preserved as a file-level result. | Symbol resolution, not file resolution: a symbol resolution names a line range. |
| Fixed input | A pinned element of the comparison: repository and revision, synthetic seed and generator version, gold version and fingerprint, candidate version and commit, and the manifest itself. | Mutable input, not fixed input: an unpinned repository or version makes two runs incomparable. |
| Fixture candidate | A committed test double with a pinned identity (`fixture-ocbi`, `fixture-ckb`, `fixture-serena`) whose normalized results prove the report contract; it starts no external tool. | Candidate, not fixture candidate: a candidate is an external tool measured through its own interface. |
| Freshness | A measure of how current the index is relative to the repositories. | Staleness, not freshness: stale is the boolean observation that a level lags. |
| Glossary | The persistent read model of the concept vocabulary: each concept with its accepted terms, languages, identifiers, and evidence. | Concept vocabulary, not glossary: the vocabulary is the consistency boundary; the glossary is its read model. |
| Gold set | The reviewed, versioned expected answers for one metric and repository, with a task count meeting the minimum for the repository's power and a content fingerprint. A manifest-referenced set without a reviewer is rejected; an unreferenced draft is never loaded. | Proposal, not gold set: an unreviewed draft or model-generated task is not an expected answer. |
| Impact | The transitive set of dependents affected by a change to a symbol. | Dependency, not impact: a dependency is one edge; impact is the reachable set under a direction. |
| Index | The derived structure that answers queries over a workspace. | Workspace, not index: the workspace is the source set; the index is the rebuildable projection over it. |
| Index deadline | The manifest time limit for a candidate's index of one repository; exceeding it marks every metric for that cell `unavailable` with `index_timeout` and keeps the elapsed time. | Index duration, not index deadline: the duration is measured; the deadline decides availability. |
| Indexed file | The consistency boundary that keeps every symbol, chunk, and edge derived from one file tied to that file's current content hash. | Index, not indexed file: the index is the workspace-wide read model; the indexed file is the per-file consistency boundary. |
| Measured gap | The signed difference between a measured value and its target (`value - target`), carrying a status and, for AC-01, `target_status: proposed`. | Zero, not measured gap: an unavailable or unmeasured cell has no gap value. |
| Metric/task coverage | The share of gold tasks that produced a measurement: attempted divided by attempted plus unavailable, excluding not-applicable cells. | Code coverage, not metric/task coverage: code coverage counts instrumented lines; metric/task coverage counts scored tasks. |
| Naming inconsistency | The state in which one concept is expressed by several terms without a declared mapping. | Synonymy, not inconsistency: synonyms are acceptable when the glossary declares them; inconsistency is undeclared variation. |
| No-expansion arm | The same concept queries repeated with query expansion disabled, reported separately from the default arm. | Default arm, not no-expansion arm: the default arm uses the candidate's normal query expansion. |
| Path decision | The deferred build/contribute/compose choice: the report publishes measured gaps, names `maintainer--ortsinn` and `docs/decisions/benchmark-path.md`, and chooses no path. | Build threshold, not path decision: a threshold would pre-register the outcome; the path decision is made after the report. |
| Power label | The declared measurement power of a reference repository: `primary` (full published gold minimums) or `secondary` (half minimums, rendered `lower_power` and excluded from cross-repository aggregate claims). | Repository, not power label: the label states how much evidence the repository carries, not which source tree it is. |
| Precision level | The stated basis of a navigation answer: precise (type-resolved) or syntactic (name-matched). | Confidence, not precision level: confidence is a score; precision level names the evidence source. |
| Provenance record | The committed `benchmark/evidence/real-run/provenance.json` that binds a real run's cassettes and report to its command, UTC times, host/toolchain, endpoint localities and models, explanation budget, manifest pins, index outcomes, targets fingerprint, and cassette digests. | Run metadata, not provenance record: run metadata is excluded from measurement identity; the provenance record is the auditable evidence chain. |
| Publication floor | The minimum evidence a real report may publish: one measured candidate cell for typo, natural-language, explanation, and concept, and one measured cell for every manifest repository; otherwise exit 2 and write neither report file. | Coverage, not publication floor: coverage measures attempted tasks; the floor decides whether a report may exist. |
| Query digest | The SHA-256 of a model request's normalized query text, included in the model-cassette key and the result cassette's `model_requests` keys so a changed query is a missing recorded response rather than a reused answer. | Recording fingerprint, not query digest: the fingerprint identifies the cassette file; the digest identifies one request within it. |
| Query error | A query that returns an error for one candidate and repository; it counts as a miss in the metric and is reported in a separate error total. | Unavailable task, not query error: an unavailable task was never attempted; an errored query was attempted and failed. |
| Query expansion | The act of deriving additional search terms — synonyms, translations, identifier forms — from a query before retrieval. | Stemming, not expansion: stemming normalizes one word form; expansion adds different words. |
| Recording boundary | The loopback OpenAI-compatible proxy through which every candidate model request passes; it records in record mode and serves recordings in replay. | Provider endpoint, not recording boundary: the boundary decides what is recorded; the provider only answers. |
| Reference | An occurrence of a symbol at a location, either type-resolved or name-matched. | Definition, not reference: the definition introduces the symbol; a reference points at it. |
| Replay | A run that reads recorded cassettes, starts no candidate, and makes no external model call. | Record, not replay: record mode starts candidates and forwards model calls to the local endpoint. |
| Repository | One of the independent directory trees in a workspace. | Workspace, not repository: a workspace is the set the index spans; a repository is one member. |
| Setup deviation | A documented difference between an installed candidate's setup and its full documented setup, named on every affected measurement so a degraded cell is interpretable. | Missing capability, not setup deviation: a capability is absent; a deviation is a known reduced setup. |
| Setup lock | The committed `benchmark/setup.lock.json` recording probe results, resolved toolchain versions, setup deviations, and endpoint/model records for a run; it never writes or re-resolves a candidate pin. | Manifest, not setup lock: the manifest is the pin origin; the lock records what was probed and installed. |
| Setup probe | The recorded check that a candidate's documented interface can be started and can answer its declared operations. | Setup, not setup probe: setup installs the candidate; the probe records whether it is usable. |
| Summary | An LLM-generated explanation of a symbol, file, module, or workspace, cached against a content hash and prompt version. | Evidence, not summary: evidence is a checkable location; a summary is an interpretation. |
| Symbol | A named code entity (function, type, constant, field) with a definition location and a kind. | Chunk, not symbol: a chunk is a retrievable text segment. |
| Target | A metric's reference value with an explicit status: `confirmed`, `proposed`, or `unstated`, plus a source citation. AC-01's 0.90 is proposed; AC-03/12/21/22 are unstated. | Threshold, not target: a threshold decides a path; a target only labels a gap. |
| Targets file | The committed `benchmark/targets.yaml` holding the real targets; the manifest references it, its SHA-256 is reported, and an explicit `--targets` path takes precedence over it. | Target, not targets file: a target is one `{value, status, source}`; the targets file is their committed container. |
| Term | One spelling used in a repository for a concept, in any language or identifier form. | Concept, not term: a concept is the meaning; a term is one spelling of it. |
| Workspace | The set of independent repositories that one index covers. | Repository, not workspace: a repository is one source tree; a workspace is the set the index spans. |

## Aggregates and boundaries

| Aggregate | Invariant | Commands | Events |
|---|---|---|---|
| Indexed file | Every symbol, chunk, and edge derived from a file carries that file's current content hash; no entry points outside the file's line range; removing the file removes all its entries. | Index file; Remove file. | File indexed; File index failed; File removed from index. |
| Concept vocabulary | Each accepted term maps to at most one concept; every mapping cites evidence; merging preserves all terms. | Declare concept; Map term; Merge concepts; Reject term. | Concept declared; Term mapped; Concepts merged; Term rejected. |

The workspace is a configuration, not an aggregate. The index, symbol table,
text index, vector index, dependency graph, precise index, freshness status,
error list, glossary, and summary cache are read models or records. The
repository, model endpoint, and precise indexer are external systems.

## Invariants

- A file's derived entries share one content hash; a partial file update is
  never published.
- A term maps to at most one concept; an accepted mapping cites evidence;
  merging two concepts preserves all terms.
- Observing use cases never change the repository or the index.
- Removed or newly ignored content is purged before the index is marked fresh.
- External transfer requires both offline mode off and endpoint permission.
- A returned explanation carries at least one citation that resolves and is
  relevant to the target.
- A gold task is scored only from a manifest-referenced gold set with a
  reviewer and a count meeting its repository's power minimum; an unreferenced
  draft is never loaded.
- A changed query never reuses another query's recorded model response.
- A real report is written only above the per-family publication floor;
  otherwise the run exits 2 and writes neither report file.
- Candidate pins originate in the manifest; the setup lock records probes and
  toolchain versions, and the provenance record binds the committed evidence to
  the manifest pins.

## Lifecycle events

Workspace configured; Repository added; Ignore rule set; File change observed;
File indexed; File index failed; File removed from index; Embeddings produced;
Dependency edges updated; Precise index loaded; Index freshness changed; Index
rebuilt; Concept candidates proposed; Concept declared; Term mapped; Concepts
merged; Term rejected; Naming inconsistency detected; Question asked; Query
expanded; Candidates retrieved; Results fused; Answer delivered; Evidence
cited; Summary generated; Summary invalidated; Impact requested; Impact
assessed; Candidate setup probed; Setup deviation recorded; Gold draft
proposed; Gold set reviewed; Publication floor checked; Real run recorded;
Provenance recorded; Real run replayed.
