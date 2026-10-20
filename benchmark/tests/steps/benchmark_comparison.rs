use std::fs;
use std::path::{Path, PathBuf};

use benchmark::manifest::Manifest;
use benchmark::report::model::Report;
use cucumber::{given, then, when};

use crate::BenchmarkWorld;

fn report(world: &BenchmarkWorld) -> &Report {
    world
        .report
        .as_ref()
        .expect("the benchmark replay must have produced a report")
}

fn run_replay(world: &mut BenchmarkWorld) -> Result<(), String> {
    match benchmark::runner::run_replay(&world.manifest_path, &world.out_dir) {
        Ok(report) => {
            world.report = Some(report);
            world.run_failure = None;
        }
        Err(failure) => {
            world.report = None;
            world.run_failure = Some(failure);
        }
    }
    Ok(())
}

fn load_fixture(world: &mut BenchmarkWorld) -> Result<(), String> {
    let manifest = Manifest::load(&world.manifest_path)
        .map_err(|error| format!("load fixture manifest: {error:#}"))?;
    world.manifest = Some(manifest);
    Ok(())
}

#[given("the fixture benchmark inputs")]
fn the_fixture_benchmark_inputs(world: &mut BenchmarkWorld) -> Result<(), String> {
    load_fixture(world)
}

#[when("the maintainer runs the benchmark in replay mode")]
fn the_maintainer_runs_the_benchmark_in_replay_mode(
    world: &mut BenchmarkWorld,
) -> Result<(), String> {
    run_replay(world)
}

#[then(regex = r#"^the report names (\S+) at version "([^"]+)" and commit "([^"]+)"$"#)]
fn the_report_names_candidate(
    world: &mut BenchmarkWorld,
    captures: &[String],
) -> Result<(), String> {
    let [candidate_id, version, commit] = captures else {
        return Err("candidate identity step needs three captures".into());
    };
    let candidate = report(world)
        .candidates
        .iter()
        .find(|candidate| &candidate.id == candidate_id)
        .ok_or_else(|| format!("report does not name candidate {candidate_id}"))?;
    assert_eq!(&candidate.version, version, "candidate version");
    assert_eq!(&candidate.commit, commit, "candidate commit");
    Ok(())
}

#[then(
    regex = r#"^the report names (\S+) at revision "([^"]+)", commit "([^"]+)", and (\d+) code lines$"#
)]
fn the_report_names_repository(
    world: &mut BenchmarkWorld,
    captures: &[String],
) -> Result<(), String> {
    let [repository_id, revision, commit, code_lines] = captures else {
        return Err("repository identity step needs four captures".into());
    };
    let repository = report(world)
        .inputs
        .repositories
        .iter()
        .find(|repository| &repository.id == repository_id)
        .ok_or_else(|| format!("report does not name repository {repository_id}"))?;
    assert_eq!(&repository.revision, revision, "repository revision");
    assert_eq!(&repository.commit, commit, "repository commit");
    let expected: u64 = code_lines
        .parse()
        .map_err(|error| format!("parse code line count: {error}"))?;
    assert_eq!(
        repository.loc.language_code_lines, expected,
        "repository code lines"
    );
    Ok(())
}

fn check_value(actual: f64, expected: &str, label: &str) -> Result<(), String> {
    let expected: f64 = expected
        .parse()
        .map_err(|error| format!("parse expected {label}: {error}"))?;
    if (actual - expected).abs() > 1e-9 {
        return Err(format!("{label}: expected {expected}, got {actual}"));
    }
    Ok(())
}

fn check_counts(
    actual_numerator: u64,
    actual_denominator: u64,
    numerator: &str,
    denominator: &str,
    label: &str,
) -> Result<(), String> {
    let numerator: u64 = numerator
        .parse()
        .map_err(|error| format!("parse {label} numerator: {error}"))?;
    let denominator: u64 = denominator
        .parse()
        .map_err(|error| format!("parse {label} denominator: {error}"))?;
    assert_eq!(actual_numerator, numerator, "{label} numerator");
    assert_eq!(actual_denominator, denominator, "{label} denominator");
    Ok(())
}

#[then(regex = r"^the typo recall at rank one is ([\d.]+) with (\d+) of (\d+) tasks$")]
fn typo_recall_at_one(world: &mut BenchmarkWorld, captures: &[String]) -> Result<(), String> {
    let [value, numerator, denominator] = captures else {
        return Err("typo @1 step needs three captures".into());
    };
    let score = &report(world).metrics.typo.aggregate;
    check_value(
        score.value.ok_or("typo @1 has no measured value")?,
        value,
        "typo recall @1",
    )?;
    check_counts(
        score.numerator,
        score.denominator,
        numerator,
        denominator,
        "typo recall @1",
    )
}

#[then(regex = r"^the typo recall at rank five is ([\d.]+) with (\d+) of (\d+) tasks$")]
fn typo_recall_at_five(world: &mut BenchmarkWorld, captures: &[String]) -> Result<(), String> {
    let [value, numerator, denominator] = captures else {
        return Err("typo @5 step needs three captures".into());
    };
    let score = &report(world).metrics.typo.recall_at_5;
    check_value(
        score.value.ok_or("typo @5 has no measured value")?,
        value,
        "typo recall @5",
    )?;
    check_counts(
        score.numerator,
        score.denominator,
        numerator,
        denominator,
        "typo recall @5",
    )
}

#[then(regex = r"^the natural-language recall at rank five is ([\d.]+) with (\d+) of (\d+) tasks$")]
fn natural_language_recall_at_five(
    world: &mut BenchmarkWorld,
    captures: &[String],
) -> Result<(), String> {
    let [value, numerator, denominator] = captures else {
        return Err("NL recall step needs three captures".into());
    };
    let score = &report(world).metrics.natural_language.aggregate;
    check_value(
        score.value.ok_or("NL recall has no measured value")?,
        value,
        "NL recall @5",
    )?;
    check_counts(
        score.numerator,
        score.denominator,
        numerator,
        denominator,
        "NL recall @5",
    )
}

#[then(regex = r"^the natural-language mean reciprocal rank at rank ten is ([\d.]+)$")]
fn natural_language_mrr(world: &mut BenchmarkWorld, captures: &[String]) -> Result<(), String> {
    let [value] = captures else {
        return Err("NL MRR step needs one capture".into());
    };
    check_value(
        report(world).metrics.natural_language.mrr_at_10.value,
        value,
        "NL MRR @10",
    )
}

#[then(regex = r"^the explanation correctness is ([\d.]+) with (\d+) of (\d+) tasks$")]
fn explanation_correctness(world: &mut BenchmarkWorld, captures: &[String]) -> Result<(), String> {
    let [value, numerator, denominator] = captures else {
        return Err("explanation correctness step needs three captures".into());
    };
    let score = &report(world).metrics.explanation.aggregate;
    check_value(
        score
            .value
            .ok_or("explanation correctness has no measured value")?,
        value,
        "explanation correctness",
    )?;
    check_counts(
        score.numerator,
        score.denominator,
        numerator,
        denominator,
        "explanation correctness",
    )
}

#[then(regex = r"^the explanation evidence grounding is ([\d.]+) with (\d+) of (\d+) tasks$")]
fn explanation_grounding(world: &mut BenchmarkWorld, captures: &[String]) -> Result<(), String> {
    let [value, numerator, denominator] = captures else {
        return Err("explanation grounding step needs three captures".into());
    };
    let grounding = &report(world).metrics.explanation.evidence_grounding;
    check_value(grounding.value, value, "explanation grounding")?;
    let expected_numerator: u64 = numerator
        .parse()
        .map_err(|error| format!("parse grounding numerator: {error}"))?;
    let expected_denominator: u64 = denominator
        .parse()
        .map_err(|error| format!("parse grounding denominator: {error}"))?;
    assert_eq!(
        grounding.attempted, expected_denominator,
        "explanation grounding attempted"
    );
    assert_eq!(
        (grounding.value * grounding.attempted as f64).round() as u64,
        expected_numerator,
        "explanation grounding numerator"
    );
    Ok(())
}

#[then(regex = r"^the concept recall at rank five is ([\d.]+) with (\d+) of (\d+) tasks$")]
fn concept_recall_at_five(world: &mut BenchmarkWorld, captures: &[String]) -> Result<(), String> {
    let [value, numerator, denominator] = captures else {
        return Err("concept recall step needs three captures".into());
    };
    let score = &report(world).metrics.concept.aggregate;
    check_value(
        score.value.ok_or("concept recall has no measured value")?,
        value,
        "concept recall @5",
    )?;
    check_counts(
        score.numerator,
        score.denominator,
        numerator,
        denominator,
        "concept recall @5",
    )
}

#[then(
    regex = r"^the concept-equivalence recall at rank five is ([\d.]+) with (\d+) of (\d+) variants$"
)]
fn concept_equivalence_recall_at_five(
    world: &mut BenchmarkWorld,
    captures: &[String],
) -> Result<(), String> {
    let [value, numerator, denominator] = captures else {
        return Err("concept-equivalence step needs three captures".into());
    };
    let score = &report(world).metrics.concept_equivalence.aggregate;
    check_value(
        score
            .value
            .ok_or("concept-equivalence recall has no measured value")?,
        value,
        "concept-equivalence recall @5",
    )?;
    check_counts(
        score.numerator,
        score.denominator,
        numerator,
        denominator,
        "concept-equivalence recall @5",
    )
}

#[then(regex = r"^the typo recall at rank one shows a gap of (-?[\d.]+) to the target ([\d.]+)$")]
fn typo_gap_to_target(world: &mut BenchmarkWorld, captures: &[String]) -> Result<(), String> {
    let [gap, target] = captures else {
        return Err("typo gap step needs two captures".into());
    };
    let report = report(world);
    let measured_target = report
        .metrics
        .typo
        .target
        .value
        .ok_or("typo target is not stated")?;
    check_value(measured_target, target, "typo target")?;
    let entry = report
        .gaps
        .to_target
        .get("typo_recall_at_1")
        .ok_or("report has no typo to-target gap")?;
    check_value(
        entry.value.ok_or("typo to-target gap has no value")?,
        gap,
        "typo to-target gap",
    )?;
    if entry.status != benchmark::report::model::CellStatus::Measured {
        return Err(format!("typo to-target gap is {:?}", entry.status));
    }
    Ok(())
}

#[then(regex = r#"^the report labels the ([\d.]+) target "([^"]+)" and never "([^"]+)"$"#)]
fn target_label(world: &mut BenchmarkWorld, captures: &[String]) -> Result<(), String> {
    let [value, label, forbidden] = captures else {
        return Err("target label step needs three captures".into());
    };
    let target = &report(world).metrics.typo.target;
    check_value(
        target.value.ok_or("typo target has no value")?,
        value,
        "target value",
    )?;
    let actual = match target.status {
        benchmark::manifest::TargetStatus::Proposed => "proposed",
        benchmark::manifest::TargetStatus::Confirmed => "confirmed",
        benchmark::manifest::TargetStatus::Unstated => "unstated",
    };
    assert_eq!(actual, label, "target status");
    if actual == forbidden {
        return Err(format!("target must never be labelled {forbidden}"));
    }
    let gap = report(world)
        .gaps
        .to_target
        .get("typo_recall_at_1")
        .ok_or("report has no typo to-target gap")?;
    assert_eq!(
        gap.target_status,
        Some(benchmark::manifest::TargetStatus::Proposed),
        "gap target status"
    );
    Ok(())
}

#[then(
    regex = r"^the natural-language recall at rank five shows its measured value ([\d.]+) and the best measured value ([\d.]+)$"
)]
fn natural_language_measured_and_best(
    world: &mut BenchmarkWorld,
    captures: &[String],
) -> Result<(), String> {
    let [measured, best] = captures else {
        return Err("NL measured/best step needs two captures".into());
    };
    let report = report(world);
    check_value(
        report
            .metrics
            .natural_language
            .aggregate
            .value
            .ok_or("NL recall has no measured value")?,
        measured,
        "NL measured value",
    )?;
    let gap = report
        .gaps
        .to_best
        .get("natural_language_recall_at_5")
        .ok_or("report has no NL to-best gap")?;
    check_value(
        gap.best_value.ok_or("NL to-best gap has no best value")?,
        best,
        "NL best measured value",
    )
}

#[then(
    regex = r"^the concept recall at rank five shows its measured value ([\d.]+) and the best measured value ([\d.]+)$"
)]
fn concept_measured_and_best(
    world: &mut BenchmarkWorld,
    captures: &[String],
) -> Result<(), String> {
    let [measured, best] = captures else {
        return Err("concept measured/best step needs two captures".into());
    };
    let report = report(world);
    check_value(
        report
            .metrics
            .concept
            .aggregate
            .value
            .ok_or("concept recall has no measured value")?,
        measured,
        "concept measured value",
    )?;
    let gap = report
        .gaps
        .to_best
        .get("concept_recall_at_5")
        .ok_or("report has no concept to-best gap")?;
    check_value(
        gap.best_value
            .ok_or("concept to-best gap has no best value")?,
        best,
        "concept best measured value",
    )
}

#[then("the report records the product-path decision as deferred")]
fn path_decision_deferred(world: &mut BenchmarkWorld) -> Result<(), String> {
    assert_eq!(
        report(world).path_decision.status,
        benchmark::report::model::PathDecisionStatus::Deferred,
        "path decision status"
    );
    Ok(())
}

#[then(
    "the report names the Ortsinn maintainer and the post-report decision record as the decision owner and artifact"
)]
fn path_decision_owner_and_artifact(world: &mut BenchmarkWorld) -> Result<(), String> {
    let decision = &report(world).path_decision;
    assert_eq!(decision.owner, "maintainer--ortsinn", "decision owner");
    assert_eq!(
        decision.artifact, "docs/decisions/benchmark-path.md",
        "decision artifact"
    );
    Ok(())
}

#[given("a candidate that supports disabling query expansion")]
fn a_candidate_supports_disabling_expansion(world: &mut BenchmarkWorld) -> Result<(), String> {
    load_fixture(world)?;
    let manifest = world
        .manifest
        .as_ref()
        .ok_or("fixture manifest was not loaded")?;
    let expansion_supported = manifest.candidates.iter().any(|candidate| {
        candidate
            .capabilities
            .get("expansion")
            .is_some_and(|capability| {
                capability.status == benchmark::manifest::CapabilityStatus::Measured
            })
    });
    if !expansion_supported {
        return Err("fixture has no candidate with a query-expansion switch".into());
    }
    Ok(())
}

#[then(regex = r"^the concept recall for the default arm is ([\d.]+) with (\d+) of (\d+) tasks$")]
fn concept_default_arm(world: &mut BenchmarkWorld, captures: &[String]) -> Result<(), String> {
    let [value, numerator, denominator] = captures else {
        return Err("concept default arm step needs three captures".into());
    };
    let score = &report(world).metrics.concept.arms.default.score;
    check_value(
        score
            .value
            .ok_or("concept default arm has no measured value")?,
        value,
        "concept default arm",
    )?;
    check_counts(
        score.numerator,
        score.denominator,
        numerator,
        denominator,
        "concept default arm",
    )
}

#[then(
    regex = r"^the concept recall for the no-expansion arm is ([\d.]+) with (\d+) of (\d+) tasks$"
)]
fn concept_no_expansion_arm(world: &mut BenchmarkWorld, captures: &[String]) -> Result<(), String> {
    let [value, numerator, denominator] = captures else {
        return Err("concept no-expansion arm step needs three captures".into());
    };
    let score = &report(world).metrics.concept.arms.no_expansion.score;
    check_value(
        score
            .value
            .ok_or("concept no-expansion arm has no measured value")?,
        value,
        "concept no-expansion arm",
    )?;
    check_counts(
        score.numerator,
        score.denominator,
        numerator,
        denominator,
        "concept no-expansion arm",
    )
}

#[then(
    regex = r"^the concept-equivalence recall for the default arm is ([\d.]+) with (\d+) of (\d+) variants$"
)]
fn concept_equivalence_default_arm(
    world: &mut BenchmarkWorld,
    captures: &[String],
) -> Result<(), String> {
    let [value, numerator, denominator] = captures else {
        return Err("concept-equivalence default arm step needs three captures".into());
    };
    let score = &report(world).metrics.concept_equivalence.arms.default.score;
    check_value(
        score
            .value
            .ok_or("concept-equivalence default arm has no measured value")?,
        value,
        "concept-equivalence default arm",
    )?;
    check_counts(
        score.numerator,
        score.denominator,
        numerator,
        denominator,
        "concept-equivalence default arm",
    )
}

#[then(
    regex = r"^the concept-equivalence recall for the no-expansion arm is ([\d.]+) with (\d+) of (\d+) variants$"
)]
fn concept_equivalence_no_expansion_arm(
    world: &mut BenchmarkWorld,
    captures: &[String],
) -> Result<(), String> {
    let [value, numerator, denominator] = captures else {
        return Err("concept-equivalence no-expansion arm step needs three captures".into());
    };
    let score = &report(world)
        .metrics
        .concept_equivalence
        .arms
        .no_expansion
        .score;
    check_value(
        score
            .value
            .ok_or("concept-equivalence no-expansion arm has no measured value")?,
        value,
        "concept-equivalence no-expansion arm",
    )?;
    check_counts(
        score.numerator,
        score.denominator,
        numerator,
        denominator,
        "concept-equivalence no-expansion arm",
    )
}

#[then(
    regex = r#"^fixture-serena's no-expansion arm is not applicable with the reason "([^"]+)", not zero$"#
)]
fn serena_no_expansion_not_applicable(
    world: &mut BenchmarkWorld,
    captures: &[String],
) -> Result<(), String> {
    let [reason] = captures else {
        return Err("serena no-expansion step needs one capture".into());
    };
    let cell = report(world)
        .metrics
        .concept
        .arms
        .no_expansion
        .per_candidate
        .get("fixture-serena")
        .ok_or("report has no fixture-serena no-expansion cell")?;
    assert_eq!(
        cell.status,
        benchmark::report::model::CellStatus::NotApplicable,
        "fixture-serena no-expansion status"
    );
    assert_eq!(cell.reason.as_deref(), Some(reason.as_str()), "reason");
    assert_eq!(cell.value, None, "not-applicable arm must not be zero");
    Ok(())
}

#[given("a candidate that cannot explain code")]
fn a_candidate_cannot_explain_code(world: &mut BenchmarkWorld) -> Result<(), String> {
    load_fixture(world)?;
    let manifest = world
        .manifest
        .as_ref()
        .ok_or("fixture manifest was not loaded")?;
    let unavailable = manifest.candidates.iter().any(|candidate| {
        candidate
            .capabilities
            .get("explanation")
            .is_some_and(|capability| {
                capability.status == benchmark::manifest::CapabilityStatus::Unavailable
            })
    });
    if !unavailable {
        return Err("fixture has no candidate without an explanation operation".into());
    }
    Ok(())
}

#[then(
    regex = r#"^the explanation correctness for (\S+) is unavailable with the reason "([^"]+)"$"#
)]
fn explanation_correctness_unavailable(
    world: &mut BenchmarkWorld,
    captures: &[String],
) -> Result<(), String> {
    let [candidate, reason] = captures else {
        return Err("explanation unavailable step needs two captures".into());
    };
    let cell = report(world)
        .metrics
        .explanation
        .per_candidate
        .get(candidate)
        .ok_or_else(|| format!("report has no explanation cell for {candidate}"))?;
    assert_eq!(
        cell.status,
        benchmark::report::model::CellStatus::Unavailable,
        "explanation status"
    );
    assert_eq!(cell.reason.as_deref(), Some(reason.as_str()), "reason");
    assert_eq!(cell.value, None, "unavailable explanation must not be zero");
    Ok(())
}

#[then(regex = r"^the explanation coverage for (\S+) is (\d+) of (\d+) with a value of ([\d.]+)$")]
fn explanation_coverage(world: &mut BenchmarkWorld, captures: &[String]) -> Result<(), String> {
    let [candidate, numerator, denominator, value] = captures else {
        return Err("explanation coverage step needs four captures".into());
    };
    let cell = report(world)
        .metrics
        .explanation
        .per_candidate
        .get(candidate)
        .ok_or_else(|| format!("report has no explanation cell for {candidate}"))?;
    let expected_numerator: u64 = numerator
        .parse()
        .map_err(|error| format!("parse coverage numerator: {error}"))?;
    let expected_denominator: u64 = denominator
        .parse()
        .map_err(|error| format!("parse coverage denominator: {error}"))?;
    assert_eq!(
        cell.coverage.numerator, expected_numerator,
        "coverage numerator"
    );
    assert_eq!(
        cell.coverage.denominator, expected_denominator,
        "coverage denominator"
    );
    check_value(
        cell.coverage
            .value
            .ok_or("coverage has no value for an unavailable cell")?,
        value,
        "coverage value",
    )
}

#[then(
    regex = r"^the aggregate explanation correctness remains ([\d.]+) with (\d+) of (\d+) tasks$"
)]
fn aggregate_explanation_correctness(
    world: &mut BenchmarkWorld,
    captures: &[String],
) -> Result<(), String> {
    let [value, numerator, denominator] = captures else {
        return Err("aggregate explanation step needs three captures".into());
    };
    let score = &report(world).metrics.explanation.aggregate;
    check_value(
        score
            .value
            .ok_or("aggregate explanation correctness has no value")?,
        value,
        "aggregate explanation correctness",
    )?;
    check_counts(
        score.numerator,
        score.denominator,
        numerator,
        denominator,
        "aggregate explanation correctness",
    )
}

#[then(
    regex = r"^the report lists the missing explanation capability as a capability gap for (\S+)$"
)]
fn explanation_capability_gap(
    world: &mut BenchmarkWorld,
    captures: &[String],
) -> Result<(), String> {
    let [candidate] = captures else {
        return Err("explanation capability gap step needs one capture".into());
    };
    let gap = report(world)
        .gaps
        .capability_gaps
        .iter()
        .find(|gap| gap.candidate == *candidate && gap.capability == "explanation")
        .ok_or_else(|| {
            format!("report does not list an explanation capability gap for {candidate}")
        })?;
    assert_eq!(
        gap.reason, "no_explanation_operation",
        "capability gap reason"
    );
    Ok(())
}

#[given("a completed benchmark run with recorded model responses")]
fn a_completed_benchmark_run(world: &mut BenchmarkWorld) -> Result<(), String> {
    load_fixture(world)?;
    let first = benchmark::runner::run_replay(&world.manifest_path, &world.out_dir)
        .map_err(|error| format!("run the completed benchmark: {error}"))?;
    world.first_report = Some(first);
    Ok(())
}

#[when("the maintainer reruns the same fixed inputs in replay mode")]
fn the_maintainer_reruns_in_replay_mode(world: &mut BenchmarkWorld) -> Result<(), String> {
    run_replay(world)
}

#[then("the measurements and their order match the first run")]
fn measurements_and_order_match(world: &mut BenchmarkWorld) -> Result<(), String> {
    let first = world
        .first_report
        .as_ref()
        .ok_or("no completed run to compare against")?;
    let second = report(world);
    if format!("{:?}", first.metrics) != format!("{:?}", second.metrics) {
        return Err("replay measurements or their order changed".into());
    }
    if format!("{:?}", first.gaps) != format!("{:?}", second.gaps) {
        return Err("replay gaps or their order changed".into());
    }
    Ok(())
}

#[then("no request leaves the local recording boundary")]
fn no_external_request(world: &mut BenchmarkWorld) -> Result<(), String> {
    let requests = report(world).run.external_requests;
    if requests != 0 {
        return Err(format!(
            "replay attempted {requests} external model request(s)"
        ));
    }
    Ok(())
}

#[then(
    regex = r#"^the report states the replay mode and the recording fingerprint "([^"]+)"$"#
)]
fn replay_mode_and_recording_fingerprint(
    world: &mut BenchmarkWorld,
    captures: &[String],
) -> Result<(), String> {
    let [fingerprint] = captures else {
        return Err("replay identity step needs one capture".into());
    };
    let run = &report(world).run;
    assert_eq!(run.mode, benchmark::manifest::RunMode::Replay, "run mode");
    assert_eq!(
        &run.recording_fingerprint, fingerprint,
        "recording fingerprint"
    );
    Ok(())
}

const MISSING_MODEL_KEY: &str = "model:explanation:fixture-ckb:exp-2";
const MISSING_MODEL_CANDIDATE: &str = "fixture-ckb";

fn missing_response_fixture(world: &mut BenchmarkWorld) -> Result<(), String> {
    let fixture = world
        .manifest_path
        .parent()
        .ok_or("fixture manifest has no parent directory")?
        .to_path_buf();
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("benchmark crate has no workspace root")?
        .join("target")
        .join("benchmark-missing-model");
    if root.exists() {
        fs::remove_dir_all(&root).map_err(|error| format!("clear {}: {error}", root.display()))?;
    }
    fs::create_dir_all(root.join("cassettes/results"))
        .map_err(|error| format!("create missing-response fixture: {error}"))?;
    fs::create_dir_all(root.join("cassettes/model"))
        .map_err(|error| format!("create missing-response fixture: {error}"))?;
    copy_tree(&fixture.join("gold"), &root.join("gold"))?;
    fs::copy(
        fixture.join("cassettes/results/results.jsonl"),
        root.join("cassettes/results/results.jsonl"),
    )
    .map_err(|error| format!("copy result cassette: {error}"))?;
    fs::copy(
        fixture.join("cassettes/model/model.jsonl"),
        root.join("cassettes/model/model.jsonl"),
    )
    .map_err(|error| format!("copy model cassette: {error}"))?;
    let model_path = root.join("cassettes/model/model.jsonl");
    let text = fs::read_to_string(&model_path)
        .map_err(|error| format!("read copied model cassette: {error}"))?;
    let kept: Vec<&str> = text
        .lines()
        .filter(|line| !line.contains(MISSING_MODEL_KEY))
        .collect();
    fs::write(&model_path, format!("{}\n", kept.join("\n")))
        .map_err(|error| format!("write model cassette without {MISSING_MODEL_KEY}: {error}"))?;
    fs::copy(fixture.join("manifest.yaml"), root.join("manifest.yaml"))
        .map_err(|error| format!("copy fixture manifest: {error}"))?;

    world.manifest_path = root.join("manifest.yaml");
    world.out_dir = root.join("out");
    world.missing_model = Some((
        MISSING_MODEL_KEY.to_string(),
        MISSING_MODEL_CANDIDATE.to_string(),
    ));
    load_fixture(world)
}

fn copy_tree(from: &Path, to: &Path) -> Result<(), String> {
    fs::create_dir_all(to).map_err(|error| format!("create {}: {error}", to.display()))?;
    for entry in fs::read_dir(from).map_err(|error| format!("read {}: {error}", from.display()))? {
        let entry = entry.map_err(|error| format!("read {}: {error}", from.display()))?;
        let source = entry.path();
        let target = to.join(entry.file_name());
        if source.is_dir() {
            copy_tree(&source, &target)?;
        } else {
            fs::copy(&source, &target)
                .map_err(|error| format!("copy {}: {error}", source.display()))?;
        }
    }
    Ok(())
}

#[given("a benchmark run that needs a model response absent from the recording")]
fn a_run_needs_a_missing_model_response(world: &mut BenchmarkWorld) -> Result<(), String> {
    missing_response_fixture(world)
}

#[then("the run exits with code 2")]
fn the_run_exits_with_code_two(world: &mut BenchmarkWorld) -> Result<(), String> {
    let failure = world
        .run_failure
        .as_ref()
        .ok_or("the run did not fail; a report was produced")?;
    if failure.exit_code != benchmark::runner::EXIT_INPUT_ERROR {
        return Err(format!(
            "run exit code was {}, expected {}",
            failure.exit_code,
            benchmark::runner::EXIT_INPUT_ERROR
        ));
    }
    Ok(())
}

#[then("the error names the missing input and the candidate that requested it")]
fn the_error_names_the_missing_input(world: &mut BenchmarkWorld) -> Result<(), String> {
    let failure = world
        .run_failure
        .as_ref()
        .ok_or("the run did not fail; no error to inspect")?;
    let (key, candidate) = world
        .missing_model
        .as_ref()
        .ok_or("the missing-response fixture did not record its key")?;
    if !failure.message.contains(key) {
        return Err(format!(
            "error does not name the missing input {key}: {}",
            failure.message
        ));
    }
    if !failure.message.contains(candidate) {
        return Err(format!(
            "error does not name the candidate {candidate}: {}",
            failure.message
        ));
    }
    Ok(())
}

#[then("no report.json or report.md is published")]
fn no_report_is_published(world: &mut BenchmarkWorld) -> Result<(), String> {
    for name in ["report.json", "report.md"] {
        let path = world.out_dir.join(name);
        if path.exists() {
            return Err(format!("failed run published {}", path.display()));
        }
    }
    Ok(())
}
