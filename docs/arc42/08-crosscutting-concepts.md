# 8. Cross-cutting Concepts

## Recording boundary and replay

All model traffic crosses one loopback OpenAI-compatible proxy. This change's
record scenario points it at a deterministic fake endpoint and writes raw
request/response pairs; replay serves only recorded responses. Normalized
result cassettes are a second layer, so a rerun does not depend on a candidate
process. `benchmark-run` records real candidate results into the same shape.
Replay starts no candidate and makes no external call. A missing key is a
typed `MissingRecordedResponse` and a hard failure (exit 2), never a live
fallback or a partial report.

## Gold validation against the fixed checkout

`Gold::load` requires each repository's `checkout` and validates every expected
location before scoring: the path is relative and stays inside the checkout,
the file exists, the range is ordered, and the end line is inside the file.
Each explanation target and acceptable-evidence range is validated the same
way. A failed check stops the run; no score is computed against a stale or
missing expected answer.

## Canonical keys and fingerprints

Cassette keys are SHA-256 over a canonical JSON object: sorted keys, normalized
fields, and no request headers, request IDs, timestamps, or transport metadata.
Model response IDs are replaced with a stable value when recorded. The report
carries the manifest, gold, model-cassette, and result-cassette fingerprints,
plus the exact reproduction command. Timestamps and run metadata are excluded
from measurement identity.

## Closed status model

Every metric cell and every gap is one of:

- `measured`: a value with numerator and denominator (or attempted tasks);
- `unavailable`: a reason such as `no_explanation_operation`,
  `no_natural_language_search`, `candidate_unstartable`, or `index_timeout`;
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
- `2`: a fixed-input mismatch or a missing replay key; no complete report is
  published.
- Other non-zero: an internal or usage error; the CLI names the failure.

Candidate-level failures (install, start, index timeout, missing operation,
query error) do not stop the run; they become unavailable cells or a separate
error total. A per-query error counts as a miss and is reported separately, and
there are no automatic retries.

## Targets and gap labeling

A target is `{value, status, source}` with status `confirmed`, `proposed`, or
`unstated`. AC-01's 0.90 is `proposed` (proposal attachments 1/7, not
confirmed); AC-03/12/21/22 are `unstated`. A proposed target's gap is labelled
"gap to proposed target" and can never be read as an accepted threshold. The
report derives no build threshold and contains no path choice.

## Observability and evidence

The run records each candidate identity, install command, adapter version,
setup fields (indexers, doctor output, language-support tiers, environment
versions, embedding-provider configuration, `git_metadata` presence),
per-repository index status and duration, and every recorded deviation. The
fixture records fixture values for those fields; `benchmark-run` records the
real values. Code coverage proves both the runner production paths and the CLI
child path through two separate instrumented scopes and a merged Cobertura
report; visual transcripts are committed as review evidence.

## Fixture candidate versus candidate

A **candidate** is an external code-knowledge tool measured through its own
documented interface at a pinned version and commit. A **fixture candidate** is
a committed test double with a pinned identity that records normalized results
in the same shape. The report's `candidates` list contains whatever the
manifest declares; in this change those are fixture candidates, and the report
and specs never present them as external tools. Code coverage is a third,
separate concept from metric/task coverage.
