# 8. Cross-cutting Concepts

## Recording boundary and replay

All benchmark model traffic crosses one recording proxy. Embedding requests go
to a loopback OpenAI-compatible endpoint; the at-most-80 reviewed gold
explanation queries go to an external OpenAI-compatible endpoint under a
recorded operator decision. Candidate-internal embedding calls stay inside the
candidate's own interface on loopback and are recorded in its setup, not as
benchmark model dependencies. The record scenario points the proxy at a
deterministic fake endpoint and writes raw request/response pairs; replay
serves only recorded responses. Normalized result cassettes are a second layer,
so a rerun does not depend on a candidate process. Replay starts no candidate
and makes no external call. A missing key is a typed `MissingRecordedResponse`
and a hard failure (exit 2), never a live fallback or a partial report.

## Gold validation against the fixed checkout

`Gold::load` requires each repository's `checkout` and validates every expected
location before scoring: the path is relative and stays inside the checkout,
the file exists, the range is ordered, and the end line is inside the file.
Each explanation target and acceptable-evidence range is validated the same
way. A failed check stops the run; no score is computed against a stale or
missing expected answer.

## Canonical keys and fingerprints

Result entries are indexed by `(candidate, metric, task, arm)`; model keys include the SHA-256 digest of the normalized query. A changed query
is therefore a distinct key and a replay miss, never a reused answer. The
one-shot migration rewrites the committed result cassette's model keys to the
digest form and recomputes its fingerprint; the report keys gold fingerprints
by `<repository>/<metric>`, so a multi-repository run cannot collapse two
repositories into one entry. The cassette fingerprint is SHA-256 over the
file bytes. Replay verifies each recorded candidate version/commit, repository
commit, gold digest, and committed-cassette digest before scoring; a mismatch
exits 2 and writes no report. The report carries the manifest, targets, gold,
model-cassette, result-cassette, and provenance fingerprints, plus the exact
reproduction command. Timestamps and run metadata are excluded from measurement
identity.

## Setup probes and deviations

Each real candidate is probed through its documented interface before
indexing. A probe records whether the interface starts and which declared
operations it exposes. A candidate that cannot start is `unavailable` with its
reason; a documented dependency that cannot be installed is a **setup
deviation**, named on every affected cell. A deviation does not turn a
measured value into zero; a missing operation is a capability gap, not a
deviation. The setup lock records the probe results and resolved toolchain
versions, and the candidate pins originate in the committed manifest, so the
evidence run is reproducible without a circular pin source.

## Reviewed gold, power, and minimum counts

The real gold sets are reviewed expected answers, not model output. A draft
proposer writes unreviewed tasks under `benchmark/gold/drafts/`; only files
referenced by the manifest are loaded, and a referenced set must carry
`reviewed_by: maintainer--ortsinn`. A referenced set without a reviewer fails
before scoring with the typed `unreviewed gold set` error; a draft the manifest
does not reference is never loaded. The manifest declares `gold_minimums` and
each repository's `power`: the primary public repository uses 50 typo, 30
natural-language, 20 explanation, 50 concept, and 20 concept-equivalence tasks;
each secondary repository uses half (25/15/10/25/10). A below-minimum referenced
set fails with the typed `gold_below_minimum` error. The fixture manifest uses
`mode: fixture` and is not subject to the published counts. Secondary
repositories are labelled lower-power and excluded from cross-repository
aggregate claims. The synthetic monorepo's gold is generated and reviewed with
its generator.

## Publication floor

A real report publishes only when there is at least one measured candidate cell
for typo, natural-language, explanation, and concept, and at least one measured
cell for every manifest repository. The runner evaluates the floor before
writing either report file; an unmet floor exits 2, names the missing family
and repository, and writes neither `report.json` nor `report.md`. Unavailable
candidates remain in any published report; the floor prevents an
all-unavailable run from being presented as a measured comparison.

## Closed status model

Every metric cell and every gap is one of:

- `measured`: a value with numerator and denominator (or attempted tasks);
- `unavailable`: a reason such as `no_explanation_operation`,
  `no_natural_language_search`, `candidate_unstartable`,
  `setup_probe_failed`, or `index_timeout`; a setup deviation labels an
  affected measured cell without replacing its value;
- `not_applicable`: a reason such as `no_expansion_switch`.

A missing measurement is never rendered as zero. An aggregate with no measured
candidate has `to_best` and `to_target` status `unavailable` with
`no_measured_candidate`. Serena's NL cells are unavailable so a regex interface
mismatch is not scored as a low capability; a `regex_pattern_search_proxy`
value may be shown separately and is excluded from the NL aggregate.

## Metric/task coverage versus code coverage

**Metric/task coverage** is the share of gold tasks that produced a measurement:
`attempted / (attempted + unavailable)`, with `not_applicable` cells excluded
from both numerator and denominator. It is always reported with its numerator
and denominator. **Code coverage** is the instrumented-line coverage produced by
`cargo llvm-cov` in `measure-coverage.sh`/`merge-coverages.sh`. The two are
never abbreviated to "coverage" without qualification.

## Determinism and ordering

Gold tasks are ordered by the manifest's repository and task order; results are
deduplicated by `(path, start_line, end_line, symbol)` and capped at the
candidate's return order, which the harness never reorders. Report maps are
serialized with sorted keys. The synthetic repository and its gold are
regenerated from one seed and generator version and verified by fingerprint.

## Error handling and exit codes

- `0`: a complete report was written after all checks passed.
- `2`: a fixed-input mismatch, a missing replay key, an unreviewed or
  below-minimum referenced gold set, or an unmet publication floor; no complete
  report is published.
- Other non-zero: an internal or usage error; the CLI names the failure.

Candidate-level failures (install, setup probe, start, index timeout, missing
operation, query error) do not stop the run; they become unavailable cells, a
setup deviation, or a separate error total. A per-query error counts as a miss and is reported separately, and
there are no automatic retries.

## Targets and gap labeling

A target is `{value, status, source}` with status `confirmed`, `proposed`, or
`unstated`. The real targets live in the committed `benchmark/targets.yaml`,
referenced by the manifest as `targets_file`; the fixture keeps inline targets.
Loading precedence is an explicit `--targets` path, then `targets_file`, then
inline targets, and more than one source is an error. The targets file's
SHA-256 is reported and recorded in the provenance record. AC-01's 0.90 is
`proposed` (proposal attachments 1/7, not confirmed); AC-03/12/21/22 are
`unstated`. A proposed target's gap is labelled "gap to proposed target" and
can never be read as an accepted threshold. The report derives no build
threshold and contains no path choice.

## Observability and evidence

The run records each candidate identity, install command, adapter version,
setup fields (indexers, doctor output, language-support tiers, environment
versions, embedding-provider configuration, `git_metadata` presence),
per-repository index status and duration, and every recorded deviation. The
fixture records fixture values for those fields; the real run records the
probed versions, doctor output, setup deviations, and index outcomes from the
setup lock and adapters. The committed provenance record additionally binds
the evidence to the exact command, UTC start/end, host/toolchain, both endpoint
localities and models, the explanation-query budget and consumed count, the
manifest pins, and the cassette digests; scenarios cross-check it against the
setup lock, cassettes, and report. Code coverage proves both the runner
production paths and the CLI child path through two separate instrumented
scopes and a merged Cobertura report; visual transcripts are committed as
review evidence.

## Fixture candidate versus candidate

A **candidate** is an external code-knowledge tool measured through its own
documented interface at a pinned version and commit. A **fixture candidate** is
a committed test double with a pinned identity that records normalized results
in the same shape. The fixture harness's report contains fixture candidates;
the real comparison's report contains the external candidates. The report and
specs never present one as the other, and the identity fields name which.
Code coverage is a third, separate concept from metric/task coverage.
