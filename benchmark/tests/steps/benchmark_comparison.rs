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
    let report = benchmark::runner::run_replay(&world.manifest_path, &world.out_dir)
        .map_err(|error| format!("run fixture replay: {error:#}"))?;
    world.report = Some(report);
    Ok(())
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
