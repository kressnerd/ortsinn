// E2E CLI steps for benchmark-comparison. These steps drive the real
// `benchmark` binary; the in-process steps live in
// `benchmark/tests/steps/benchmark_comparison.rs`.

use cucumber::{given, then, when};

use crate::BenchmarkWorld;

#[given("a benchmark fixture with fixed repository revisions, pinned candidate identities, and recorded model responses")]
fn a_benchmark_fixture(_world: &mut BenchmarkWorld) -> Result<(), String> {
    unimplemented!("benchmark-comparison e2e: fixture benchmark run")
}

#[when("the maintainer runs \"benchmark run\" in the terminal")]
fn the_maintainer_runs_benchmark_run(_world: &mut BenchmarkWorld) -> Result<(), String> {
    unimplemented!("benchmark-comparison e2e: run benchmark run")
}

#[then("the command exits with code 0")]
fn the_command_exits_with_code_zero(_world: &mut BenchmarkWorld) -> Result<(), String> {
    unimplemented!("benchmark-comparison e2e: exit code")
}

#[then("the terminal shows the four measurement sections and the report path")]
fn the_terminal_shows_sections_and_report_path(_world: &mut BenchmarkWorld) -> Result<(), String> {
    unimplemented!("benchmark-comparison e2e: terminal summary")
}

#[then("the report at that path states typo recall at rank one 0.60, natural-language recall at rank five 0.50, explanation correctness 0.50, and concept recall at rank five 0.40")]
fn the_report_states_the_measured_values(_world: &mut BenchmarkWorld) -> Result<(), String> {
    unimplemented!("benchmark-comparison e2e: report values")
}

#[then("the terminal renders every metric label and value on one line at width 80")]
fn the_terminal_renders_at_width_80(_world: &mut BenchmarkWorld) -> Result<(), String> {
    unimplemented!("benchmark-comparison e2e: width 80")
}

#[then("the terminal renders every metric label and value on one line at width 160")]
fn the_terminal_renders_at_width_160(_world: &mut BenchmarkWorld) -> Result<(), String> {
    unimplemented!("benchmark-comparison e2e: width 160")
}

#[given("a local fake OpenAI-compatible endpoint and the fixture candidates")]
fn a_local_fake_endpoint(_world: &mut BenchmarkWorld) -> Result<(), String> {
    unimplemented!("benchmark-comparison e2e: local endpoint")
}

#[when("the maintainer records the benchmark run against that endpoint")]
fn the_maintainer_records_the_run(_world: &mut BenchmarkWorld) -> Result<(), String> {
    unimplemented!("benchmark-comparison e2e: record run")
}

#[then("the model requests are recorded and the model cassette fingerprint is written")]
fn the_model_requests_are_recorded(_world: &mut BenchmarkWorld) -> Result<(), String> {
    unimplemented!("benchmark-comparison e2e: model cassette")
}

#[then("the normalized results are recorded")]
fn the_normalized_results_are_recorded(_world: &mut BenchmarkWorld) -> Result<(), String> {
    unimplemented!("benchmark-comparison e2e: result cassette")
}

#[when("the maintainer replays the recorded run")]
fn the_maintainer_replays_the_recorded_run(_world: &mut BenchmarkWorld) -> Result<(), String> {
    unimplemented!("benchmark-comparison e2e: replay run")
}

#[then("the measurements and their order match the recorded run")]
fn the_measurements_and_order_match(_world: &mut BenchmarkWorld) -> Result<(), String> {
    unimplemented!("benchmark-comparison e2e: replay measurements")
}

#[then("no request leaves the local recording boundary")]
fn no_request_leaves_the_local_boundary(_world: &mut BenchmarkWorld) -> Result<(), String> {
    unimplemented!("benchmark-comparison e2e: local boundary")
}
