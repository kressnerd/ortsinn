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
