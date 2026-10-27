use std::fs;
use std::path::{Path, PathBuf};

use benchmark::adapters::session::{
    CandidateAnswer, CandidateSession, FakeSession, Operation, QueryRequest, apply_session,
    normalize_answers,
};
use benchmark::adapters::NormalizedLocation;
use benchmark::manifest::{Candidate, Manifest};
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
    if world.e2e_endpoint.is_some() {
        return crate::steps::e2e::benchmark_cli::assert_recording_boundary(world);
    }
    let requests = report(world).run.external_requests;
    if requests != 0 {
        return Err(format!(
            "replay attempted {requests} external model request(s)"
        ));
    }
    Ok(())
}

#[then(regex = r#"^the report states the replay mode and the recording fingerprint "([^"]+)"$"#)]
fn replay_mode_and_recording_fingerprint(
    world: &mut BenchmarkWorld,
    captures: &[String],
) -> Result<(), String> {
    let [fingerprint] = captures else {
        return Err("replay identity step needs one capture".into());
    };
    let report = report(world);
    assert_eq!(
        report.run.mode,
        benchmark::manifest::RunMode::Replay,
        "run mode"
    );
    assert_eq!(
        &report.reproduction.model_cassette_fingerprint, fingerprint,
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
    copy_tree(&fixture.join("repo"), &root.join("repo"))?;
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
    let mut manifest = read_fixture_manifest()?;
    refresh_cassette_fingerprint(&mut manifest, &root, "model")?;
    world.manifest_path = write_variant_manifest(&root, &manifest)?;
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

#[given("the fixed synthetic seed and generator version")]
fn the_fixed_synthetic_seed_and_generator_version(
    world: &mut BenchmarkWorld,
) -> Result<(), String> {
    use benchmark::r#gen::synthetic_repo as synthetic;
    assert_eq!(synthetic::SEED, 20261020, "synthetic seed");
    assert_eq!(synthetic::GENERATOR_VERSION, 1, "generator version");
    world.synthetic_first = Some(synthetic::generate());
    Ok(())
}

#[when("the maintainer regenerates the synthetic monorepo")]
fn the_maintainer_regenerates_the_synthetic_monorepo(
    world: &mut BenchmarkWorld,
) -> Result<(), String> {
    world.synthetic_second = Some(benchmark::r#gen::synthetic_repo::generate());
    Ok(())
}

fn synthetic_repo(world: &BenchmarkWorld) -> &benchmark::r#gen::synthetic_repo::SyntheticRepo {
    world
        .synthetic_second
        .as_ref()
        .expect("the synthetic monorepo must have been regenerated")
}

#[then("it contains ten million code lines across the TypeScript, Python, and Rust languages")]
fn synthetic_ten_million_lines(world: &mut BenchmarkWorld) -> Result<(), String> {
    use benchmark::r#gen::synthetic_repo as synthetic;
    let repo = synthetic_repo(world);
    assert_eq!(
        repo.code_lines,
        synthetic::CODE_LINES,
        "synthetic code lines"
    );
    for language in ["typescript", "python", "rust"] {
        let lines = repo
            .lines_by_language
            .get(language)
            .ok_or_else(|| format!("synthetic repository has no {language} code lines"))?;
        if *lines == 0 {
            return Err(format!(
                "synthetic repository has zero {language} code lines"
            ));
        }
    }
    let total: u64 = repo.lines_by_language.values().sum();
    if total != synthetic::CODE_LINES {
        return Err(format!(
            "synthetic language totals sum to {total}, expected {}",
            synthetic::CODE_LINES
        ));
    }
    Ok(())
}

#[then(regex = r#"^its content fingerprint matches the recorded fixed input "([^"]+)"$"#)]
fn synthetic_content_fingerprint(
    world: &mut BenchmarkWorld,
    captures: &[String],
) -> Result<(), String> {
    let [fingerprint] = captures else {
        return Err("synthetic fingerprint step needs one capture".into());
    };
    let first = world
        .synthetic_first
        .as_ref()
        .ok_or("the synthetic monorepo was not generated before regeneration")?;
    let second = synthetic_repo(world);
    assert_eq!(
        &second.recorded_fingerprint, fingerprint,
        "recorded synthetic fingerprint"
    );
    if first.content_digest != second.content_digest {
        return Err(format!(
            "regenerated content digest {} differs from {}",
            second.content_digest, first.content_digest
        ));
    }
    Ok(())
}

#[given("the fixture typo gold set of five tasks")]
fn the_fixture_typo_gold_set(world: &mut BenchmarkWorld) -> Result<(), String> {
    load_fixture(world)?;
    let tasks = world
        .manifest
        .as_ref()
        .and_then(|manifest| manifest.gold_set("typo"))
        .map(|set| set.tasks)
        .ok_or("fixture manifest has no typo gold set")?;
    assert_eq!(tasks, 5, "fixture typo gold task count");
    Ok(())
}

#[then(regex = r"^(\S+) returns no result for (\d+) of the (\d+) typo tasks$")]
fn candidate_typo_no_match(world: &mut BenchmarkWorld, captures: &[String]) -> Result<(), String> {
    let [candidate, numerator, denominator] = captures else {
        return Err("typo no-match step needs three captures".into());
    };
    let rate = report(world)
        .metrics
        .typo
        .no_match_rate_per_candidate
        .get(candidate)
        .ok_or_else(|| format!("report has no typo no-match cell for {candidate}"))?;
    check_counts(
        rate.numerator,
        rate.denominator,
        numerator,
        denominator,
        "typo no-match rate",
    )
}

#[then(regex = r"^the typo no-match rate is ([\d.]+) with (\d+) of (\d+) tasks$")]
fn typo_no_match_rate(world: &mut BenchmarkWorld, captures: &[String]) -> Result<(), String> {
    let [value, numerator, denominator] = captures else {
        return Err("typo no-match aggregate step needs three captures".into());
    };
    let rate = &report(world).metrics.typo.no_match_rate;
    check_value(
        rate.value
            .ok_or("typo no-match rate has no measured value")?,
        value,
        "typo no-match rate",
    )?;
    check_counts(
        rate.numerator,
        rate.denominator,
        numerator,
        denominator,
        "typo no-match rate",
    )
}

#[then(
    regex = r"^the typo recall at rank five is ([\d.]+) because (\d+) of (\d+) tasks are ranked$"
)]
fn typo_recall_at_five_ranked(
    world: &mut BenchmarkWorld,
    captures: &[String],
) -> Result<(), String> {
    let [value, numerator, denominator] = captures else {
        return Err("typo ranked recall step needs three captures".into());
    };
    let typo = &report(world).metrics.typo;
    check_value(
        typo.recall_at_5
            .value
            .ok_or("typo @5 has no measured value")?,
        value,
        "typo recall @5",
    )?;
    check_counts(
        typo.recall_at_5.numerator,
        typo.recall_at_5.denominator,
        numerator,
        denominator,
        "typo recall @5",
    )?;
    let numerator: u64 = numerator
        .parse()
        .map_err(|error| format!("parse typo ranked numerator: {error}"))?;
    if typo.no_match_rate.numerator + numerator != typo.recall_at_5.denominator {
        return Err(format!(
            "ranked {} plus no-match {} does not equal the {} typo tasks",
            numerator, typo.no_match_rate.numerator, typo.recall_at_5.denominator
        ));
    }
    Ok(())
}

fn metric_coverage<'a>(
    metrics: &'a benchmark::report::model::Metrics,
    name: &str,
) -> Option<&'a benchmark::report::model::Coverage> {
    match name {
        "typo" => Some(&metrics.typo.aggregate.coverage),
        "natural-language" => Some(&metrics.natural_language.aggregate.coverage),
        "explanation" => Some(&metrics.explanation.aggregate.coverage),
        "concept" => Some(&metrics.concept.aggregate.coverage),
        "concept-equivalence" => Some(&metrics.concept_equivalence.aggregate.coverage),
        _ => None,
    }
}

fn check_coverage(
    label: &str,
    coverage: &benchmark::report::model::Coverage,
    numerator: &str,
    denominator: &str,
    value: &str,
) -> Result<(), String> {
    check_counts(
        coverage.numerator,
        coverage.denominator,
        numerator,
        denominator,
        label,
    )?;
    check_value(
        coverage
            .value
            .ok_or_else(|| format!("{label} coverage has no value"))?,
        value,
        label,
    )
}

#[then(regex = r"^the (\S+) coverage is (\d+) of (\d+) with a value of ([\d.]+)$")]
fn metric_coverage_step(world: &mut BenchmarkWorld, captures: &[String]) -> Result<(), String> {
    let [metric, numerator, denominator, value] = captures else {
        return Err("metric coverage step needs four captures".into());
    };
    let coverage = metric_coverage(&report(world).metrics, metric)
        .ok_or_else(|| format!("report has no {metric} aggregate coverage"))?;
    check_coverage(metric, coverage, numerator, denominator, value)
}

#[then(
    regex = r"^(\S+)'s explanation coverage is (\d+) of (\d+) and (\S+)'s explanation coverage is (\d+) of (\d+)$"
)]
fn explanation_coverage_per_candidate(
    world: &mut BenchmarkWorld,
    captures: &[String],
) -> Result<(), String> {
    let [
        first_candidate,
        first_numerator,
        first_denominator,
        second_candidate,
        second_numerator,
        second_denominator,
    ] = captures
    else {
        return Err("per-candidate explanation coverage step needs six captures".into());
    };
    let cells = &report(world).metrics.explanation.per_candidate;
    for (candidate, numerator, denominator) in [
        (first_candidate, first_numerator, first_denominator),
        (second_candidate, second_numerator, second_denominator),
    ] {
        let cell = cells
            .get(candidate)
            .ok_or_else(|| format!("report has no explanation cell for {candidate}"))?;
        check_counts(
            cell.coverage.numerator,
            cell.coverage.denominator,
            numerator,
            denominator,
            "candidate explanation coverage",
        )?;
        let expected = cell.coverage.numerator as f64 / cell.coverage.denominator as f64;
        let actual = cell
            .coverage
            .value
            .ok_or_else(|| format!("{candidate} coverage has no value"))?;
        if (actual - expected).abs() > 1e-9 {
            return Err(format!(
                "{candidate} coverage value {actual} is not {} / {}",
                cell.coverage.numerator, cell.coverage.denominator
            ));
        }
    }
    Ok(())
}

#[then(
    "every coverage value is attempted divided by attempted plus unavailable, excluding not-applicable cells"
)]
fn every_coverage_value_is_attempted_over_attempted_plus_unavailable(
    world: &mut BenchmarkWorld,
) -> Result<(), String> {
    use benchmark::report::model::{CellStatus, Metrics, Score};

    let metrics = &report(world).metrics;

    fn collect<'a>(cells: &mut Vec<(String, &'a Score)>, metric: &'a Metrics) {
        cells.push(("typo.aggregate".to_string(), &metric.typo.aggregate));
        for (candidate, cell) in &metric.typo.per_candidate {
            cells.push((format!("typo.{candidate}"), cell));
        }
        cells.push((
            "natural_language.aggregate".to_string(),
            &metric.natural_language.aggregate,
        ));
        for (candidate, cell) in &metric.natural_language.per_candidate {
            cells.push((format!("natural_language.{candidate}"), cell));
        }
        cells.push((
            "explanation.aggregate".to_string(),
            &metric.explanation.aggregate,
        ));
        for (candidate, cell) in &metric.explanation.per_candidate {
            cells.push((format!("explanation.{candidate}"), cell));
        }
        for (name, arm) in [
            ("concept.default", &metric.concept.arms.default),
            ("concept.no_expansion", &metric.concept.arms.no_expansion),
            (
                "concept_equivalence.default",
                &metric.concept_equivalence.arms.default,
            ),
            (
                "concept_equivalence.no_expansion",
                &metric.concept_equivalence.arms.no_expansion,
            ),
        ] {
            cells.push((format!("{name}.aggregate"), &arm.score));
            for (candidate, cell) in &arm.per_candidate {
                cells.push((format!("{name}.{candidate}"), cell));
            }
        }
    }

    let mut cells = Vec::new();
    collect(&mut cells, metrics);
    for (label, cell) in cells {
        if cell.status == CellStatus::NotApplicable {
            if cell.attempted != 0 || cell.unavailable != 0 {
                return Err(format!(
                    "{label} is not applicable but has {} attempted and {} unavailable tasks",
                    cell.attempted, cell.unavailable
                ));
            }
            if cell.coverage.numerator != 0 || cell.coverage.denominator != 0 {
                return Err(format!(
                    "{label} is not applicable but has coverage {} of {}",
                    cell.coverage.numerator, cell.coverage.denominator
                ));
            }
            continue;
        }
        let denominator = cell.attempted + cell.unavailable;
        if denominator == 0 {
            return Err(format!(
                "{label} is {:?} but has no attempted or unavailable tasks",
                cell.status
            ));
        }
        if cell.coverage.numerator != cell.attempted {
            return Err(format!(
                "{label} coverage numerator {} is not attempted {}",
                cell.coverage.numerator, cell.attempted
            ));
        }
        if cell.coverage.denominator != denominator {
            return Err(format!(
                "{label} coverage denominator {} is not attempted + unavailable {denominator}",
                cell.coverage.denominator
            ));
        }
        let expected = cell.attempted as f64 / denominator as f64;
        let actual = cell
            .coverage
            .value
            .ok_or_else(|| format!("{label} coverage has no value"))?;
        if (actual - expected).abs() > 1e-9 {
            return Err(format!(
                "{label} coverage value {actual} is not attempted/attempted+unavailable {expected}"
            ));
        }
    }
    Ok(())
}

fn candidate_report<'a>(
    world: &'a BenchmarkWorld,
    id: &str,
) -> Result<&'a benchmark::report::model::CandidateReport, String> {
    report(world)
        .candidates
        .iter()
        .find(|candidate| candidate.id == id)
        .ok_or_else(|| format!("report does not name candidate {id}"))
}

#[then("every candidate records its install command, version, commit, and adapter version")]
fn candidate_identity_recorded(world: &mut BenchmarkWorld) -> Result<(), String> {
    for candidate in &report(world).candidates {
        if candidate.install.trim().is_empty() {
            return Err(format!("{} has no install command", candidate.id));
        }
        if candidate.version.trim().is_empty() {
            return Err(format!("{} has no version", candidate.id));
        }
        if candidate.commit.len() != 40 {
            return Err(format!(
                "{} commit {} is not 40 hex characters",
                candidate.id, candidate.commit
            ));
        }
        if candidate.adapter.trim().is_empty() || candidate.adapter_version == 0 {
            return Err(format!("{} has no adapter version", candidate.id));
        }
    }
    Ok(())
}

#[then(
    "CKB records the pinned scip-typescript, scip-python, and rust-analyzer indexer versions and its doctor output"
)]
fn ckb_indexers_and_doctor(world: &mut BenchmarkWorld) -> Result<(), String> {
    let setup = &candidate_report(world, "fixture-ckb")?.setup;
    for indexer in ["scip-typescript", "scip-python", "rust-analyzer"] {
        match setup.indexers.get(indexer) {
            Some(version) if !version.trim().is_empty() => {}
            _ => return Err(format!("CKB does not record the {indexer} indexer version")),
        }
    }
    match setup.doctor.as_deref() {
        Some(doctor) if !doctor.trim().is_empty() => Ok(()),
        _ => Err("CKB does not record its doctor output".into()),
    }
}

#[then("CKB records Rust language support as Tier 3 in its capability note")]
fn ckb_rust_tier(world: &mut BenchmarkWorld) -> Result<(), String> {
    let setup = &candidate_report(world, "fixture-ckb")?.setup;
    let tier = setup
        .language_support
        .get("rust")
        .ok_or("CKB does not record Rust language support")?;
    if !tier.eq_ignore_ascii_case("tier 3") {
        return Err(format!(
            "CKB records Rust support as {tier}, expected Tier 3"
        ));
    }
    Ok(())
}

#[then(
    "Serena records its Python 3.13 environment and the pyright, typescript-language-server, and rust-analyzer versions"
)]
fn serena_environment(world: &mut BenchmarkWorld) -> Result<(), String> {
    let setup = &candidate_report(world, "fixture-serena")?.setup;
    assert_eq!(
        setup.environment.get("python").map(String::as_str),
        Some("3.13"),
        "Serena Python version"
    );
    for tool in ["pyright", "typescript-language-server", "rust-analyzer"] {
        match setup.environment.get(tool) {
            Some(version) if !version.trim().is_empty() => {}
            _ => return Err(format!("Serena does not record the {tool} version")),
        }
    }
    Ok(())
}

#[then(
    regex = r#"^open-codebase-index records embeddingProvider "([^"]+)" and the recorder base URL$"#
)]
fn ocbi_embedding_provider(world: &mut BenchmarkWorld, captures: &[String]) -> Result<(), String> {
    let [provider] = captures else {
        return Err("embedding provider step needs one capture".into());
    };
    let embedding = candidate_report(world, "fixture-ocbi")?
        .setup
        .embedding
        .as_ref()
        .ok_or("open-codebase-index does not record its embedding configuration")?;
    assert_eq!(&embedding.provider, provider, "embeddingProvider");
    if !embedding.base_url.starts_with("http://127.0.0.1") {
        return Err(format!(
            "open-codebase-index embedding base URL {} is not the loopback recorder",
            embedding.base_url
        ));
    }
    Ok(())
}

#[then(regex = r#"^every candidate copy records git_metadata "([^"]+)"$"#)]
fn candidate_git_metadata(world: &mut BenchmarkWorld, captures: &[String]) -> Result<(), String> {
    let [metadata] = captures else {
        return Err("git metadata step needs one capture".into());
    };
    for candidate in &report(world).candidates {
        assert_eq!(
            &candidate.setup.git_metadata, metadata,
            "{} git_metadata",
            candidate.id
        );
    }
    Ok(())
}

const FIXED_REPOSITORY_COMMIT: &str = "1111111111111111111111111111111111111111";

#[given(regex = r#"^a fixture manifest whose fixture-repo commit is "([^"]+)"$"#)]
fn a_fixture_manifest_with_commit(
    world: &mut BenchmarkWorld,
    captures: &[String],
) -> Result<(), String> {
    let [actual] = captures else {
        return Err("mismatched-commit step needs one capture".into());
    };
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("benchmark crate has no workspace root")?
        .join("benchmark/tests/fixtures/e2e");
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("benchmark crate has no workspace root")?
        .join("target")
        .join("benchmark-fixed-input-mismatch");
    if root.exists() {
        fs::remove_dir_all(&root).map_err(|error| format!("clear {}: {error}", root.display()))?;
    }
    copy_tree(&fixture.join("gold"), &root.join("gold"))?;
    copy_tree(&fixture.join("cassettes"), &root.join("cassettes"))?;
    copy_tree(&fixture.join("repo"), &root.join("repo"))?;
    let manifest = fs::read_to_string(fixture.join("manifest.yaml"))
        .map_err(|error| format!("read fixture manifest: {error}"))?;
    if !manifest.contains(FIXED_REPOSITORY_COMMIT) {
        return Err(format!(
            "fixture manifest does not fix the repository at {FIXED_REPOSITORY_COMMIT}"
        ));
    }
    fs::write(
        root.join("manifest.yaml"),
        manifest.replace(FIXED_REPOSITORY_COMMIT, actual),
    )
    .map_err(|error| format!("write mismatched manifest: {error}"))?;

    world.manifest_path = root.join("manifest.yaml");
    world.out_dir = root.join("out");
    world.mismatch = Some((FIXED_REPOSITORY_COMMIT.to_string(), actual.clone()));
    load_fixture(world)
}

const UNINSTALLABLE_CANDIDATE: &str = r#"
id: fixture-uninstallable
version: fixture-1
commit: "dddddddddddddddddddddddddddddddddddddddd"
install: "fixture adapter"
adapter: fixture
adapter_version: 1
unavailable:
  reason: candidate_uninstallable
  detail: "fixture adapter install failed"
setup:
  git_metadata: present
  indexes:
    - {repository: fixture-repo, status: not_installed, seconds: 0.0, reason: candidate_uninstallable}
"#;

const UNSTARTABLE_CANDIDATE: &str = r#"
id: fixture-unstartable
version: fixture-1
commit: "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"
install: "fixture adapter"
adapter: fixture
adapter_version: 1
unavailable:
  reason: candidate_unstartable
  detail: "fixture adapter did not start"
setup:
  git_metadata: present
  indexes:
    - {repository: fixture-repo, status: not_started, seconds: 0.0, reason: candidate_unstartable}
"#;

#[given("fixture-uninstallable and fixture-unstartable as recorded fixture candidates")]
fn fixture_uninstallable_and_unstartable(world: &mut BenchmarkWorld) -> Result<(), String> {
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("benchmark crate has no workspace root")?
        .join("benchmark/tests/fixtures/e2e");
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("benchmark crate has no workspace root")?
        .join("target")
        .join("benchmark-unstartable-variant");
    if root.exists() {
        fs::remove_dir_all(&root).map_err(|error| format!("clear {}: {error}", root.display()))?;
    }
    copy_tree(&fixture.join("gold"), &root.join("gold"))?;
    copy_tree(&fixture.join("cassettes"), &root.join("cassettes"))?;
    copy_tree(&fixture.join("repo"), &root.join("repo"))?;

    let text = fs::read_to_string(fixture.join("manifest.yaml"))
        .map_err(|error| format!("read fixture manifest: {error}"))?;
    let mut manifest: serde_yaml_ng::Value = serde_yaml_ng::from_str(&text)
        .map_err(|error| format!("parse fixture manifest: {error}"))?;
    for (name, template) in [
        ("fixture-uninstallable", UNINSTALLABLE_CANDIDATE),
        ("fixture-unstartable", UNSTARTABLE_CANDIDATE),
    ] {
        let candidate: serde_yaml_ng::Value = serde_yaml_ng::from_str(template)
            .map_err(|error| format!("parse {name} candidate: {error}"))?;
        manifest
            .get_mut("candidates")
            .and_then(serde_yaml_ng::Value::as_sequence_mut)
            .ok_or("fixture manifest has no candidates sequence")?
            .push(candidate);
    }
    fs::write(
        root.join("manifest.yaml"),
        serde_yaml_ng::to_string(&manifest)
            .map_err(|error| format!("serialize unstartable manifest: {error}"))?,
    )
    .map_err(|error| format!("write unstartable manifest: {error}"))?;

    world.manifest_path = root.join("manifest.yaml");
    world.out_dir = root.join("out");
    load_fixture(world)
}

#[then(regex = r#"^the report marks (\S+) unavailable with the reason "([^"]+)"$"#)]
fn candidate_unavailable_with_reason(
    world: &mut BenchmarkWorld,
    captures: &[String],
) -> Result<(), String> {
    let [candidate, reason] = captures else {
        return Err("candidate unavailability step needs two captures".into());
    };
    let entry = candidate_report(world, candidate)?;
    assert_eq!(
        entry.status,
        benchmark::report::model::CandidateStatus::Unavailable,
        "{candidate} status"
    );
    assert_eq!(
        entry.unavailable_reason.as_deref(),
        Some(reason.as_str()),
        "{candidate} unavailable reason"
    );
    let index = entry
        .setup
        .indexes
        .first()
        .ok_or_else(|| format!("{candidate} records no index outcome"))?;
    assert_eq!(
        index.reason.as_deref(),
        Some(reason.as_str()),
        "{candidate} index reason"
    );
    Ok(())
}

fn candidate_has_measured_cell(report: &Report, candidate: &str) -> bool {
    use benchmark::report::model::{CellStatus, Score};

    fn measured(cell: Option<&Score>) -> bool {
        cell.is_some_and(|cell| cell.status == CellStatus::Measured)
    }

    let metrics = &report.metrics;
    measured(metrics.typo.per_candidate.get(candidate))
        || measured(metrics.typo.recall_at_5_per_candidate.get(candidate))
        || measured(metrics.natural_language.per_candidate.get(candidate))
        || measured(metrics.explanation.per_candidate.get(candidate))
        || [&metrics.concept, &metrics.concept_equivalence]
            .iter()
            .any(|metric| {
                [&metric.arms.default, &metric.arms.no_expansion]
                    .iter()
                    .any(|arm| measured(arm.per_candidate.get(candidate)))
            })
}

#[then("the run completes and reports measured values for fixture-ocbi and fixture-ckb")]
fn run_completes_with_measured_candidates(world: &mut BenchmarkWorld) -> Result<(), String> {
    let report = world
        .report
        .as_ref()
        .ok_or("the run did not complete with a report")?;
    for candidate in ["fixture-ocbi", "fixture-ckb"] {
        if !candidate_has_measured_cell(report, candidate) {
            return Err(format!(
                "the completed run reports no measured value for {candidate}"
            ));
        }
    }
    Ok(())
}

#[then(regex = r"^the report lists (\S+) and (\S+) in the capability gaps$")]
fn unavailable_capability_gaps(
    world: &mut BenchmarkWorld,
    captures: &[String],
) -> Result<(), String> {
    let [first, second] = captures else {
        return Err("capability gap step needs two captures".into());
    };
    for (candidate, reason) in [
        (first.as_str(), "candidate_uninstallable"),
        (second.as_str(), "candidate_unstartable"),
    ] {
        let gap = report(world)
            .gaps
            .capability_gaps
            .iter()
            .find(|gap| gap.candidate == candidate)
            .ok_or_else(|| format!("report does not list {candidate} in its capability gaps"))?;
        assert_eq!(gap.reason, reason, "capability gap reason");
    }
    Ok(())
}

#[then("the error names fixture-repo and the mismatched expected and actual commits")]
fn the_error_names_mismatched_commits(world: &mut BenchmarkWorld) -> Result<(), String> {
    let failure = world
        .run_failure
        .as_ref()
        .ok_or("the run did not fail; no error to inspect")?;
    let (expected, actual) = world
        .mismatch
        .as_ref()
        .ok_or("the mismatch fixture did not record its commits")?;
    if !failure.message.contains("fixture-repo") {
        return Err(format!(
            "error does not name fixture-repo: {}",
            failure.message
        ));
    }
    for (commit, label) in [(expected, "expected"), (actual, "actual")] {
        if !failure.message.contains(commit) {
            return Err(format!(
                "error does not name the {label} commit {commit}: {}",
                failure.message
            ));
        }
    }
    Ok(())
}

#[then("every candidate records a per-repository index status and duration")]
fn candidate_index_status(world: &mut BenchmarkWorld) -> Result<(), String> {
    let report = report(world);
    let repositories: Vec<&str> = report
        .inputs
        .repositories
        .iter()
        .map(|repository| repository.id.as_str())
        .collect();
    for candidate in &report.candidates {
        if candidate.setup.indexes.is_empty() {
            return Err(format!("{} records no index status", candidate.id));
        }
        for index in &candidate.setup.indexes {
            if index.status.trim().is_empty() {
                return Err(format!(
                    "{} records no index status for {}",
                    candidate.id, index.repository
                ));
            }
            if index.seconds < 0.0 {
                return Err(format!(
                    "{} records a negative index duration for {}",
                    candidate.id, index.repository
                ));
            }
            if !repositories.contains(&index.repository.as_str()) {
                return Err(format!(
                    "{} records an index for unknown repository {}",
                    candidate.id, index.repository
                ));
            }
        }
    }
    Ok(())
}

#[when("the maintainer reads the report")]
fn the_maintainer_reads_the_report(world: &mut BenchmarkWorld) -> Result<(), String> {
    world.report = world.first_report.clone();
    if world.report.is_none() {
        return Err("no completed run to read".into());
    }
    Ok(())
}

#[then(
    "the report states the replay command for the fixture manifest with mode replay and its output path"
)]
fn reproduction_command(world: &mut BenchmarkWorld) -> Result<(), String> {
    let reproduction = &report(world).reproduction;
    let manifest = world.manifest_path.display().to_string();
    let out = world.out_dir.display().to_string();
    let command = &reproduction.command;
    for (fragment, label) in [
        (manifest.as_str(), "fixture manifest path"),
        ("--mode replay", "replay mode"),
        (out.as_str(), "output path"),
    ] {
        if !command.contains(fragment) {
            return Err(format!(
                "reproduction command does not state the {label}: {command}"
            ));
        }
    }
    Ok(())
}

#[then("the report states the toolchain environment prefix used by the command")]
fn reproduction_toolchain_prefix(world: &mut BenchmarkWorld) -> Result<(), String> {
    let prefix = &report(world).reproduction.toolchain_prefix;
    for required in [
        "RUSTUP_TOOLCHAIN=stable-x86_64-unknown-linux-gnu",
        "LD_LIBRARY_PATH",
        "ZLIB_LIBRARY_PATH",
    ] {
        if !prefix.contains(required) {
            return Err(format!(
                "toolchain prefix does not state {required}: {prefix}"
            ));
        }
    }
    for host_specific in ["/nix/store", "/usr/lib"] {
        if prefix.contains(host_specific) {
            return Err(format!(
                "toolchain prefix embeds the host-specific path {host_specific}: {prefix}"
            ));
        }
    }
    Ok(())
}

#[then("the report states the manifest, gold, model-cassette, and result-cassette fingerprints")]
fn reproduction_fingerprints(world: &mut BenchmarkWorld) -> Result<(), String> {
    let manifest = world
        .manifest
        .as_ref()
        .ok_or("fixture manifest was not loaded")?;
    let reproduction = &report(world).reproduction;
    let manifest_fingerprint = manifest
        .fingerprint()
        .map_err(|error| format!("compute manifest fingerprint: {error}"))?;
    assert_eq!(
        reproduction.manifest_fingerprint, manifest_fingerprint,
        "manifest fingerprint"
    );
    let expected_gold: std::collections::BTreeMap<String, String> = manifest
        .gold
        .iter()
        .map(|set| (set.metric.clone(), set.fingerprint.clone()))
        .collect();
    assert_eq!(
        reproduction.gold_fingerprints, expected_gold,
        "gold fingerprints"
    );
    for metric in ["typo", "natural_language", "explanation", "concept"] {
        if !reproduction.gold_fingerprints.contains_key(metric) {
            return Err(format!("report has no {metric} gold fingerprint"));
        }
    }
    assert_eq!(
        reproduction.model_cassette_fingerprint, manifest.cassettes.model.fingerprint,
        "model cassette fingerprint"
    );
    assert_eq!(
        reproduction.result_cassette_fingerprint, manifest.cassettes.results.fingerprint,
        "result cassette fingerprint"
    );
    Ok(())
}

#[then(
    regex = r#"^the (\S+) gold set is version (\d+) with (\d+) tasks and fingerprint "([^"]+)"$"#
)]
fn gold_set_identity(world: &mut BenchmarkWorld, captures: &[String]) -> Result<(), String> {
    let [label, version, tasks, fingerprint] = captures else {
        return Err("gold set identity step needs four captures".into());
    };
    let metric = match label.as_str() {
        "typo" => "typo",
        "natural-language" => "natural_language",
        "explanation" => "explanation",
        "concept" => "concept",
        "concept-equivalence" => "concept_equivalence",
        other => return Err(format!("unknown gold set label {other}")),
    };
    let set = report(world)
        .inputs
        .gold
        .iter()
        .find(|set| set.metric == metric)
        .ok_or_else(|| format!("report does not name the {metric} gold set"))?;
    let expected_version: u32 = version
        .parse()
        .map_err(|error| format!("parse gold version: {error}"))?;
    let expected_tasks: u64 = tasks
        .parse()
        .map_err(|error| format!("parse gold task count: {error}"))?;
    assert_eq!(set.version, expected_version, "{metric} gold version");
    assert_eq!(set.tasks, expected_tasks, "{metric} gold task count");
    assert_eq!(&set.fingerprint, fingerprint, "{metric} gold fingerprint");
    Ok(())
}

#[then("its gold sets name the same locations")]
fn synthetic_gold_locations_match(world: &mut BenchmarkWorld) -> Result<(), String> {
    let first = world
        .synthetic_first
        .as_ref()
        .ok_or("the synthetic monorepo was not generated before regeneration")?;
    let second = synthetic_repo(world);
    if first.gold != second.gold {
        return Err("regenerated synthetic gold sets changed".into());
    }
    for task in &second.gold {
        let file = second
            .files
            .iter()
            .find(|file| file.path == task.expected.path)
            .ok_or_else(|| format!("gold task {} names an unknown file", task.id))?;
        if task.expected.start_line == 0 || u64::from(task.expected.start_line) > file.code_lines {
            return Err(format!(
                "gold task {} names line {} outside {}",
                task.id, task.expected.start_line, file.path
            ));
        }
        if !task.observed_line.contains(&task.expected.symbol) {
            return Err(format!(
                "gold task {} expected symbol {} but the line is {:?}",
                task.id, task.expected.symbol, task.observed_line
            ));
        }
    }
    Ok(())
}

const TIMEOUT_CANDIDATE_TEMPLATE: &str = r#"
id: fixture-timeout
version: fixture-1
commit: "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"
install: "fixture adapter"
adapter: fixture
adapter_version: 1
index_deadline_seconds: DEADLINE_SECONDS
capabilities:
  typo: {status: measured}
  natural_language: {status: measured}
  explanation: {status: measured}
  concept: {status: measured}
  concept_equivalence: {status: measured}
  expansion: {status: measured}
setup:
  git_metadata: present
  indexes:
    - {repository: REPOSITORY, status: indexed, seconds: 5.2}
"#;

fn variant_fixture_root(name: &str) -> Result<PathBuf, String> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("benchmark crate has no workspace root")?
        .join("target")
        .join(name);
    if root.exists() {
        fs::remove_dir_all(&root).map_err(|error| format!("clear {}: {error}", root.display()))?;
    }
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/e2e");
    copy_tree(&fixture.join("gold"), &root.join("gold"))?;
    copy_tree(&fixture.join("cassettes"), &root.join("cassettes"))?;
    copy_tree(&fixture.join("repo"), &root.join("repo"))?;
    Ok(root)
}

fn append_candidate(
    manifest: &mut serde_yaml_ng::Value,
    candidate: serde_yaml_ng::Value,
) -> Result<(), String> {
    manifest
        .get_mut("candidates")
        .and_then(serde_yaml_ng::Value::as_sequence_mut)
        .ok_or("fixture manifest has no candidates sequence")?
        .push(candidate);
    Ok(())
}

fn read_fixture_manifest() -> Result<serde_yaml_ng::Value, String> {
    let fixture =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/e2e/manifest.yaml");
    let text =
        fs::read_to_string(&fixture).map_err(|error| format!("read fixture manifest: {error}"))?;
    serde_yaml_ng::from_str(&text).map_err(|error| format!("parse fixture manifest: {error}"))
}

fn refresh_gold_fingerprint(
    manifest: &mut serde_yaml_ng::Value,
    root: &Path,
    metric: &str,
) -> Result<(), String> {
    let set = manifest
        .get_mut("gold")
        .and_then(serde_yaml_ng::Value::as_sequence_mut)
        .ok_or("fixture manifest has no gold sequence")?
        .iter_mut()
        .find(|set| set["metric"] == metric)
        .ok_or_else(|| format!("fixture manifest has no {metric} gold entry"))?;
    let relative = set["path"]
        .as_str()
        .ok_or("gold entry has no path")?
        .to_string();
    let fingerprint = benchmark::manifest::file_fingerprint(&root.join(relative))
        .map_err(|error| format!("fingerprint variant gold: {error:#}"))?;
    set["fingerprint"] = serde_yaml_ng::Value::String(fingerprint);
    Ok(())
}

fn refresh_cassette_fingerprint(
    manifest: &mut serde_yaml_ng::Value,
    root: &Path,
    layer: &str,
) -> Result<(), String> {
    let relative = manifest["cassettes"][layer]["path"]
        .as_str()
        .ok_or_else(|| format!("manifest has no {layer} cassette path"))?
        .to_string();
    let fingerprint = benchmark::record::cassette::fingerprint(&root.join(relative))
        .map_err(|error| format!("fingerprint variant {layer} cassette: {error:#}"))?;
    manifest["cassettes"][layer]["fingerprint"] = serde_yaml_ng::Value::String(fingerprint);
    Ok(())
}

fn write_variant_manifest(root: &Path, manifest: &serde_yaml_ng::Value) -> Result<PathBuf, String> {
    let path = root.join("manifest.yaml");
    fs::write(
        &path,
        serde_yaml_ng::to_string(manifest)
            .map_err(|error| format!("serialize variant manifest: {error}"))?,
    )
    .map_err(|error| format!("write variant manifest: {error}"))?;
    Ok(path)
}

fn rewrite_first_typo_task(
    root: &Path,
    mutate: impl FnOnce(&mut serde_json::Value),
) -> Result<(), String> {
    let path = root.join("gold/typo.jsonl");
    let text = fs::read_to_string(&path)
        .map_err(|error| format!("read variant gold {}: {error}", path.display()))?;
    let mut lines = text.lines();
    let first = lines
        .next()
        .ok_or("fixture typo gold is empty")?;
    let mut task: serde_json::Value =
        serde_json::from_str(first).map_err(|error| format!("parse variant gold: {error}"))?;
    mutate(&mut task);
    let mut out = serde_json::to_string(&task)
        .map_err(|error| format!("serialize variant gold: {error}"))?;
    out.push('\n');
    for line in lines {
        out.push_str(line);
        out.push('\n');
    }
    fs::write(&path, out).map_err(|error| format!("write variant gold {}: {error}", path.display()))
}

#[given("a fixture gold task whose path is absent from the fixed checkout")]
fn gold_path_absent(world: &mut BenchmarkWorld) -> Result<(), String> {
    let root = variant_fixture_root("benchmark-gold-path-absent")?;
    rewrite_first_typo_task(&root, |task| {
        task["expected"][0]["path"] = serde_json::Value::String("src/absent.py".to_string());
    })?;
    let mut manifest = read_fixture_manifest()?;
    refresh_gold_fingerprint(&mut manifest, &root, "typo")?;
    world.manifest_path = write_variant_manifest(&root, &manifest)?;
    world.out_dir = root.join("out");
    load_fixture(world)
}

#[given("a fixture gold task whose line range exceeds the file")]
fn gold_line_range_exceeds(world: &mut BenchmarkWorld) -> Result<(), String> {
    let root = variant_fixture_root("benchmark-gold-line-range")?;
    rewrite_first_typo_task(&root, |task| {
        task["expected"][0]["end_line"] = serde_json::Value::from(1000);
    })?;
    let mut manifest = read_fixture_manifest()?;
    refresh_gold_fingerprint(&mut manifest, &root, "typo")?;
    world.manifest_path = write_variant_manifest(&root, &manifest)?;
    world.out_dir = root.join("out");
    load_fixture(world)
}

#[then(regex = r#"^the error names the gold task "([^"]+)" and the missing path "([^"]+)"$"#)]
fn error_names_missing_gold_path(
    world: &mut BenchmarkWorld,
    captures: &[String],
) -> Result<(), String> {
    let [task, path] = captures else {
        return Err("missing-gold-path step needs two captures".into());
    };
    let message = &world
        .run_failure
        .as_ref()
        .ok_or("the run did not fail; a report was produced")?
        .message;
    for expected in [task.as_str(), path.as_str(), "does not exist"] {
        if !message.contains(expected) {
            return Err(format!("error does not contain `{expected}`: {message}"));
        }
    }
    Ok(())
}

#[then(regex = r#"^the error names the gold task "([^"]+)" and the out-of-range end line$"#)]
fn error_names_out_of_range_line(
    world: &mut BenchmarkWorld,
    captures: &[String],
) -> Result<(), String> {
    let [task] = captures else {
        return Err("out-of-range-gold step needs one capture".into());
    };
    let message = &world
        .run_failure
        .as_ref()
        .ok_or("the run did not fail; a report was produced")?
        .message;
    for expected in [task.as_str(), "1000", "exceeds"] {
        if !message.contains(expected) {
            return Err(format!("error does not contain `{expected}`: {message}"));
        }
    }
    Ok(())
}

#[given(regex = r"^fixture-timeout with an index deadline of (\d+) seconds? on (\S+)$")]
fn fixture_timeout(world: &mut BenchmarkWorld, captures: &[String]) -> Result<(), String> {
    let [deadline, repository] = captures else {
        return Err("index-deadline step needs two captures".into());
    };
    let root = variant_fixture_root("benchmark-index-timeout")?;
    let mut manifest = read_fixture_manifest()?;
    let candidate: serde_yaml_ng::Value = serde_yaml_ng::from_str(
        &TIMEOUT_CANDIDATE_TEMPLATE
            .replace("DEADLINE_SECONDS", deadline)
            .replace("REPOSITORY", repository),
    )
    .map_err(|error| format!("parse fixture-timeout candidate: {error}"))?;
    append_candidate(&mut manifest, candidate)?;
    let path = write_variant_manifest(&root, &manifest)?;

    world.manifest_path = path;
    world.out_dir = root.join("out");
    load_fixture(world)
}

#[then(regex = r#"^the report marks (\S+) unavailable on (\S+) with the reason "([^"]+)"$"#)]
fn candidate_unavailable_on_repository(
    world: &mut BenchmarkWorld,
    captures: &[String],
) -> Result<(), String> {
    let [candidate, repository, reason] = captures else {
        return Err("index unavailability step needs three captures".into());
    };
    let entry = candidate_report(world, candidate)?;
    let index = entry
        .setup
        .indexes
        .iter()
        .find(|index| &index.repository == repository)
        .ok_or_else(|| format!("{candidate} records no index for {repository}"))?;
    assert_eq!(index.status, "timed_out", "{candidate} index status");
    assert_eq!(
        index.reason.as_deref(),
        Some(reason.as_str()),
        "{candidate} index reason"
    );
    Ok(())
}

#[then(regex = r"^the report records the applied index deadline of (\d+) seconds?$")]
fn report_records_applied_deadline(
    world: &mut BenchmarkWorld,
    captures: &[String],
) -> Result<(), String> {
    let [deadline] = captures else {
        return Err("applied-deadline step needs one capture".into());
    };
    let expected: u64 = deadline
        .parse()
        .map_err(|error| format!("parse applied deadline: {error}"))?;
    let index = candidate_report(world, "fixture-timeout")?
        .setup
        .indexes
        .first()
        .ok_or("fixture-timeout records no index outcome")?;
    assert_eq!(
        index.deadline_seconds,
        Some(expected),
        "applied index deadline"
    );
    let markdown = fs::read_to_string(world.out_dir.join("report.md"))
        .map_err(|error| format!("read report.md: {error}"))?;
    let rendered = format!("{expected}s deadline");
    if !markdown.contains(&rendered) {
        return Err(format!(
            "report.md does not state the applied deadline `{rendered}`"
        ));
    }
    Ok(())
}

#[then("the report records the elapsed index time")]
fn report_records_elapsed_index_time(world: &mut BenchmarkWorld) -> Result<(), String> {
    let deadline = world
        .manifest
        .as_ref()
        .and_then(|manifest| manifest.candidate("fixture-timeout"))
        .and_then(|candidate| candidate.index_deadline_seconds)
        .ok_or("fixture-timeout has no index deadline")?;
    let index = candidate_report(world, "fixture-timeout")?
        .setup
        .indexes
        .first()
        .ok_or("fixture-timeout records no index outcome")?;
    if index.seconds <= deadline as f64 {
        return Err(format!(
            "elapsed index time {} does not exceed the {deadline}s deadline",
            index.seconds
        ));
    }
    Ok(())
}

#[then(regex = r"^every metric for (\S+) on (\S+) is unavailable and never zero$")]
fn every_metric_unavailable_for_candidate(
    world: &mut BenchmarkWorld,
    captures: &[String],
) -> Result<(), String> {
    let [candidate, repository] = captures else {
        return Err("all-metrics unavailable step needs two captures".into());
    };
    let report = report(world);
    let named = report
        .candidates
        .iter()
        .find(|entry| &entry.id == candidate)
        .ok_or_else(|| format!("report does not name {candidate}"))?;
    let index = named
        .setup
        .indexes
        .iter()
        .find(|index| &index.repository == repository)
        .ok_or_else(|| format!("{candidate} records no index for {repository}"))?;
    if index.reason.as_deref() != Some("index_timeout") {
        return Err(format!("{candidate} index is not timed out"));
    }

    use benchmark::report::model::{CellStatus, Score};
    let metrics = &report.metrics;
    let cells: Vec<(&str, Option<&Score>)> = vec![
        ("typo @1", metrics.typo.per_candidate.get(candidate)),
        (
            "typo @5",
            metrics.typo.recall_at_5_per_candidate.get(candidate),
        ),
        (
            "natural-language",
            metrics.natural_language.per_candidate.get(candidate),
        ),
        (
            "explanation",
            metrics.explanation.per_candidate.get(candidate),
        ),
        (
            "concept default",
            metrics.concept.arms.default.per_candidate.get(candidate),
        ),
        (
            "concept no-expansion",
            metrics
                .concept
                .arms
                .no_expansion
                .per_candidate
                .get(candidate),
        ),
        (
            "concept-equivalence default",
            metrics
                .concept_equivalence
                .arms
                .default
                .per_candidate
                .get(candidate),
        ),
        (
            "concept-equivalence no-expansion",
            metrics
                .concept_equivalence
                .arms
                .no_expansion
                .per_candidate
                .get(candidate),
        ),
    ];
    for (label, cell) in cells {
        let cell = cell.ok_or_else(|| format!("{candidate} has no {label} cell"))?;
        assert_eq!(
            cell.status,
            CellStatus::Unavailable,
            "{candidate} {label} status"
        );
        assert_eq!(cell.value, None, "{candidate} {label} must not be zero");
        assert_eq!(
            cell.reason.as_deref(),
            Some("index_timeout"),
            "{candidate} {label} reason"
        );
    }
    Ok(())
}

#[then("the run continues with the other candidates")]
fn run_continues_with_other_candidates(world: &mut BenchmarkWorld) -> Result<(), String> {
    let report = world
        .report
        .as_ref()
        .ok_or("the run did not complete with a report")?;
    for candidate in ["fixture-ocbi", "fixture-ckb"] {
        if !candidate_has_measured_cell(report, candidate) {
            return Err(format!("the run reports no measured value for {candidate}"));
        }
    }
    Ok(())
}

#[given("a natural-language gold set of four tasks where 1 query returns an error")]
fn natural_language_query_error(world: &mut BenchmarkWorld) -> Result<(), String> {
    let root = variant_fixture_root("benchmark-query-error")?;
    let path = root.join("cassettes/results/results.jsonl");
    let text =
        fs::read_to_string(&path).map_err(|error| format!("read result cassette: {error}"))?;
    let mut rewritten = String::new();
    let mut replaced = false;
    for line in text.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let mut value: serde_json::Value = serde_json::from_str(line)
            .map_err(|error| format!("parse result cassette line: {error}"))?;
        let is_errored_task = value["candidate"] == "fixture-ocbi"
            && value["metric"] == "natural_language"
            && value["task"] == "nl-3";
        if is_errored_task {
            value["error"] = serde_json::json!("query failed");
            replaced = true;
        }
        rewritten.push_str(
            &serde_json::to_string(&value)
                .map_err(|error| format!("serialize result cassette line: {error}"))?,
        );
        rewritten.push('\n');
    }
    if !replaced {
        return Err("fixture result cassette has no fixture-ocbi nl-3 entry".into());
    }
    fs::write(&path, rewritten).map_err(|error| format!("write errored cassette: {error}"))?;
    let mut manifest = read_fixture_manifest()?;
    refresh_cassette_fingerprint(&mut manifest, &root, "results")?;
    world.manifest_path = write_variant_manifest(&root, &manifest)?;
    world.out_dir = root.join("out");
    load_fixture(world)
}

#[then("the report states a query error total of 1 separately from the recall value")]
fn query_error_total(world: &mut BenchmarkWorld) -> Result<(), String> {
    let metric = &report(world).metrics.natural_language;
    assert_eq!(metric.query_errors, 1, "NL query error total");
    assert_eq!(
        metric.query_errors_per_candidate.get("fixture-ocbi"),
        Some(&1),
        "fixture-ocbi query error total"
    );
    assert_eq!(
        metric.aggregate.value,
        Some(0.5),
        "the recall value stays separate from the error total"
    );
    let markdown = fs::read_to_string(world.out_dir.join("report.md"))
        .map_err(|error| format!("read report.md: {error}"))?;
    if !markdown.contains("NL query errors: 1") {
        return Err("report.md does not state the query error total".into());
    }
    Ok(())
}

#[then("the errored query is counted as a miss and not as an unavailable task")]
fn errored_query_is_a_miss(world: &mut BenchmarkWorld) -> Result<(), String> {
    use benchmark::report::model::CellStatus;

    let cell = report(world)
        .metrics
        .natural_language
        .per_candidate
        .get("fixture-ocbi")
        .ok_or("report has no fixture-ocbi natural-language cell")?;
    assert_eq!(cell.status, CellStatus::Measured, "NL cell status");
    assert_eq!(cell.attempted, 4, "every task was attempted");
    assert_eq!(cell.unavailable, 0, "the errored query is not unavailable");
    assert_eq!(cell.numerator, 2, "errored query is a miss, not a hit");
    assert_eq!(cell.value, Some(0.5), "recall after the error");
    Ok(())
}

#[given("a fixture run where every typo candidate is unavailable")]
fn every_typo_candidate_unavailable(world: &mut BenchmarkWorld) -> Result<(), String> {
    let root = variant_fixture_root("benchmark-no-typo-candidate")?;
    let mut manifest = read_fixture_manifest()?;
    let unavailable: serde_yaml_ng::Value =
        serde_yaml_ng::from_str("{status: unavailable, reason: no_symbol_search}")
            .map_err(|error| format!("parse typo capability: {error}"))?;
    for candidate in manifest
        .get_mut("candidates")
        .and_then(serde_yaml_ng::Value::as_sequence_mut)
        .ok_or("fixture manifest has no candidates sequence")?
    {
        let capabilities = candidate
            .get_mut("capabilities")
            .and_then(serde_yaml_ng::Value::as_mapping_mut)
            .ok_or("fixture candidate has no capabilities mapping")?;
        capabilities.insert(
            serde_yaml_ng::Value::String("typo".to_string()),
            unavailable.clone(),
        );
    }
    let path = write_variant_manifest(&root, &manifest)?;
    world.manifest_path = path;
    world.out_dir = root.join("out");
    load_fixture(world)
}

#[then(regex = r#"^the typo to-(best|target) gap has status "([^"]+)" with the reason "([^"]+)"$"#)]
fn typo_gap_status(world: &mut BenchmarkWorld, captures: &[String]) -> Result<(), String> {
    use benchmark::report::model::CellStatus;

    let [kind, status, reason] = captures else {
        return Err("typo gap status step needs three captures".into());
    };
    let gaps = &report(world).gaps;
    let gap = match kind.as_str() {
        "best" => gaps.to_best.get("typo_recall_at_1"),
        "target" => gaps.to_target.get("typo_recall_at_1"),
        other => return Err(format!("unknown gap kind {other}")),
    }
    .ok_or_else(|| format!("report has no typo to-{kind} gap"))?;
    let expected_status = match status.as_str() {
        "unavailable" => CellStatus::Unavailable,
        "measured" => CellStatus::Measured,
        "not_applicable" => CellStatus::NotApplicable,
        other => return Err(format!("unknown gap status {other}")),
    };
    assert_eq!(gap.status, expected_status, "gap status");
    assert_eq!(gap.reason.as_deref(), Some(reason.as_str()), "gap reason");
    if expected_status == CellStatus::Unavailable {
        assert_eq!(gap.value, None, "typo to-{kind} gap value");
    }
    Ok(())
}

#[then("neither gap value is rendered as zero")]
fn gaps_not_rendered_as_zero(world: &mut BenchmarkWorld) -> Result<(), String> {
    let gaps = &report(world).gaps;
    for (label, gap) in [
        ("to-best", gaps.to_best.get("typo_recall_at_1")),
        ("to-target", gaps.to_target.get("typo_recall_at_1")),
    ] {
        let gap = gap.ok_or_else(|| format!("report has no typo {label} gap"))?;
        if gap.value.is_some() {
            return Err(format!(
                "typo {label} gap carries a numeric value without a measured candidate"
            ));
        }
    }
    let json: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(world.out_dir.join("report.json"))
            .map_err(|error| format!("read report.json: {error}"))?,
    )
    .map_err(|error| format!("parse report.json: {error}"))?;
    for path in [
        &["gaps", "to_best", "typo_recall_at_1", "value"][..],
        &["gaps", "to_target", "typo_recall_at_1", "value"][..],
    ] {
        let mut value = &json;
        for key in path {
            value = value
                .get(key)
                .ok_or_else(|| format!("report.json has no {}", path.join(".")))?;
        }
        if !value.is_null() {
            return Err(format!(
                "report.json renders {} as {value}, not null",
                path.join(".")
            ));
        }
    }
    let markdown = fs::read_to_string(world.out_dir.join("report.md"))
        .map_err(|error| format!("read report.md: {error}"))?;
    for expected in [
        "typo_recall_at_1 to proposed target: unavailable (no_measured_candidate)",
        "typo_recall_at_1 best measured: unavailable (no_measured_candidate)",
    ] {
        if !markdown.contains(expected) {
            return Err(format!("report.md does not render `{expected}`"));
        }
    }
    Ok(())
}

#[then("the report is still published as complete")]
fn report_published_as_complete(world: &mut BenchmarkWorld) -> Result<(), String> {
    use benchmark::report::model::CellStatus;

    let json_path = world.out_dir.join("report.json");
    let md_path = world.out_dir.join("report.md");
    for path in [&json_path, &md_path] {
        if !path.exists() {
            return Err(format!("report was not published at {}", path.display()));
        }
    }
    let json: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(&json_path).map_err(|error| format!("read report.json: {error}"))?,
    )
    .map_err(|error| format!("parse report.json: {error}"))?;
    if json["schema_version"].as_u64() != Some(u64::from(benchmark::report::model::SCHEMA_VERSION))
    {
        return Err("report.json is not a complete schema-versioned report".into());
    }
    if json["metrics"]["typo"]["aggregate"]["status"] != "unavailable" {
        return Err("typo aggregate is not marked unavailable".into());
    }
    let typo = &report(world).metrics.typo.aggregate;
    assert_eq!(
        typo.status,
        CellStatus::Unavailable,
        "typo aggregate status"
    );
    Ok(())
}

#[given("fixture-serena exposes no natural-language search operation")]
fn serena_has_no_natural_language_search(world: &mut BenchmarkWorld) -> Result<(), String> {
    use benchmark::manifest::CapabilityStatus;

    load_fixture(world)?;
    let manifest = world
        .manifest
        .as_ref()
        .ok_or("fixture manifest was not loaded")?;
    let serena = manifest
        .candidate("fixture-serena")
        .ok_or("fixture manifest does not name fixture-serena")?;
    let natural_language = serena
        .capabilities
        .get("natural_language")
        .ok_or("fixture-serena declares no natural-language capability")?;
    assert_eq!(
        natural_language.status,
        CapabilityStatus::Unavailable,
        "fixture-serena natural-language status"
    );
    assert_eq!(
        natural_language.reason.as_deref(),
        Some("no_natural_language_search"),
        "fixture-serena natural-language reason"
    );
    let proxy = serena
        .capabilities
        .get("regex_pattern_search_proxy")
        .ok_or("fixture-serena declares no regex proxy arm")?;
    assert_eq!(
        proxy.status,
        CapabilityStatus::Measured,
        "fixture-serena regex proxy capability"
    );
    Ok(())
}

#[then(
    regex = r#"^fixture-serena's natural-language cells are unavailable with the reason "([^"]+)"$"#
)]
fn serena_natural_language_unavailable(
    world: &mut BenchmarkWorld,
    captures: &[String],
) -> Result<(), String> {
    use benchmark::report::model::CellStatus;

    let [reason] = captures else {
        return Err("serena NL step needs one capture".into());
    };
    let cell = report(world)
        .metrics
        .natural_language
        .per_candidate
        .get("fixture-serena")
        .ok_or("report has no fixture-serena natural-language cell")?;
    assert_eq!(cell.status, CellStatus::Unavailable, "NL status");
    assert_eq!(cell.value, None, "unavailable NL cell must not be zero");
    assert_eq!(cell.reason.as_deref(), Some(reason.as_str()), "NL reason");
    Ok(())
}

#[then(
    regex = r"^the natural-language recall at rank five remains ([\d.]+) with (\d+) of (\d+) tasks, excluding fixture-serena$"
)]
fn natural_language_recall_excluding_serena(
    world: &mut BenchmarkWorld,
    captures: &[String],
) -> Result<(), String> {
    let [value, numerator, denominator] = captures else {
        return Err("NL recall excluding serena step needs three captures".into());
    };
    let metric = &report(world).metrics.natural_language;
    check_value(
        metric
            .aggregate
            .value
            .ok_or("NL recall has no measured value")?,
        value,
        "NL recall @5",
    )?;
    check_counts(
        metric.aggregate.numerator,
        metric.aggregate.denominator,
        numerator,
        denominator,
        "NL recall @5",
    )?;
    assert_eq!(
        metric.aggregate.attempted, 4,
        "only the measured candidate contributes to the NL aggregate"
    );
    Ok(())
}

#[then(
    regex = r#"^a measured regex arm is labelled "([^"]+)" and excluded from natural-language recall and mean reciprocal rank$"#
)]
fn regex_proxy_arm_excluded(world: &mut BenchmarkWorld, captures: &[String]) -> Result<(), String> {
    use benchmark::report::model::CellStatus;

    let [label] = captures else {
        return Err("regex proxy step needs one capture".into());
    };
    let metric = &report(world).metrics.natural_language;
    let arm = metric
        .proxy_arms
        .get(label)
        .ok_or_else(|| format!("report has no proxy arm labelled {label}"))?;
    assert_eq!(arm.score.status, CellStatus::Measured, "proxy arm status");
    check_value(
        arm.score
            .value
            .ok_or("regex proxy arm has no measured value")?,
        "0.50",
        "regex proxy arm",
    )?;
    assert!(
        arm.per_candidate
            .get("fixture-serena")
            .is_some_and(|cell| cell.status == CellStatus::Measured),
        "the regex proxy is measured on fixture-serena"
    );

    check_value(
        metric
            .aggregate
            .value
            .ok_or("NL recall has no measured value")?,
        "0.50",
        "NL recall excluding the proxy",
    )?;
    assert_eq!(metric.aggregate.numerator, 2, "NL recall numerator");
    assert_eq!(metric.aggregate.denominator, 4, "NL recall denominator");
    check_value(metric.mrr_at_10.value, "0.30", "NL MRR excluding the proxy")?;
    assert_eq!(
        metric.mrr_at_10.attempted, 4,
        "the proxy arm is excluded from MRR"
    );

    let markdown = fs::read_to_string(world.out_dir.join("report.md"))
        .map_err(|error| format!("read report.md: {error}"))?;
    let expected = format!("NL `{label}` arm (proxy only; excluded from NL recall and MRR)");
    if !markdown.contains(&expected) {
        return Err(format!(
            "report.md does not label the regex arm as the proxy: {expected}"
        ));
    }
    Ok(())
}

fn report_json(world: &BenchmarkWorld) -> Result<serde_json::Value, String> {
    let path = world.out_dir.join("report.json");
    let text = fs::read_to_string(&path)
        .map_err(|error| format!("read report {}: {error}", path.display()))?;
    serde_json::from_str(&text)
        .map_err(|error| format!("parse report {}: {error}", path.display()))
}

const EXTRA_REPOSITORY: &str = r#"
id: fixture-extra
url: "fixture://fixture-extra"
revision: extra-1
commit: "2222222222222222222222222222222222222222"
language: python
checkout: repo-extra
loc:
  language_code_lines: 40
  total_code_lines: 40
  tool: fixture
  command: "wc -l benchmark/tests/fixtures/e2e/repo-extra/src/*.py"
  exclusions: []
"#;

fn extra_repository_source() -> String {
    let mut source = String::new();
    for line in 1..=60 {
        source.push_str(&format!("def extra_{line}():\n    return {line}\n"));
    }
    source
}

fn extra_typo_gold() -> String {
    let mut gold = String::new();
    for (task, line) in [1u32, 11, 21, 31, 41].iter().enumerate() {
        gold.push_str(&format!(
            "{{\"id\":\"extra-{}\",\"query\":\"extra helper {}\",\"expected\":[{{\"path\":\"src/extra.py\",\"start_line\":{line},\"end_line\":{line},\"symbol\":\"extra_{line}\"}}]}}\n",
            task + 1,
            task + 1
        ));
    }
    gold
}

fn extra_results() -> Vec<serde_json::Value> {
    let mut entries = Vec::new();
    for task in 1..=5u32 {
        let results = match task {
            1 => serde_json::json!([{"path": "src/extra.py", "start_line": 1, "end_line": 1, "symbol": "extra_1", "kind": "function", "resolution": "symbol"}]),
            2 => serde_json::json!([{"path": "src/extra.py", "start_line": 11, "end_line": 11, "symbol": "extra_11", "kind": "function", "resolution": "symbol"}]),
            _ => serde_json::json!([]),
        };
        entries.push(serde_json::json!({
            "candidate": "fixture-ocbi",
            "candidate_version": "fixture-1",
            "candidate_commit": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "repository": "fixture-extra",
            "repository_commit": "2222222222222222222222222222222222222222",
            "metric": "typo",
            "task": format!("extra-{task}"),
            "arm": "default",
            "results": results,
        }));
    }
    entries
}

#[given("a fixture with a second repository and its own typo gold set")]
fn second_repository_fixture(world: &mut BenchmarkWorld) -> Result<(), String> {
    let root = variant_fixture_root("benchmark-second-repository")?;
    let source = root.join("repo-extra/src/extra.py");
    fs::create_dir_all(
        source
            .parent()
            .ok_or("extra repository source has no parent")?,
    )
    .map_err(|error| format!("create extra repository: {error}"))?;
    fs::write(&source, extra_repository_source())
        .map_err(|error| format!("write extra repository source: {error}"))?;

    let gold_path = root.join("gold/extra/typo.jsonl");
    fs::create_dir_all(
        gold_path
            .parent()
            .ok_or("extra gold has no parent")?,
    )
    .map_err(|error| format!("create extra gold directory: {error}"))?;
    fs::write(&gold_path, extra_typo_gold())
        .map_err(|error| format!("write extra gold: {error}"))?;
    let fingerprint = benchmark::manifest::file_fingerprint(&gold_path)
        .map_err(|error| format!("fingerprint extra gold: {error:#}"))?;

    let mut manifest = read_fixture_manifest()?;
    let repository: serde_yaml_ng::Value = serde_yaml_ng::from_str(EXTRA_REPOSITORY)
        .map_err(|error| format!("parse extra repository: {error}"))?;
    manifest
        .get_mut("repositories")
        .and_then(serde_yaml_ng::Value::as_sequence_mut)
        .ok_or("fixture manifest has no repositories sequence")?
        .push(repository);
    let gold: serde_yaml_ng::Value = serde_yaml_ng::from_str(&format!(
        "{{repository: fixture-extra, metric: typo, version: 1, tasks: 5, fingerprint: \"{fingerprint}\", path: gold/extra/typo.jsonl}}"
    ))
    .map_err(|error| format!("parse extra gold entry: {error}"))?;
    manifest
        .get_mut("gold")
        .and_then(serde_yaml_ng::Value::as_sequence_mut)
        .ok_or("fixture manifest has no gold sequence")?
        .push(gold);

    let cassette_path = root.join("cassettes/results/results.jsonl");
    let mut cassette = fs::read_to_string(&cassette_path)
        .map_err(|error| format!("read result cassette: {error}"))?;
    for entry in extra_results() {
        cassette.push_str(
            &serde_json::to_string(&entry)
                .map_err(|error| format!("serialize extra result: {error}"))?,
        );
        cassette.push('\n');
    }
    fs::write(&cassette_path, cassette)
        .map_err(|error| format!("write extra results: {error}"))?;
    let cassette_fingerprint = benchmark::record::cassette::fingerprint(&cassette_path)
        .map_err(|error| format!("fingerprint variant result cassette: {error:#}"))?;
    manifest["cassettes"]["results"]["fingerprint"] =
        serde_yaml_ng::Value::String(cassette_fingerprint);

    world.manifest_path = write_variant_manifest(&root, &manifest)?;
    world.out_dir = root.join("out");
    load_fixture(world)
}

#[then(
    regex = r"^the typo recall at rank one for (\S+) is ([\d.]+) with (\d+) of (\d+) tasks$"
)]
fn typo_recall_at_one_per_repository(
    world: &mut BenchmarkWorld,
    captures: &[String],
) -> Result<(), String> {
    let [repository, value, numerator, denominator] = captures else {
        return Err("per-repository typo step needs four captures".into());
    };
    let json = report_json(world)?;
    let expected_value: f64 = value
        .parse()
        .map_err(|error| format!("parse per-repository value: {error}"))?;
    let pointer = format!("/metrics/typo/per_repository/{repository}");
    let cell = json
        .pointer(&pointer)
        .ok_or_else(|| format!("report has no per-repository typo value for {repository}"))?;
    let actual_value = cell
        .get("value")
        .and_then(serde_json::Value::as_f64)
        .ok_or_else(|| format!("{repository} per-repository typo value is not measured"))?;
    check_value(actual_value, value, "per-repository typo recall @1")?;
    let expected_value_json = cell
        .get("value")
        .and_then(serde_json::Value::as_f64)
        .ok_or("missing measured value")?;
    if (expected_value_json - expected_value).abs() > 1e-9 {
        return Err(format!(
            "{repository} per-repository typo value is {expected_value_json}, expected {expected_value}"
        ));
    }
    let actual_numerator = cell
        .get("numerator")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| format!("{repository} per-repository cell has no numerator"))?;
    let actual_denominator = cell
        .get("denominator")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| format!("{repository} per-repository cell has no denominator"))?;
    check_counts(
        actual_numerator,
        actual_denominator,
        numerator,
        denominator,
        "per-repository typo recall @1",
    )?;
    let markdown = fs::read_to_string(world.out_dir.join("report.md"))
        .map_err(|error| format!("read report.md: {error}"))?;
    let expected_line = format!(
        "Typo recall @1 for `{repository}`: {value} ({numerator} of {denominator})"
    );
    if !markdown.contains(&expected_line) {
        return Err(format!(
            "report.md does not state the per-repository value `{expected_line}`"
        ));
    }
    Ok(())
}

#[given("a fixture gold file changed without updating its declared fingerprint")]
fn gold_fingerprint_mismatch(world: &mut BenchmarkWorld) -> Result<(), String> {
    let root = variant_fixture_root("benchmark-gold-fingerprint-mismatch")?;
    rewrite_first_typo_task(&root, |task| {
        task["query"] = serde_json::Value::String("mutated after the fingerprint".to_string());
    })?;
    let manifest = read_fixture_manifest()?;
    world.manifest_path = write_variant_manifest(&root, &manifest)?;
    world.out_dir = root.join("out");
    load_fixture(world)
}

#[then(
    regex = r"^the error names the (\S+) gold set and the mismatched expected and actual fingerprints$"
)]
fn error_names_gold_fingerprints(
    world: &mut BenchmarkWorld,
    captures: &[String],
) -> Result<(), String> {
    let [metric] = captures else {
        return Err("gold-fingerprint error step needs one capture".into());
    };
    let message = &world
        .run_failure
        .as_ref()
        .ok_or("the run did not fail; a report was produced")?
        .message;
    let manifest = world
        .manifest
        .as_ref()
        .ok_or("variant manifest was not loaded")?;
    let set = manifest
        .gold
        .iter()
        .find(|set| set.metric == metric.as_str())
        .ok_or_else(|| format!("manifest has no {metric} gold set"))?;
    let base = world
        .manifest_path
        .parent()
        .ok_or("variant manifest has no parent")?;
    let actual = benchmark::manifest::file_fingerprint(&base.join(&set.path))
        .map_err(|error| format!("fingerprint gold file: {error:#}"))?;
    if set.fingerprint == actual {
        return Err("the variant gold file was not mutated".into());
    }
    for expected in [metric.as_str(), set.fingerprint.as_str(), actual.as_str(), "gold"] {
        if !message.contains(expected) {
            return Err(format!("error does not contain `{expected}`: {message}"));
        }
    }
    Ok(())
}

#[given("a fixture result cassette changed without updating its declared fingerprint")]
fn cassette_fingerprint_mismatch(world: &mut BenchmarkWorld) -> Result<(), String> {
    let root = variant_fixture_root("benchmark-cassette-fingerprint-mismatch")?;
    let path = root.join("cassettes/results/results.jsonl");
    let text = fs::read_to_string(&path)
        .map_err(|error| format!("read result cassette: {error}"))?;
    let mut lines = text.lines();
    let first = lines.next().ok_or("fixture result cassette is empty")?;
    let mut entry: serde_json::Value =
        serde_json::from_str(first).map_err(|error| format!("parse result cassette: {error}"))?;
    entry["results"][0]["end_line"] = serde_json::Value::from(9999);
    let mut rewritten = serde_json::to_string(&entry)
        .map_err(|error| format!("serialize mutated result: {error}"))?;
    rewritten.push('\n');
    for line in lines {
        rewritten.push_str(line);
        rewritten.push('\n');
    }
    fs::write(&path, rewritten).map_err(|error| format!("write mutated cassette: {error}"))?;
    let manifest = read_fixture_manifest()?;
    world.manifest_path = write_variant_manifest(&root, &manifest)?;
    world.out_dir = root.join("out");
    load_fixture(world)
}

#[then(
    regex = r"^the error names the (result|model) cassette and the mismatched expected and actual fingerprints$"
)]
fn error_names_cassette_fingerprints(
    world: &mut BenchmarkWorld,
    captures: &[String],
) -> Result<(), String> {
    let [layer] = captures else {
        return Err("cassette-fingerprint error step needs one capture".into());
    };
    let message = &world
        .run_failure
        .as_ref()
        .ok_or("the run did not fail; a report was produced")?
        .message;
    let manifest = world
        .manifest
        .as_ref()
        .ok_or("variant manifest was not loaded")?;
    let cassette = match layer.as_str() {
        "result" => &manifest.cassettes.results,
        "model" => &manifest.cassettes.model,
        other => return Err(format!("unknown cassette layer {other}")),
    };
    let base = world
        .manifest_path
        .parent()
        .ok_or("variant manifest has no parent")?;
    let actual = benchmark::record::cassette::fingerprint(&base.join(&cassette.path))
        .map_err(|error| format!("fingerprint cassette: {error:#}"))?;
    if cassette.fingerprint == actual {
        return Err("the variant cassette was not mutated".into());
    }
    for expected in [
        layer.as_str(),
        "cassette",
        cassette.fingerprint.as_str(),
        actual.as_str(),
    ] {
        if !message.contains(expected) {
            return Err(format!("error does not contain `{expected}`: {message}"));
        }
    }
    Ok(())
}

#[given("a fixture result cassette recorded for a different candidate version")]
fn candidate_identity_mismatch(world: &mut BenchmarkWorld) -> Result<(), String> {
    let root = variant_fixture_root("benchmark-candidate-identity-mismatch")?;
    let path = root.join("cassettes/results/results.jsonl");
    let text = fs::read_to_string(&path)
        .map_err(|error| format!("read result cassette: {error}"))?;
    let mut lines = text.lines();
    let first = lines.next().ok_or("fixture result cassette is empty")?;
    let mut entry: serde_json::Value =
        serde_json::from_str(first).map_err(|error| format!("parse result cassette: {error}"))?;
    if entry["candidate"] != "fixture-ocbi" {
        return Err("first result cassette entry is not fixture-ocbi".into());
    }
    entry["candidate_version"] = serde_json::Value::String("fixture-0".to_string());
    let mut rewritten = serde_json::to_string(&entry)
        .map_err(|error| format!("serialize mutated result: {error}"))?;
    rewritten.push('\n');
    for line in lines {
        rewritten.push_str(line);
        rewritten.push('\n');
    }
    fs::write(&path, rewritten).map_err(|error| format!("write mutated cassette: {error}"))?;
    let mut manifest = read_fixture_manifest()?;
    refresh_cassette_fingerprint(&mut manifest, &root, "results")?;
    world.manifest_path = write_variant_manifest(&root, &manifest)?;
    world.out_dir = root.join("out");
    load_fixture(world)
}

#[then("the error names fixture-ocbi and the mismatched candidate versions")]
fn error_names_candidate_versions(world: &mut BenchmarkWorld) -> Result<(), String> {
    let message = &world
        .run_failure
        .as_ref()
        .ok_or("the run did not fail; a report was produced")?
        .message;
    for expected in ["fixture-ocbi", "fixture-1", "fixture-0", "candidate identity mismatch"] {
        if !message.contains(expected) {
            return Err(format!("error does not contain `{expected}`: {message}"));
        }
    }
    Ok(())
}

fn limitations(world: &BenchmarkWorld) -> Result<Vec<String>, String> {
    let json = report_json(world)?;
    json.get("limitations")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| "report.json has no limitations list".to_string())?
        .iter()
        .map(|entry| {
            entry
                .as_str()
                .map(str::to_string)
                .ok_or_else(|| "limitation is not a string".to_string())
        })
        .collect()
}

#[then("the report states the explanation rubric's surface-form limitation")]
fn explanation_surface_form_limitation(world: &mut BenchmarkWorld) -> Result<(), String> {
    let limitations = limitations(world)?;
    if !limitations
        .iter()
        .any(|limitation| limitation.contains("surface form"))
    {
        return Err(format!(
            "report limitations do not state the surface-form limitation: {limitations:?}"
        ));
    }
    let markdown = fs::read_to_string(world.out_dir.join("report.md"))
        .map_err(|error| format!("read report.md: {error}"))?;
    if !markdown.contains("## Limitations") {
        return Err("report.md has no Limitations section".into());
    }
    Ok(())
}

#[then(
    "the report lists the proposed target, the unavailable capability reasons, and the regex proxy arm as limitations"
)]
fn report_limitations_list(world: &mut BenchmarkWorld) -> Result<(), String> {
    let limitations = limitations(world)?;
    for expected in [
        "proposed",
        "no_explanation_operation",
        "no_natural_language_search",
        "regex_pattern_search_proxy",
    ] {
        if !limitations
            .iter()
            .any(|limitation| limitation.contains(expected))
        {
            return Err(format!(
                "report limitations do not name `{expected}`: {limitations:?}"
            ));
        }
    }
    let markdown = fs::read_to_string(world.out_dir.join("report.md"))
        .map_err(|error| format!("read report.md: {error}"))?;
    if !markdown.contains("regex_pattern_search_proxy") {
        return Err("report.md limitations do not name the regex proxy arm".into());
    }
    Ok(())
}

#[then("an invalid report is rejected before it is written")]
fn invalid_report_rejected(world: &mut BenchmarkWorld) -> Result<(), String> {
    let report = world
        .report
        .as_ref()
        .ok_or("the run did not produce a report")?;
    let mut invalid = report.clone();
    invalid.metrics.typo.aggregate.coverage.numerator += 1;
    let out = world.out_dir.join("invalid-report");
    if let Err(error) = benchmark::report::render::write_reports(&invalid, &out) {
        if !format!("{error:#}").contains("coverage") {
            return Err(format!(
                "invalid report was rejected for the wrong reason: {error:#}"
            ));
        }
    } else {
        return Err("an inconsistent report was written instead of rejected".into());
    }
    for name in ["report.json", "report.md"] {
        if out.join(name).exists() {
            return Err(format!("rejected report wrote {name}"));
        }
    }
    Ok(())
}

fn real_manifest_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("manifest.yaml")
}

fn real_setup_lock_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("setup.lock.json")
}

fn load_real_manifest(world: &mut BenchmarkWorld) -> Result<Manifest, String> {
    let path = real_manifest_path();
    let manifest = Manifest::load(&path)
        .map_err(|error| format!("load real manifest {}: {error:#}", path.display()))?;
    world.manifest_path = path;
    world.manifest = Some(manifest.clone());
    Ok(manifest)
}

#[given("the real benchmark manifest")]
fn the_real_benchmark_manifest(world: &mut BenchmarkWorld) -> Result<(), String> {
    load_real_manifest(world).map(|_| ())
}

#[then("it pins typescript-reference, python-reference, and rust-reference at a URL, revision, commit, language, measured size, and power")]
fn repositories_are_pinned(world: &mut BenchmarkWorld) -> Result<(), String> {
    let manifest = load_real_manifest(world)?;
    for id in ["typescript-reference", "python-reference", "rust-reference"] {
        let repository = manifest
            .repository(id)
            .ok_or_else(|| format!("real manifest does not pin {id}"))?;
        if repository.url.is_empty() || !repository.url.starts_with("https://") {
            return Err(format!("{id} url is not an https URL: {}", repository.url));
        }
        if repository.revision.is_empty() {
            return Err(format!("{id} has no revision"));
        }
        if repository.commit.len() != 40
            || !repository.commit.chars().all(|c| c.is_ascii_hexdigit())
        {
            return Err(format!("{id} commit is not 40 hex characters: {}", repository.commit));
        }
        if repository.language.is_empty() {
            return Err(format!("{id} has no language"));
        }
        let measured = repository.loc.language_code_lines;
        if measured == 0 {
            return Err(format!("{id} has no measured size"));
        }
        let minimum = repository.minimum_code_lines.unwrap_or(200_000);
        if measured < minimum {
            return Err(format!(
                "{id} measured {measured} code lines, below the {minimum}-line minimum"
            ));
        }
        if !matches!(repository.power, Some(benchmark::manifest::Power::Primary))
            && !matches!(repository.power, Some(benchmark::manifest::Power::Secondary))
        {
            return Err(format!("{id} has no power label"));
        }
    }
    let primary = manifest
        .repositories
        .iter()
        .filter(|repository| matches!(repository.power, Some(benchmark::manifest::Power::Primary)))
        .count();
    if primary != 1 {
        return Err(format!("real manifest has {primary} primary repositories, expected 1"));
    }
    Ok(())
}

#[then("it pins the synthetic monorepo with its fixed seed, generator version, and content fingerprint")]
fn synthetic_monorepo_is_pinned(world: &mut BenchmarkWorld) -> Result<(), String> {
    let manifest = load_real_manifest(world)?;
    let repository = manifest
        .repository("synthetic-monorepo")
        .ok_or("real manifest does not pin synthetic-monorepo")?;
    let synthetic = repository
        .synthetic
        .as_ref()
        .ok_or("synthetic-monorepo has no synthetic pin record")?;
    if synthetic.seed != 20_261_020 {
        return Err(format!("synthetic seed is {}, expected 20261020", synthetic.seed));
    }
    if synthetic.generator_version != 1 {
        return Err(format!(
            "synthetic generator version is {}, expected 1",
            synthetic.generator_version
        ));
    }
    if !synthetic.content_fingerprint.starts_with("sha256:")
        || synthetic.content_fingerprint.len() != "sha256:".len() + 64
    {
        return Err(format!(
            "synthetic content fingerprint is not a sha256 digest: {}",
            synthetic.content_fingerprint
        ));
    }
    if synthetic.code_lines != 10_000_000 {
        return Err(format!(
            "synthetic code lines are {}, expected 10000000",
            synthetic.code_lines
        ));
    }
    if repository.loc.language_code_lines != synthetic.code_lines {
        return Err("synthetic measured size differs from its generated code lines".into());
    }
    Ok(())
}

#[then("it sets the public repository index deadline to 14400 seconds")]
fn public_deadline(world: &mut BenchmarkWorld) -> Result<(), String> {
    let manifest = load_real_manifest(world)?;
    if manifest.index_deadlines.get("public_repository_seconds").copied() != Some(14_400) {
        return Err(format!(
            "public repository deadline is {:?}, expected 14400",
            manifest.index_deadlines.get("public_repository_seconds")
        ));
    }
    Ok(())
}

#[then("it sets the synthetic monorepo index deadline to 1800 seconds")]
fn synthetic_deadline(world: &mut BenchmarkWorld) -> Result<(), String> {
    let manifest = load_real_manifest(world)?;
    if manifest.index_deadlines.get("synthetic_seconds").copied() != Some(1_800) {
        return Err(format!(
            "synthetic deadline is {:?}, expected 1800",
            manifest.index_deadlines.get("synthetic_seconds")
        ));
    }
    Ok(())
}

#[then("it pins open-codebase-index, CKB, and Serena at a candidate version and source commit")]
fn candidates_are_pinned(world: &mut BenchmarkWorld) -> Result<(), String> {
    let manifest = load_real_manifest(world)?;
    for (id, name) in [("ocbi", "open-codebase-index"), ("ckb", "CKB"), ("serena", "Serena")] {
        if manifest.candidate(id).is_none() {
            return Err(format!("real manifest does not pin {name} ({id})"));
        }
    }
    manifest
        .validate_candidate_pins()
        .map_err(|error| format!("{error:#}"))
}

#[then("each candidate pin records the evidence for its version and commit")]
fn candidate_evidence_is_recorded(world: &mut BenchmarkWorld) -> Result<(), String> {
    let manifest = load_real_manifest(world)?;
    manifest
        .validate_candidate_pins()
        .map_err(|error| format!("{error:#}"))
}

#[given("the real benchmark manifest and the committed setup lock")]
fn the_real_manifest_and_setup_lock(world: &mut BenchmarkWorld) -> Result<(), String> {
    load_real_manifest(world)?;
    let lock_path = real_setup_lock_path();
    benchmark::setup::SetupLock::load(&lock_path)
        .map_err(|error| format!("load committed setup lock {}: {error:#}", lock_path.display()))?;
    Ok(())
}

fn real_lock() -> Result<benchmark::setup::SetupLock, String> {
    benchmark::setup::SetupLock::load(&real_setup_lock_path())
        .map_err(|error| format!("load committed setup lock: {error:#}"))
}

#[then("every candidate and repository pin is identical in both files")]
fn setup_lock_pins_agree(world: &mut BenchmarkWorld) -> Result<(), String> {
    let manifest = load_real_manifest(world)?;
    benchmark::setup::validate_pins(&real_manifest_path(), &real_setup_lock_path())
        .map_err(|error| format!("setup validation failed: {error:#}"))?;
    let lock = real_lock()?;
    for candidate in &manifest.candidates {
        let entry = lock
            .candidates
            .get(&candidate.id)
            .ok_or_else(|| format!("setup lock has no candidate {}", candidate.id))?;
        if entry.version != candidate.version || entry.commit != candidate.commit {
            return Err(format!("candidate {} pins differ between manifest and lock", candidate.id));
        }
    }
    for repository in &manifest.repositories {
        let entry = lock
            .repositories
            .get(&repository.id)
            .ok_or_else(|| format!("setup lock has no repository {}", repository.id))?;
        if repository.synthetic.is_none() && entry.commit.as_deref() != Some(repository.commit.as_str()) {
            return Err(format!("repository {} pins differ between manifest and lock", repository.id));
        }
    }
    Ok(())
}

#[then("every pinned commit is a 40-character commit")]
fn pinned_commits_are_40_hex(world: &mut BenchmarkWorld) -> Result<(), String> {
    let manifest = load_real_manifest(world)?;
    let lock = real_lock()?;
    let mut commits = Vec::new();
    for candidate in &manifest.candidates {
        commits.push((candidate.id.clone(), candidate.commit.clone()));
    }
    for repository in &manifest.repositories {
        if repository.synthetic.is_none() {
            commits.push((repository.id.clone(), repository.commit.clone()));
        }
    }
    for (label, commit) in commits {
        if commit.len() != 40 || !commit.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(format!("{label} pin {commit} is not a 40-character commit"));
        }
    }
    for (id, entry) in &lock.candidates {
        if entry.commit.len() != 40 || !entry.commit.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(format!("setup lock candidate {id} commit is not 40 hex characters"));
        }
    }
    Ok(())
}

#[then("the setup lock records a probe result and the toolchain versions for every candidate")]
fn setup_lock_probes_and_toolchains(world: &mut BenchmarkWorld) -> Result<(), String> {
    let manifest = load_real_manifest(world)?;
    let lock = real_lock()?;
    for candidate in &manifest.candidates {
        let entry = lock
            .candidates
            .get(&candidate.id)
            .ok_or_else(|| format!("setup lock has no probe record for {}", candidate.id))?;
        if entry.probe.status != "ok" || entry.probe.command.is_empty() {
            return Err(format!(
                "setup lock probe for {} is not a successful probe result",
                candidate.id
            ));
        }
        if entry.toolchain.is_empty() {
            return Err(format!("setup lock has no toolchain versions for {}", candidate.id));
        }
    }
    Ok(())
}

#[then("the setup lock records no candidate pin as its origin")]
fn setup_lock_has_no_self_pin_origin(world: &mut BenchmarkWorld) -> Result<(), String> {
    load_real_manifest(world)?;
    let lock = real_lock()?;
    for (id, entry) in &lock.candidates {
        let origin = &entry.pinned_by;
        if origin.is_empty()
            || origin == id
            || origin == &entry.version
            || origin == &entry.commit
            || !origin.contains("manifest")
        {
            return Err(format!(
                "setup lock candidate {id} records pin origin {origin}, which is not the manifest"
            ));
        }
    }
    Ok(())
}

#[then("the setup lock records both endpoint localities, both model names, and the explanation-query budget")]
fn setup_lock_endpoints(world: &mut BenchmarkWorld) -> Result<(), String> {
    load_real_manifest(world)?;
    let lock = real_lock()?;
    let endpoints = &lock.endpoints;
    for (label, endpoint) in [
        ("embedding", &endpoints.embedding),
        ("explanation", &endpoints.explanation),
    ] {
        if endpoint.locality.is_empty() {
            return Err(format!("setup lock does not record the {label} locality"));
        }
        if endpoint.model.as_deref().is_none_or(str::is_empty) {
            return Err(format!("setup lock does not record the {label} model name"));
        }
    }
    if endpoints.embedding.locality != "loopback" {
        return Err("embedding endpoint locality is not loopback".into());
    }
    let budget = endpoints
        .explanation
        .query_budget
        .ok_or("setup lock does not record the explanation-query budget")?;
    if budget > 80 {
        return Err(format!("explanation-query budget {budget} exceeds 80"));
    }
    Ok(())
}

#[given("a real fixed input whose checkout or candidate installation does not match the manifest")]
fn a_fixed_input_mismatch(world: &mut BenchmarkWorld) -> Result<(), String> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("benchmark crate has no workspace root")?
        .join("target")
        .join("benchmark-setup-mismatch");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).map_err(|error| format!("create {}: {error}", root.display()))?;

    let checkout = root.join("checkout");
    fs::create_dir_all(&checkout).map_err(|error| format!("create {}: {error}", checkout.display()))?;
    fs::write(checkout.join("file.ts"), "export const value = 1;\n")
        .map_err(|error| format!("write mismatch checkout: {error}"))?;
    let git = |args: &[&str]| -> Result<(), String> {
        let output = std::process::Command::new("git")
            .arg("-C")
            .arg(&checkout)
            .args(["-c", "user.name=benchmark", "-c", "user.email=benchmark@example.invalid"])
            .args(args)
            .output()
            .map_err(|error| format!("run git: {error}"))?;
        if !output.status.success() {
            return Err(format!(
                "git {:?} failed: {}",
                args,
                String::from_utf8_lossy(&output.stderr).trim()
            ));
        }
        Ok(())
    };
    git(&["init", "-q"])?;
    git(&["add", "file.ts"])?;
    git(&["commit", "-q", "-m", "wrong pin"])?;
    let actual = std::process::Command::new("git")
        .arg("-C")
        .arg(&checkout)
        .args(["rev-parse", "HEAD"])
        .output()
        .map_err(|error| format!("run git rev-parse: {error}"))?;
    let actual = String::from_utf8_lossy(&actual.stdout).trim().to_string();

    let synthetic = root.join("synthetic");
    fs::create_dir_all(&synthetic).map_err(|error| format!("create {}: {error}", synthetic.display()))?;
    fs::write(synthetic.join("module_0000.ts"), "export const value = 1;\n")
        .map_err(|error| format!("write synthetic checkout: {error}"))?;

    let mut manifest = Manifest::load(&real_manifest_path())
        .map_err(|error| format!("load real manifest: {error:#}"))?;
    let expected = manifest
        .repository("typescript-reference")
        .ok_or("real manifest has no typescript-reference")?
        .commit
        .clone();
    for repository in &mut manifest.repositories {
        repository.checkout = if repository.synthetic.is_some() {
            synthetic.to_string_lossy().to_string()
        } else {
            checkout.to_string_lossy().to_string()
        };
    }
    for candidate in &mut manifest.candidates {
        candidate.install = "/bin/sh".to_string();
    }
    let manifest_path = root.join("manifest.yaml");
    let text = serde_yaml_ng::to_string(&manifest)
        .map_err(|error| format!("serialize mismatch manifest: {error}"))?;
    fs::write(&manifest_path, text)
        .map_err(|error| format!("write {}: {error}", manifest_path.display()))?;

    world.manifest_path = manifest_path;
    world.manifest = Some(manifest);
    world.out_dir = root.join("out");
    world.report = None;
    world.run_failure = None;
    world.mismatch = Some((expected, actual));
    Ok(())
}

#[when("the benchmark validates the fixed inputs")]
fn validate_fixed_inputs(world: &mut BenchmarkWorld) -> Result<(), String> {
    match benchmark::setup::validate_fixed_inputs(&world.manifest_path) {
        Ok(()) => Err("a mismatched fixed input passed validation".into()),
        Err(error) => {
            world.run_failure = Some(benchmark::runner::RunFailure::from_error(error));
            Ok(())
        }
    }
}

#[then("the run exits with code 2 and names the mismatched fixed input")]
fn mismatch_exits_two_and_names_input(world: &mut BenchmarkWorld) -> Result<(), String> {
    let failure = world
        .run_failure
        .as_ref()
        .ok_or("the mismatched input did not stop validation")?;
    if failure.exit_code != benchmark::setup::EXIT_INPUT_ERROR {
        return Err(format!(
            "validation exit code was {}, expected {}",
            failure.exit_code,
            benchmark::setup::EXIT_INPUT_ERROR
        ));
    }
    if !failure.message.contains("typescript-reference") {
        return Err(format!(
            "error does not name the mismatched fixed input: {}",
            failure.message
        ));
    }
    if !failure.message.contains("expected commit") || !failure.message.contains("actual commit") {
        return Err(format!(
            "error does not report the expected and actual commits: {}",
            failure.message
        ));
    }
    Ok(())
}

#[then("no benchmark run starts")]
fn no_benchmark_run_starts(world: &mut BenchmarkWorld) -> Result<(), String> {
    if world.report.is_some() {
        return Err("a benchmark report was produced after invalid preparation".into());
    }
    for name in ["report.json", "report.md"] {
        let path = world.out_dir.join(name);
        if path.exists() {
            return Err(format!("invalid preparation wrote {}", path.display()));
        }
    }
    Ok(())
}

// --- S13: normalized evidence keeps the candidate's order and resolution ---

fn adapter_answers(world: &BenchmarkWorld) -> Result<&Vec<CandidateAnswer>, String> {
    world
        .adapter_answers
        .as_ref()
        .ok_or_else(|| "the candidate session returned no answers".to_string())
}

fn adapter_results(world: &BenchmarkWorld) -> Result<&Vec<NormalizedLocation>, String> {
    world
        .adapter_results
        .as_ref()
        .ok_or_else(|| "the candidate answers were not normalized".to_string())
}

#[given("a real candidate that returns duplicate answers and a file-only answer")]
fn a_candidate_returns_duplicate_and_file_answers(world: &mut BenchmarkWorld) -> Result<(), String> {
    world.adapter_answers = Some(vec![
        CandidateAnswer::symbol("src/math_utils.py", 10, 10, "helper_10"),
        CandidateAnswer::symbol("src/math_utils.py", 10, 10, "helper_10"),
        CandidateAnswer::file("src/text_utils.py"),
        CandidateAnswer::symbol("src/math_utils.py", 30, 30, "helper_30"),
    ]);
    world.adapter_results = None;
    Ok(())
}

#[when("the maintainer runs the real benchmark")]
fn the_maintainer_runs_the_real_benchmark(world: &mut BenchmarkWorld) -> Result<(), String> {
    let Some(answers) = world.adapter_answers.clone() else {
        return run_replay(world);
    };
    let mut session = FakeSession::exposing("fixture-ocbi", vec![Operation::Typo])
        .answering("typo-1", answers);
    let returned = session
        .query(&QueryRequest {
            repository: "fixture-repo".to_string(),
            metric: "typo".to_string(),
            task: "typo-1".to_string(),
            query: "helper_10".to_string(),
        })
        .map_err(|error| format!("query candidate session: {error}"))?;
    world.adapter_answers = Some(returned.clone());
    world.adapter_results = Some(normalize_answers(&returned));
    Ok(())
}

#[then("the normalized results keep the candidate's order")]
fn normalized_results_keep_candidate_order(world: &mut BenchmarkWorld) -> Result<(), String> {
    let raw = adapter_answers(world)?;
    let normalized = adapter_results(world)?;
    if normalized.len() != 3 {
        return Err(format!(
            "expected 3 normalized results after deduplication, got {}",
            normalized.len()
        ));
    }
    // The first, third, and fourth candidate answers survive in that order.
    let expected_positions = [0usize, 2, 3];
    for (rank, raw_index) in expected_positions.iter().enumerate() {
        if normalized[rank] != raw[*raw_index].normalize() {
            return Err(format!(
                "normalized result {rank} does not match candidate answer {raw_index}"
            ));
        }
    }
    Ok(())
}

#[then("duplicate locations are removed without reordering")]
fn duplicates_are_removed_without_reordering(world: &mut BenchmarkWorld) -> Result<(), String> {
    let raw = adapter_answers(world)?;
    let normalized = adapter_results(world)?;
    if raw.len() != 4 {
        return Err(format!("expected 4 candidate answers, got {}", raw.len()));
    }
    if normalized.len() != 3 {
        return Err(format!(
            "the duplicate was not removed: {} normalized results",
            normalized.len()
        ));
    }
    if normalized
        .windows(2)
        .any(|pair| pair[0] == pair[1])
    {
        return Err("a duplicate location survived normalization".into());
    }
    let expected = [
        ("src/math_utils.py", 10, "helper_10"),
        ("src/text_utils.py", 0, ""),
        ("src/math_utils.py", 30, "helper_30"),
    ];
    for (location, (path, line, symbol)) in normalized.iter().zip(expected) {
        if location.path != path || location.start_line != line || location.symbol != symbol {
            return Err(format!(
                "normalized order changed: got {} at {} symbol {:?}, expected {} at {} symbol {:?}",
                location.path, location.start_line, location.symbol, path, line, symbol
            ));
        }
    }
    Ok(())
}

#[then("the file-only answer remains a file resolution")]
fn file_only_answer_remains_a_file_resolution(world: &mut BenchmarkWorld) -> Result<(), String> {
    let normalized = adapter_results(world)?;
    let file = normalized
        .iter()
        .find(|location| location.path == "src/text_utils.py")
        .ok_or("the file-only answer is missing from the normalized results")?;
    if file.resolution.as_deref() != Some("file") {
        return Err(format!(
            "file-only answer resolution is {:?}, expected file",
            file.resolution
        ));
    }
    if file.start_line != 0 || file.end_line != 0 || !file.symbol.is_empty() {
        return Err(
            "the file-only answer was promoted to a line/symbol resolution".to_string(),
        );
    }
    Ok(())
}

// --- Shared adapter scenario fixtures ---------------------------------------

fn fixture_manifest_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/e2e/manifest.yaml")
}

fn fixture_candidate(id: &str, version: &str, commit: &str) -> Candidate {
    Candidate {
        id: id.to_string(),
        version: version.to_string(),
        commit: commit.to_string(),
        install: "fixture adapter".to_string(),
        adapter: "fixture".to_string(),
        adapter_version: 1,
        evidence: None,
        capabilities: std::collections::BTreeMap::new(),
        setup: benchmark::manifest::CandidateSetup::default(),
        index_deadline_seconds: None,
        unavailable: None,
    }
}

fn write_typed_manifest(root: &Path, manifest: &Manifest) -> Result<PathBuf, String> {
    let path = root.join("manifest.yaml");
    let text = serde_yaml_ng::to_string(manifest)
        .map_err(|error| format!("serialize variant manifest: {error}"))?;
    fs::write(&path, text).map_err(|error| format!("write {}: {error}", path.display()))?;
    Ok(path)
}

fn refresh_results_cassette(manifest: &mut Manifest, root: &Path) -> Result<(), String> {
    let relative = manifest.cassettes.results.path.clone();
    let fingerprint = benchmark::record::cassette::fingerprint(&root.join(relative))
        .map_err(|error| format!("fingerprint variant results cassette: {error:#}"))?;
    manifest.cassettes.results.fingerprint = fingerprint;
    Ok(())
}

/// Duplicate one candidate's recorded results under another candidate id so a
/// scenario can drive a differently-probed candidate without a candidate
/// process. The duplicated entries keep their recorded order.
fn duplicate_results(root: &Path, from: &str, to: &str) -> Result<(), String> {
    let path = root.join("cassettes/results/results.jsonl");
    let text = fs::read_to_string(&path)
        .map_err(|error| format!("read variant results {}: {error}", path.display()))?;
    let mut out = String::new();
    for line in text.lines() {
        out.push_str(line);
        out.push('\n');
        let mut value: serde_json::Value =
            serde_json::from_str(line).map_err(|error| format!("parse variant result: {error}"))?;
        if value["candidate"] == from {
            value["candidate"] = serde_json::Value::String(to.to_string());
            out.push_str(
                &serde_json::to_string(&value)
                    .map_err(|error| format!("serialize duplicated result: {error}"))?,
            );
            out.push('\n');
        }
    }
    fs::write(&path, out).map_err(|error| format!("write variant results {}: {error}", path.display()))
}

fn focus_candidate(world: &BenchmarkWorld) -> Result<String, String> {
    world
        .focus_candidate
        .clone()
        .ok_or_else(|| "no candidate is under test".to_string())
}

// --- S14: a missing operation is unavailable with its capability reason ---

#[given("a real candidate whose documented interface has no explanation operation")]
fn a_candidate_without_an_explanation_operation(world: &mut BenchmarkWorld) -> Result<(), String> {
    let root = variant_fixture_root("benchmark-capability-gap")?;
    let mut manifest = Manifest::load(&fixture_manifest_path())
        .map_err(|error| format!("load fixture manifest: {error:#}"))?;
    let id = "fixture-partial";
    manifest.candidates.push(fixture_candidate(
        id,
        "fixture-1",
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    ));
    duplicate_results(&root, "fixture-ocbi", id)?;
    refresh_results_cassette(&mut manifest, &root)?;
    let mut session = FakeSession::exposing(
        id,
        vec![
            Operation::Typo,
            Operation::NaturalLanguage,
            Operation::Concept,
            Operation::ConceptEquivalence,
            Operation::Expansion,
        ],
    );
    apply_session(&mut manifest, id, &mut session)
        .map_err(|error| format!("apply candidate session: {error:#}"))?;
    let path = write_typed_manifest(&root, &manifest)?;

    world.manifest_path = path;
    world.out_dir = root.join("out");
    world.report = None;
    world.run_failure = None;
    world.adapter_answers = None;
    world.adapter_results = None;
    world.focus_candidate = Some(id.to_string());
    Ok(())
}

#[then("its explanation cell is unavailable with the capability reason")]
fn explanation_cell_is_unavailable(world: &mut BenchmarkWorld) -> Result<(), String> {
    let id = focus_candidate(world)?;
    let cell = report(world)
        .metrics
        .explanation
        .per_candidate
        .get(&id)
        .ok_or_else(|| format!("the report has no explanation cell for {id}"))?;
    if cell.status != benchmark::report::model::CellStatus::Unavailable {
        return Err(format!(
            "explanation cell for {id} is {:?}, expected unavailable",
            cell.status
        ));
    }
    if cell.value.is_some() {
        return Err(format!("unavailable explanation cell for {id} carries a value"));
    }
    if cell.reason.as_deref() != Some("no_explanation_operation") {
        return Err(format!(
            "explanation cell reason is {:?}, expected no_explanation_operation",
            cell.reason
        ));
    }
    Ok(())
}

#[then("its measured families remain measured")]
fn measured_families_remain_measured(world: &mut BenchmarkWorld) -> Result<(), String> {
    let id = focus_candidate(world)?;
    let report = report(world);
    let cells = [
        ("typo", report.metrics.typo.per_candidate.get(&id)),
        (
            "natural-language",
            report.metrics.natural_language.per_candidate.get(&id),
        ),
        ("concept", report.metrics.concept.per_candidate.get(&id)),
    ];
    for (family, cell) in cells {
        let cell = cell.ok_or_else(|| format!("the report has no {family} cell for {id}"))?;
        if cell.status != benchmark::report::model::CellStatus::Measured {
            return Err(format!(
                "{family} cell for {id} is {:?}, expected measured",
                cell.status
            ));
        }
        if cell.value.is_none() || cell.denominator == 0 {
            return Err(format!("measured {family} cell for {id} has no value"));
        }
    }
    Ok(())
}

#[then("the report lists the missing operation as a capability gap")]
fn missing_operation_is_a_capability_gap(world: &mut BenchmarkWorld) -> Result<(), String> {
    let id = focus_candidate(world)?;
    let gap = report(world)
        .gaps
        .capability_gaps
        .iter()
        .find(|gap| gap.candidate == id && gap.capability == "explanation")
        .ok_or_else(|| format!("the report does not list the explanation gap for {id}"))?;
    if gap.reason != "no_explanation_operation" {
        return Err(format!(
            "capability gap reason is {:?}, expected no_explanation_operation",
            gap.reason
        ));
    }
    Ok(())
}

// --- S6: a failed setup probe is unavailable, never zero -------------------

#[given("a real candidate whose documented interface cannot be started")]
fn a_candidate_whose_interface_cannot_start(world: &mut BenchmarkWorld) -> Result<(), String> {
    let root = variant_fixture_root("benchmark-probe-failure")?;
    let mut manifest = Manifest::load(&fixture_manifest_path())
        .map_err(|error| format!("load fixture manifest: {error:#}"))?;
    let id = "fixture-failed-probe";
    manifest.candidates.push(fixture_candidate(
        id,
        "fixture-1",
        "dddddddddddddddddddddddddddddddddddddddd",
    ));
    let mut session = FakeSession::failing_probe(
        id,
        "candidate_probe_failed",
        "the documented interface did not complete its startup handshake",
    );
    apply_session(&mut manifest, id, &mut session)
        .map_err(|error| format!("apply candidate session: {error:#}"))?;
    let path = write_typed_manifest(&root, &manifest)?;

    world.manifest_path = path;
    world.out_dir = root.join("out");
    world.report = None;
    world.run_failure = None;
    world.adapter_answers = None;
    world.adapter_results = None;
    world.focus_candidate = Some(id.to_string());
    Ok(())
}

#[then("every metric cell for that candidate is unavailable with its setup reason")]
fn every_cell_for_the_candidate_is_unavailable(world: &mut BenchmarkWorld) -> Result<(), String> {
    let id = focus_candidate(world)?;
    let report = report(world);
    let cells = [
        ("typo", report.metrics.typo.per_candidate.get(&id)),
        (
            "typo recall @5",
            report.metrics.typo.recall_at_5_per_candidate.get(&id),
        ),
        (
            "natural-language",
            report.metrics.natural_language.per_candidate.get(&id),
        ),
        (
            "explanation",
            report.metrics.explanation.per_candidate.get(&id),
        ),
        ("concept", report.metrics.concept.per_candidate.get(&id)),
        (
            "concept-equivalence",
            report.metrics.concept_equivalence.per_candidate.get(&id),
        ),
    ];
    for (family, cell) in cells {
        let cell = cell.ok_or_else(|| format!("the report omits the {family} cell for {id}"))?;
        if cell.status != benchmark::report::model::CellStatus::Unavailable {
            return Err(format!(
                "{family} cell for {id} is {:?}, expected unavailable",
                cell.status
            ));
        }
        if cell.value.is_some() {
            return Err(format!(
                "unavailable {family} cell for {id} carries a value"
            ));
        }
        if cell.reason.as_deref() != Some("candidate_probe_failed") {
            return Err(format!(
                "{family} cell reason is {:?}, expected candidate_probe_failed",
                cell.reason
            ));
        }
    }
    Ok(())
}

#[then("the other installed candidates are measured")]
fn other_installed_candidates_are_measured(world: &mut BenchmarkWorld) -> Result<(), String> {
    let report = report(world);
    for id in ["fixture-ocbi", "fixture-ckb", "fixture-serena"] {
        let candidate = report
            .candidates
            .iter()
            .find(|candidate| candidate.id == id)
            .ok_or_else(|| format!("the report omits candidate {id}"))?;
        if candidate.status != benchmark::report::model::CandidateStatus::Available {
            return Err(format!(
                "{id} is {:?}, expected available",
                candidate.status
            ));
        }
    }
    let ocbi_typo = report
        .metrics
        .typo
        .per_candidate
        .get("fixture-ocbi")
        .ok_or("the report omits fixture-ocbi typo")?;
    let ckb_explanation = report
        .metrics
        .explanation
        .per_candidate
        .get("fixture-ckb")
        .ok_or("the report omits fixture-ckb explanation")?;
    for (label, cell) in [("fixture-ocbi typo", ocbi_typo), ("fixture-ckb explanation", ckb_explanation)] {
        if cell.status != benchmark::report::model::CellStatus::Measured {
            return Err(format!(
                "{label} is {:?}, expected measured",
                cell.status
            ));
        }
    }
    Ok(())
}

#[then("the report lists the candidate as unavailable")]
fn report_lists_the_candidate_unavailable(world: &mut BenchmarkWorld) -> Result<(), String> {
    let id = focus_candidate(world)?;
    let report = report(world);
    let candidate = report
        .candidates
        .iter()
        .find(|candidate| candidate.id == id)
        .ok_or_else(|| format!("the report omits candidate {id}"))?;
    if candidate.status != benchmark::report::model::CandidateStatus::Unavailable {
        return Err(format!(
            "{id} is {:?}, expected unavailable",
            candidate.status
        ));
    }
    if candidate.unavailable_reason.as_deref() != Some("candidate_probe_failed") {
        return Err(format!(
            "unavailable reason is {:?}, expected candidate_probe_failed",
            candidate.unavailable_reason
        ));
    }
    Ok(())
}
