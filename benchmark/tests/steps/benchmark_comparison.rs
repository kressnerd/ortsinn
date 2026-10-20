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
    let report = benchmark::runner::run_replay(&world.manifest_path, &world.out_dir)
        .map_err(|error| format!("run fixture replay: {error:#}"))?;
    world.report = Some(report);
    Ok(())
}

fn load_fixture(world: &mut BenchmarkWorld) -> Result<(), String> {
    let manifest = Manifest::load(&world.manifest_path)
        .map_err(|error| format!("load fixture manifest: {error:#}"))?;
    world.manifest = Some(manifest);
    if world.report.is_none() {
        run_replay(world)?;
    }
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
