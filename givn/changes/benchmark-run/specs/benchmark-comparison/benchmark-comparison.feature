@givn.delta @benchmark-comparison

Feature: Benchmark comparison

  @givn.added
  Scenario: Real manifest pins every reference repository and the synthetic monorepo
    Given  the real benchmark manifest
    Then  it pins typescript-reference, python-reference, and rust-reference at a URL, revision, commit, language, measured size, and power
    And  it pins the synthetic monorepo with its fixed seed, generator version, and content fingerprint
    And  it sets the public repository index deadline to 14400 seconds
    And  it sets the synthetic monorepo index deadline to 1800 seconds

  @givn.added
  Scenario: Real manifest pins every candidate version and commit
    Given  the real benchmark manifest
    Then  it pins open-codebase-index, CKB, and Serena at a candidate version and source commit
    And  each candidate pin records the evidence for its version and commit

  @givn.added
  Scenario: The real manifest and setup lock agree on every pin
    Given  the real benchmark manifest and the committed setup lock
    Then  every candidate and repository pin is identical in both files
    And  every pinned commit is a 40-character commit
    And  the setup lock records a probe result and the toolchain versions for every candidate
    And  the setup lock records no candidate pin as its origin
    And  the setup lock records both endpoint localities, both model names, and the explanation-query budget

  @givn.added
  Scenario: A real fixed input that differs from its pin stops before scoring
    Given  a real fixed input whose checkout or candidate installation does not match the manifest
    When  the benchmark validates the fixed inputs
    Then  the run exits with code 2 and names the mismatched fixed input
    And  no benchmark run starts

  @givn.added @wip
  Scenario: A documented setup deviation is recorded on every affected cell
    Given  a real candidate whose installed setup differs from its full documented setup
    When  the maintainer runs the real benchmark in record mode
    Then  the report names the candidate's setup deviation
    And  every cell affected by the deviation is measured and labelled with the deviation
    And  no affected cell is reported as zero

  @givn.added @wip
  Scenario: A candidate that fails its setup probe is unavailable, not zero
    Given  a real candidate whose documented interface cannot be started
    When  the maintainer runs the real benchmark
    Then  every metric cell for that candidate is unavailable with its setup reason
    And  the other installed candidates are measured
    And  the report lists the candidate as unavailable

  @givn.added @wip
  Scenario: The committed real run records its recording mode, cassette fingerprints, and model dependencies
    Given  the committed real benchmark evidence
    Then  it states that the run used record mode
    And  it records the model and result cassette fingerprints
    And  every measured cell's model dependency exists in the model cassette
    And  its provenance record names the exact run command, the UTC start and end, both endpoint localities, both model names, and the explanation-query budget
    And  its provenance pins, cassette digests, and index outcomes agree with the manifest, setup lock, cassettes, and report

  @givn.added @wip
  Scenario: A real replay reproduces the report without an external call
    Given  a recorded real benchmark run
    When  the maintainer replays the recorded run
    Then  the measurements and their order match the recorded run
    And  no request leaves the recording boundary
    And  no candidate process starts
    And  every installed candidate's measured slice replays identically

  @givn.added @wip
  Scenario: A changed query does not reuse a recorded model response
    Given  a recorded real benchmark run and one of its gold explanation queries
    When  the maintainer copies the fixed inputs, changes that query's text, recomputes the gold set fingerprint, and replays
    Then  the run exits with code 2
    And  the error is a missing recorded response for the changed query's digest
    And  the recorded response for the original query is not used
    And  no report.json or report.md is published

  @givn.added @wip
  Scenario: An unreviewed gold set referenced by the manifest stops before scoring
    Given  a real manifest that references a gold set with no reviewed_by
    When  the benchmark loads the gold sets
    Then  the run exits with code 2
    And  the error is the typed "unreviewed gold set" error naming the set and metric
    And  no report.json or report.md is published
    And  a draft gold file that the manifest does not reference is not loaded

  @givn.added @wip
  Scenario: The real record run covers every manifest candidate and repository
    Given  the real benchmark manifest and the committed result cassette
    When  the maintainer replays the recorded real run
    Then  every candidate, repository, and metric combination in the manifest has a measured, unavailable, or not-applicable cell
    And  no manifest candidate or repository is omitted
    And  every index outcome in the provenance record agrees with the report
    And  an index outcome that exceeded its deadline is unavailable with the reason "index_timeout" and its elapsed time
    And  each installed candidate's recorded slice replays identically

  @givn.added @wip
  Scenario: The real gold sets declare reviewed task counts
    Given  the real gold sets for every reference repository
    Then  each gold set declares a version, a reviewed task count, a content fingerprint, and a reviewer
    And  the primary repository's reviewed task counts meet 50 typo, 30 natural-language, 20 explanation, 50 concept, and 20 concept-equivalence
    And  each secondary repository's reviewed task counts meet 25 typo, 15 natural-language, 10 explanation, 25 concept, and 10 concept-equivalence
    And  each secondary repository is labelled lower-power
    And  the synthetic monorepo's gold is generated and reviewed with its generator

  @givn.added
  Scenario: A real candidate's evidence keeps its order and resolution
    Given  a real candidate that returns duplicate answers and a file-only answer
    When  the maintainer runs the real benchmark
    Then  the normalized results keep the candidate's order
    And  duplicate locations are removed without reordering
    And  the file-only answer remains a file resolution

  @givn.added
  Scenario: A real candidate without an operation is unavailable with the capability reason
    Given  a real candidate whose documented interface has no explanation operation
    When  the maintainer runs the real benchmark
    Then  its explanation cell is unavailable with the capability reason
    And  its measured families remain measured
    And  the report lists the missing operation as a capability gap

  @givn.added @wip
  Scenario: The real report publishes the measured comparison and defers the product path
    Given  a recorded real benchmark run
    When  the maintainer reads the real report
    Then  it contains the typo, natural-language, explanation, and concept-recall measurements and their measured gaps
    And  it contains the concept no-expansion arm where applicable
    And  it names the committed targets with their labels and sources, the targets fingerprint, and the provenance fingerprint
    And  it names both endpoint localities, both model names, and the explanation-query budget
    And  it labels each secondary repository lower-power and excludes it from cross-repository aggregate claims
    And  it names every candidate, repository, gold set, and cassette fingerprint
    And  it records the product-path decision as deferred to the Ortsinn maintainer and the post-report decision record

  @givn.added @wip
  Scenario: A real run below the per-family measurement floor publishes no report
    Given  a real run where one metric family and one repository have no measured cell
    When  the benchmark writes the report
    Then  the run exits with code 2
    And  the error names the missing measurement family and repository
    And  no report.json or report.md is published

  @givn.added @wip
  Scenario: A real gold set below the published minimum stops before scoring
    Given  a real gold set whose reviewed count is below the published minimum for its repository power
    When  the benchmark loads the gold sets
    Then  the run exits with code 2
    And  the error names the gold set, the metric, the published minimum, and the reviewed count
    And  no report.json or report.md is published

  @givn.modified @wip
  Scenario: Replay rerun reproduces the report without an external model call
    Given  a completed benchmark run with recorded model responses
    When  the maintainer reruns the same fixed inputs in replay mode
    Then  the measurements and their order match the first run
    And  no request leaves the local recording boundary
    And  the report states the replay mode and the model cassette fingerprint declared by the fixture manifest
    And  the committed model cassette's recomputed digest equals that declared fingerprint
    And  every model-cassette key carries the query's SHA-256 digest
