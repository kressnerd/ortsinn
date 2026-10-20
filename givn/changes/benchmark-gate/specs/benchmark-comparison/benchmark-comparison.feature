@givn.delta @benchmark-comparison

Feature: Benchmark comparison

  The maintainer gets a reproducible, measured comparison of the nearest
  code-knowledge tools against the acceptance criteria on fixed repositories,
  so the product path can be chosen after the report rather than in advance.

  The scenarios below run against the committed fixture described in
  `design.md` ("Fixture contract"): repository `fixture-repo` at revision
  `fixture-1`, candidates `fixture-ocbi`, `fixture-ckb`, and
  `fixture-serena`, five typo tasks, four natural-language tasks, four
  explanation tasks, five concept tasks, and four concept-equivalence variants.
  Every Then-step asserts the expected fixture value, not the presence of a
  report section.

  @givn.added
  Scenario: Report identifies every candidate and fixed repository revision
    Given the fixture benchmark inputs
    When the maintainer runs the benchmark in replay mode
    Then the report names fixture-ocbi at version "fixture-1" and commit "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
    And the report names fixture-ckb at version "fixture-1" and commit "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
    And the report names fixture-serena at version "fixture-1" and commit "cccccccccccccccccccccccccccccccccccccccc"
    And the report names fixture-repo at revision "fixture-1", commit "1111111111111111111111111111111111111111", and 120 code lines

  @givn.added
  Scenario: Report contains the four acceptance measurements
    Given the fixture benchmark inputs
    When the maintainer runs the benchmark in replay mode
    Then the typo recall at rank one is 0.60 with 3 of 5 tasks
    And the typo recall at rank five is 0.80 with 4 of 5 tasks
    And the natural-language recall at rank five is 0.50 with 2 of 4 tasks
    And the natural-language mean reciprocal rank at rank ten is 0.30
    And the explanation correctness is 0.50 with 2 of 4 tasks
    And the explanation evidence grounding is 1.00 with 4 of 4 tasks
    And the concept recall at rank five is 0.40 with 2 of 5 tasks
    And the concept-equivalence recall at rank five is 0.50 with 2 of 4 variants

  @givn.added
  Scenario: Report states measured gaps without a pre-registered build threshold
    Given the fixture benchmark inputs
    When the maintainer runs the benchmark in replay mode
    Then the typo recall at rank one shows a gap of -0.30 to the target 0.90
    And the report labels the 0.90 target "proposed" and never "confirmed"
    And the natural-language recall at rank five shows its measured value 0.50 and the best measured value 0.50
    And the concept recall at rank five shows its measured value 0.40 and the best measured value 0.40
    And the report records the product-path decision as deferred
    And the report names the Ortsinn maintainer and the post-report decision record as the decision owner and artifact

  @givn.added @wip
  Scenario: Report compares concept recall with and without query expansion
    Given a candidate that supports disabling query expansion
    When the maintainer runs the benchmark in replay mode
    Then the concept recall for the default arm is 0.40 with 2 of 5 tasks
    And the concept recall for the no-expansion arm is 0.20 with 1 of 5 tasks
    And the concept-equivalence recall for the default arm is 0.50 with 2 of 4 variants
    And the concept-equivalence recall for the no-expansion arm is 0.25 with 1 of 4 variants
    And fixture-serena's no-expansion arm is not applicable with the reason "no_expansion_switch", not zero

  @givn.added @wip
  Scenario: Report distinguishes an unavailable capability from a zero score
    Given a candidate that cannot explain code
    When the maintainer runs the benchmark in replay mode
    Then the explanation correctness for fixture-ocbi is unavailable with the reason "no_explanation_operation"
    And the explanation coverage for fixture-ocbi is 0 of 4 with a value of 0.0
    And the aggregate explanation correctness remains 0.50 with 2 of 4 tasks
    And the report lists the missing explanation capability as a capability gap for fixture-ocbi

  @givn.added @wip
  Scenario: Replay rerun reproduces the report without an external model call
    Given a completed benchmark run with recorded model responses
    When the maintainer reruns the same fixed inputs in replay mode
    Then the measurements and their order match the first run
    And no request leaves the local recording boundary
    And the report states the replay mode and the recording fingerprint "sha256:fixture-model-v1"

  @givn.added @wip
  Scenario: A missing recorded response stops the run
    Given a benchmark run that needs a model response absent from the recording
    When the maintainer runs the benchmark in replay mode
    Then the run exits with code 2
    And the error names the missing input and the candidate that requested it
    And no report.json or report.md is published

  @givn.added @wip
  Scenario: The synthetic monorepo is regenerated to the same content fingerprint
    Given the fixed synthetic seed and generator version
    When the maintainer regenerates the synthetic monorepo
    Then it contains ten million code lines across the TypeScript, Python, and Rust languages
    And its content fingerprint matches the recorded fixed input "sha256:synthetic-demo-v1"
    And its gold sets name the same locations

  @givn.added @wip
  Scenario: Report separates wrong rank from no match in typo recall
    Given the fixture typo gold set of five tasks
    When the maintainer runs the benchmark in replay mode
    Then fixture-ocbi returns no result for 1 of the 5 typo tasks
    And the typo no-match rate is 0.20 with 1 of 5 tasks
    And the typo recall at rank five is 0.80 because 4 of 5 tasks are ranked

  @givn.added @wip
  Scenario: Report states metric and task coverage with denominators
    Given the fixture benchmark inputs
    When the maintainer runs the benchmark in replay mode
    Then the typo coverage is 5 of 5 with a value of 1.0
    And the natural-language coverage is 4 of 8 with a value of 0.5
    And the explanation coverage is 4 of 8 with a value of 0.5
    And the concept coverage is 5 of 5 with a value of 1.0
    And the concept-equivalence coverage is 4 of 4 with a value of 1.0
    And fixture-ckb's explanation coverage is 4 of 4 and fixture-ocbi's explanation coverage is 0 of 4
    And every coverage value is attempted divided by attempted plus unavailable, excluding not-applicable cells

  @givn.added @wip
  Scenario: Report names each gold set's version, task count, and fingerprint
    Given the fixture benchmark inputs
    When the maintainer runs the benchmark in replay mode
    Then the typo gold set is version 1 with 5 tasks and fingerprint "sha256:fixture-typo-v1"
    And the natural-language gold set is version 1 with 4 tasks and fingerprint "sha256:fixture-nl-v1"
    And the explanation gold set is version 1 with 4 tasks and fingerprint "sha256:fixture-explanation-v1"
    And the concept gold set is version 1 with 5 tasks and fingerprint "sha256:fixture-concept-v1"

  @givn.added @wip
  Scenario: Report states the exact command that reproduces the run
    Given a completed benchmark run with recorded model responses
    When the maintainer reads the report
    Then the report states the replay command for the fixture manifest with mode replay and its output path
    And the report states the toolchain environment prefix used by the command
    And the report states the manifest, gold, model-cassette, and result-cassette fingerprints

  @givn.added @wip
  Scenario: Report records candidate setup, index status, and timeouts
    Given the fixture benchmark inputs
    When the maintainer runs the benchmark in replay mode
    Then every candidate records its install command, version, commit, and adapter version
    And CKB records the pinned scip-typescript, scip-python, and rust-analyzer indexer versions and its doctor output
    And CKB records Rust language support as Tier 3 in its capability note
    And Serena records its Python 3.13 environment and the pyright, typescript-language-server, and rust-analyzer versions
    And open-codebase-index records embeddingProvider "custom" and the recorder base URL
    And every candidate copy records git_metadata "present"
    And every candidate records a per-repository index status and duration

  @givn.added @wip
  Scenario: A fixed-input mismatch stops the run
    Given a fixture manifest whose fixture-repo commit is "2222222222222222222222222222222222222222"
    When the maintainer runs the benchmark in replay mode
    Then the run exits with code 2
    And the error names fixture-repo and the mismatched expected and actual commits
    And no report.json or report.md is published

  @givn.added @wip
  Scenario: An uninstallable or unstartable candidate is recorded and the run continues
    Given fixture-unstartable whose MCP handshake fails
    When the maintainer runs the benchmark in replay mode
    Then the report marks fixture-unstartable unavailable with the reason "candidate_unstartable"
    And the run completes and reports measured values for fixture-ocbi and fixture-ckb
    And the report lists fixture-unstartable in the capability gaps

  @givn.added @wip
  Scenario: A candidate that exceeds its index deadline is unavailable, not zero
    Given fixture-timeout with an index deadline of 1 second on fixture-repo
    When the maintainer runs the benchmark in replay mode
    Then the report marks fixture-timeout unavailable on fixture-repo with the reason "index_timeout"
    And the report records the elapsed index time
    And every metric for fixture-timeout on fixture-repo is unavailable and never zero
    And the run continues with the other candidates

  @givn.added @wip
  Scenario: A query error counts as a miss and is reported separately
    Given a natural-language gold set of four tasks where 1 query returns an error
    When the maintainer runs the benchmark in replay mode
    Then the natural-language recall at rank five is 0.50 with 2 of 4 tasks
    And the report states a query error total of 1 separately from the recall value
    And the errored query is counted as a miss and not as an unavailable task

  @givn.added @wip
  Scenario: A metric with no measured candidate reports an unavailable gap
    Given a fixture run where every typo candidate is unavailable
    When the maintainer runs the benchmark in replay mode
    Then the typo to-best gap has status "unavailable" with the reason "no_measured_candidate"
    And the typo to-target gap has status "unavailable" with the reason "no_measured_candidate"
    And neither gap value is rendered as zero
    And the report is still published as complete

  @givn.added @wip
  Scenario: Report does not score a regex interface mismatch as natural-language recall
    Given fixture-serena exposes no natural-language search operation
    When the maintainer runs the benchmark in replay mode
    Then fixture-serena's natural-language cells are unavailable with the reason "no_natural_language_search"
    And the natural-language recall at rank five remains 0.50 with 2 of 4 tasks, excluding fixture-serena
    And a measured regex arm is labelled "regex_pattern_search_proxy" and excluded from natural-language recall and mean reciprocal rank

  @givn.added @e2e @wip
  Scenario: Maintainer runs the benchmark and reads the measured report
    Given a benchmark fixture with fixed repository revisions, pinned candidate identities, and recorded model responses
    When the maintainer runs "benchmark run" in the terminal
    Then the command exits with code 0
    And the terminal shows the four measurement sections and the report path
    And the report at that path states typo recall at rank one 0.60, natural-language recall at rank five 0.50, explanation correctness 0.50, and concept recall at rank five 0.40
    And the terminal renders every metric label and value on one line at width 80
    And the terminal renders every metric label and value on one line at width 160

  @givn.added @e2e @wip
  Scenario: Maintainer records a run and replays it without an external call
    Given a local fake OpenAI-compatible endpoint and the fixture candidates
    When the maintainer records the benchmark run against that endpoint
    Then the model requests are recorded and the model cassette fingerprint is written
    And the normalized results are recorded
    When the maintainer replays the recorded run
    Then the measurements and their order match the recorded run
    And no request leaves the local recording boundary
