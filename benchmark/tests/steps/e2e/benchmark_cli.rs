// E2E CLI steps for benchmark-comparison. These steps drive the real
// `benchmark` binary; the in-process steps live in
// `benchmark/tests/steps/benchmark_comparison.rs`.

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use benchmark::record::cassette::{self, ModelCassette, ResultCassette};
use benchmark::record::proxy::{
    FakeOpenAiEndpoint, FixtureCompletion, is_loopback,
};
use cucumber::{given, then, when};
use serde_json::Value;

use crate::BenchmarkWorld;

const S20_SLUG: &str = "maintainer-runs-the-benchmark-and-reads-the-measured-report";
const S21_SLUG: &str = "maintainer-records-a-run-and-replays-it-without-an-external-call";

/// Metric rows the terminal summary always renders, in report order.
const METRIC_LABELS: &[&str] = &[
    "Typo recall @1",
    "Typo recall @5",
    "Typo no-match rate",
    "Typo coverage",
    "NL recall @5",
    "NL MRR @10",
    "NL query errors",
    "NL coverage",
    "Explanation correctness",
    "Explanation grounding",
    "Explanation coverage",
    "Concept recall @5",
    "Concept-equivalence recall @5",
    "Concept coverage",
    "Concept no-expansion coverage",
];

fn fixture_manifest() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/e2e/manifest.yaml")
}

fn benchmark_binary() -> &'static str {
    env!("CARGO_BIN_EXE_benchmark")
}

fn e2e_root(name: &str) -> Result<PathBuf, String> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("benchmark crate has no workspace root")?
        .join("target")
        .join("benchmark-e2e")
        .join(name);
    if root.exists() {
        fs::remove_dir_all(&root)
            .map_err(|error| format!("clear {}: {error}", root.display()))?;
    }
    fs::create_dir_all(&root).map_err(|error| format!("create {}: {error}", root.display()))?;
    Ok(root)
}

fn run_and_capture(world: &mut BenchmarkWorld, args: &[String]) -> Result<(), String> {
    let output = Command::new(benchmark_binary())
        .args(args)
        .output()
        .map_err(|error| format!("run {}: {error}", benchmark_binary()))?;
    let exit = output.status.code().unwrap_or(-1);
    let mut terminal = String::from_utf8_lossy(&output.stdout).to_string();
    terminal.push_str(&String::from_utf8_lossy(&output.stderr));
    world.e2e_exit = Some(exit);
    world.e2e_terminal = terminal.clone();
    world.e2e_transcript.push_str(&format!(
        "$ benchmark {}\n{terminal}[exit {exit}]\n\n",
        args.join(" ")
    ));
    Ok(())
}

fn base_args(world: &BenchmarkWorld) -> Vec<String> {
    vec![
        "run".to_string(),
        "--manifest".to_string(),
        world.manifest_path.display().to_string(),
        "--mode".to_string(),
        "replay".to_string(),
        "--out".to_string(),
        world.e2e_out.display().to_string(),
    ]
}

fn write_transcript(slug: &str, transcript: &str) -> Result<PathBuf, String> {
    let base = std::env::var_os("GIVN_EVIDENCE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("evidence/visual"));
    let directory = base.join(slug);
    fs::create_dir_all(&directory)
        .map_err(|error| format!("create evidence directory {}: {error}", directory.display()))?;
    let path = directory.join("transcript.txt");
    fs::write(&path, transcript)
        .map_err(|error| format!("write transcript {}: {error}", path.display()))?;
    Ok(path)
}

fn assert_metric_rows(terminal: &str, width: usize) -> Result<(), String> {
    for label in METRIC_LABELS {
        let prefix = format!("  {label}:");
        let rows: Vec<&str> = terminal
            .lines()
            .filter(|line| line.starts_with(&prefix))
            .collect();
        match rows.as_slice() {
            [row] => {
                let length = row.chars().count();
                if length != width {
                    return Err(format!(
                        "metric row {label:?} is {length} columns, expected {width}: {row:?}"
                    ));
                }
                let value = row[prefix.len()..].trim();
                if value.is_empty() {
                    return Err(format!("metric row {label:?} has no value: {row:?}"));
                }
            }
            [] => return Err(format!("terminal has no metric row for {label:?}")),
            rows => {
                return Err(format!(
                    "metric label {label:?} is split across {} lines: {rows:?}",
                    rows.len()
                ));
            }
        }
    }
    Ok(())
}

fn report_json(world: &BenchmarkWorld) -> Result<Value, String> {
    let path = world
        .e2e_report_path
        .as_ref()
        .ok_or("the terminal did not print a report path")?;
    let text = fs::read_to_string(path)
        .map_err(|error| format!("read report {}: {error}", path.display()))?;
    serde_json::from_str(&text).map_err(|error| format!("parse report {}: {error}", path.display()))
}

fn metric_value(report: &Value, family: &str) -> Result<f64, String> {
    report
        .pointer(&format!("/metrics/{family}/aggregate/value"))
        .and_then(Value::as_f64)
        .ok_or_else(|| format!("report has no measured {family} aggregate"))
}

fn assert_value(actual: f64, expected: f64, label: &str) -> Result<(), String> {
    if (actual - expected).abs() > 1e-9 {
        return Err(format!("{label}: expected {expected}, got {actual}"));
    }
    Ok(())
}

#[given("a benchmark fixture with fixed repository revisions, pinned candidate identities, and recorded model responses")]
fn a_benchmark_fixture(world: &mut BenchmarkWorld) -> Result<(), String> {
    let fixture = fixture_manifest();
    for relative in [
        "manifest.yaml",
        "cassettes/results/results.jsonl",
        "cassettes/model/model.jsonl",
        "gold/typo.jsonl",
        "gold/natural_language.jsonl",
        "gold/explanation.jsonl",
        "gold/concept.jsonl",
    ] {
        let path = fixture
            .parent()
            .ok_or("fixture manifest has no parent")?
            .join(relative);
        if !path.is_file() {
            return Err(format!("missing fixture input {}", path.display()));
        }
    }
    world.manifest_path = fixture;
    world.e2e_out = e2e_root("s20-report")?;
    Ok(())
}

#[when("the maintainer runs \"benchmark run\" in the terminal")]
fn the_maintainer_runs_benchmark_run(world: &mut BenchmarkWorld) -> Result<(), String> {
    let args = base_args(world);
    run_and_capture(world, &args)
}

#[then("the command exits with code 0")]
fn the_command_exits_with_code_zero(world: &mut BenchmarkWorld) -> Result<(), String> {
    match world.e2e_exit {
        Some(0) => Ok(()),
        Some(code) => Err(format!(
            "benchmark run exited {code}:\n{}",
            world.e2e_terminal
        )),
        None => Err("benchmark run did not execute".into()),
    }
}

#[then("the terminal shows the four measurement sections and the report path")]
fn the_terminal_shows_sections_and_report_path(world: &mut BenchmarkWorld) -> Result<(), String> {
    let terminal = &world.e2e_terminal;
    for section in [
        "Typo recall",
        "Natural-language retrieval",
        "Explanation correctness",
        "Concept recall",
    ] {
        if !terminal.lines().any(|line| line == section) {
            return Err(format!("terminal has no {section:?} section:\n{terminal}"));
        }
    }
    let printed = terminal
        .lines()
        .find_map(|line| line.strip_prefix("Report: "))
        .ok_or_else(|| format!("terminal has no report path:\n{terminal}"))?;
    let path = PathBuf::from(printed.trim());
    if !path.is_file() {
        return Err(format!("printed report path {} does not exist", path.display()));
    }
    if path.file_name() != Some(std::ffi::OsStr::new("report.json")) {
        return Err(format!("printed report path {} is not report.json", path.display()));
    }
    world.e2e_report_path = Some(path);
    Ok(())
}

#[then("the report at that path states typo recall at rank one 0.60, natural-language recall at rank five 0.50, explanation correctness 0.50, and concept recall at rank five 0.40")]
fn the_report_states_the_measured_values(world: &mut BenchmarkWorld) -> Result<(), String> {
    let report = report_json(world)?;
    assert_value(metric_value(&report, "typo")?, 0.60, "typo recall @1")?;
    assert_value(
        metric_value(&report, "natural_language")?,
        0.50,
        "natural-language recall @5",
    )?;
    assert_value(
        metric_value(&report, "explanation")?,
        0.50,
        "explanation correctness",
    )?;
    assert_value(metric_value(&report, "concept")?, 0.40, "concept recall @5")?;
    Ok(())
}

#[then("the terminal renders every metric label and value on one line at width 80")]
fn the_terminal_renders_at_width_80(world: &mut BenchmarkWorld) -> Result<(), String> {
    render_at_width(world, 80)
}

#[then("the terminal renders every metric label and value on one line at width 160")]
fn the_terminal_renders_at_width_160(world: &mut BenchmarkWorld) -> Result<(), String> {
    render_at_width(world, 160)?;
    write_transcript(S20_SLUG, &world.e2e_transcript)?;
    Ok(())
}

fn render_at_width(world: &mut BenchmarkWorld, width: usize) -> Result<(), String> {
    let mut args = base_args(world);
    args.push("--width".to_string());
    args.push(width.to_string());
    run_and_capture(world, &args)?;
    if world.e2e_exit != Some(0) {
        return Err(format!(
            "benchmark run --width {width} exited {:?}:\n{}",
            world.e2e_exit, world.e2e_terminal
        ));
    }
    assert_metric_rows(&world.e2e_terminal, width)?;
    Ok(())
}

#[given("a local fake OpenAI-compatible endpoint and the fixture candidates")]
fn a_local_fake_endpoint(world: &mut BenchmarkWorld) -> Result<(), String> {
    world.manifest_path = fixture_manifest();
    world.e2e_out = e2e_root("s21-run")?;
    let model_path = world
        .manifest_path
        .parent()
        .ok_or("fixture manifest has no parent")?
        .join("cassettes/model/model.jsonl");
    let models = ModelCassette::load(&model_path)
        .map_err(|error| format!("load fixture model cassette: {error:#}"))?;
    let mut completions = BTreeMap::new();
    for record in models.entries() {
        if record.layer != "explanation" {
            continue;
        }
        let completion: FixtureCompletion = serde_json::from_value(record.response.clone())
            .map_err(|error| format!("parse fixture completion {}: {error}", record.key))?;
        completions.insert(record.query.clone(), completion);
    }
    if completions.is_empty() {
        return Err("fixture model cassette has no explanation responses".into());
    }
    world.e2e_endpoint = Some(
        FakeOpenAiEndpoint::start(completions)
            .map_err(|error| format!("start local fixture endpoint: {error}"))?,
    );
    Ok(())
}

#[when("the maintainer records the benchmark run against that endpoint")]
fn the_maintainer_records_the_run(world: &mut BenchmarkWorld) -> Result<(), String> {
    let base_url = world
        .e2e_endpoint
        .as_ref()
        .ok_or("the local fixture endpoint is not running")?
        .base_url();
    let args = vec![
        "run".to_string(),
        "--manifest".to_string(),
        world.manifest_path.display().to_string(),
        "--mode".to_string(),
        "record".to_string(),
        "--model-endpoint".to_string(),
        base_url,
        "--out".to_string(),
        world.e2e_out.display().to_string(),
        "--width".to_string(),
        "80".to_string(),
    ];
    run_and_capture(world, &args)?;
    if world.e2e_exit != Some(0) {
        return Err(format!(
            "record run exited {:?}:\n{}",
            world.e2e_exit, world.e2e_terminal
        ));
    }
    let report_path = world.e2e_out.join("report.json");
    let text = fs::read_to_string(&report_path)
        .map_err(|error| format!("read record report {}: {error}", report_path.display()))?;
    let report: Value = serde_json::from_str(&text)
        .map_err(|error| format!("parse record report {}: {error}", report_path.display()))?;
    world.e2e_report_path = Some(report_path);
    world.e2e_record_report = Some(report);
    world.e2e_record_requests = world
        .e2e_endpoint
        .as_ref()
        .ok_or("the local fixture endpoint is not running")?
        .request_count();
    Ok(())
}

#[then("the model requests are recorded and the model cassette fingerprint is written")]
fn the_model_requests_are_recorded(world: &mut BenchmarkWorld) -> Result<(), String> {
    let endpoint = world
        .e2e_endpoint
        .as_ref()
        .ok_or("the local fixture endpoint is not running")?;
    let model_path = world.e2e_out.join("cassettes/model/model.jsonl");
    if !model_path.is_file() {
        return Err(format!("model cassette {} was not written", model_path.display()));
    }
    let models = ModelCassette::load(&model_path)
        .map_err(|error| format!("load recorded model cassette: {error:#}"))?;
    if models.entry_count() == 0 {
        return Err("recorded model cassette is empty".into());
    }
    let requests = endpoint.requests();
    if requests.len() != models.entry_count() {
        return Err(format!(
            "endpoint saw {} request(s) but {} model response(s) were recorded",
            requests.len(),
            models.entry_count()
        ));
    }
    for request in &requests {
        if !request.path.starts_with("/v1/") {
            return Err(format!("unexpected request path {}", request.path));
        }
        if !is_loopback(request.peer) {
            return Err(format!(
                "request left loopback from peer {}",
                request.peer
            ));
        }
    }
    let report = world
        .e2e_record_report
        .as_ref()
        .ok_or("the record run did not produce a report")?;
    let printed = report
        .pointer("/reproduction/model_cassette_fingerprint")
        .and_then(Value::as_str)
        .ok_or("record report has no model cassette fingerprint")?;
    let computed = cassette::fingerprint(&model_path)
        .map_err(|error| format!("fingerprint model cassette: {error:#}"))?;
    if printed != computed {
        return Err(format!(
            "model cassette fingerprint {printed} does not match recorded content {computed}"
        ));
    }
    if !printed.starts_with("sha256:") {
        return Err(format!("model cassette fingerprint {printed} is not a sha256 value"));
    }
    Ok(())
}

#[then("the normalized results are recorded")]
fn the_normalized_results_are_recorded(world: &mut BenchmarkWorld) -> Result<(), String> {
    let results_path = world.e2e_out.join("cassettes/results/results.jsonl");
    if !results_path.is_file() {
        return Err(format!(
            "result cassette {} was not written",
            results_path.display()
        ));
    }
    let recorded = ResultCassette::load(&results_path)
        .map_err(|error| format!("load recorded result cassette: {error:#}"))?;
    let committed_path = world
        .manifest_path
        .parent()
        .ok_or("fixture manifest has no parent")?
        .join("cassettes/results/results.jsonl");
    let committed = ResultCassette::load(&committed_path)
        .map_err(|error| format!("load committed result cassette: {error:#}"))?;
    if recorded.entry_count() != committed.entry_count() {
        return Err(format!(
            "recorded {} normalized result(s), expected {}",
            recorded.entry_count(),
            committed.entry_count()
        ));
    }
    for expected in committed.entries() {
        let Some(actual) = recorded.get(
            &expected.candidate,
            &expected.metric,
            &expected.task,
            &expected.arm,
        ) else {
            return Err(format!(
                "recorded results are missing {}/{}/{}/{}",
                expected.candidate, expected.metric, expected.task, expected.arm
            ));
        };
        let expected_json = serde_json::to_value(expected)
            .map_err(|error| format!("serialize committed result: {error}"))?;
        let actual_json = serde_json::to_value(actual)
            .map_err(|error| format!("serialize recorded result: {error}"))?;
        if expected_json != actual_json {
            return Err(format!(
                "recorded normalized result differs for {}/{}/{}/{}",
                expected.candidate, expected.metric, expected.task, expected.arm
            ));
        }
    }
    let report = world
        .e2e_record_report
        .as_ref()
        .ok_or("the record run did not produce a report")?;
    let printed = report
        .pointer("/reproduction/result_cassette_fingerprint")
        .and_then(Value::as_str)
        .ok_or("record report has no result cassette fingerprint")?;
    let computed = cassette::fingerprint(&results_path)
        .map_err(|error| format!("fingerprint result cassette: {error:#}"))?;
    if printed != computed {
        return Err(format!(
            "result cassette fingerprint {printed} does not match recorded content {computed}"
        ));
    }
    Ok(())
}

#[when("the maintainer replays the recorded run")]
fn the_maintainer_replays_the_recorded_run(world: &mut BenchmarkWorld) -> Result<(), String> {
    let before = world
        .e2e_endpoint
        .as_ref()
        .ok_or("the local fixture endpoint is not running")?
        .request_count();
    let args = base_args(world);
    run_and_capture(world, &args)?;
    if world.e2e_exit != Some(0) {
        return Err(format!(
            "replay run exited {:?}:\n{}",
            world.e2e_exit, world.e2e_terminal
        ));
    }
    let after = world
        .e2e_endpoint
        .as_ref()
        .ok_or("the local fixture endpoint is not running")?
        .request_count();
    if after != before || after != world.e2e_record_requests {
        return Err(format!(
            "replay changed the endpoint request count: {before} -> {after} (record run: {})",
            world.e2e_record_requests
        ));
    }
    Ok(())
}

#[then("the measurements and their order match the recorded run")]
fn the_measurements_and_order_match(world: &mut BenchmarkWorld) -> Result<(), String> {
    let recorded = world
        .e2e_record_report
        .as_ref()
        .ok_or("no recorded report to compare against")?
        .clone();
    let replayed = report_json(world)?;
    for pointer in ["/metrics", "/gaps"] {
        let recorded_value = recorded
            .pointer(pointer)
            .ok_or_else(|| format!("recorded report has no {pointer}"))?;
        let replayed_value = replayed
            .pointer(pointer)
            .ok_or_else(|| format!("replayed report has no {pointer}"))?;
        if recorded_value != replayed_value {
            return Err(format!("replay changed {pointer}"));
        }
    }
    let order = |report: &Value| -> Result<Vec<String>, String> {
        report
            .get("candidates")
            .and_then(Value::as_array)
            .ok_or_else(|| "report has no candidates".to_string())?
            .iter()
            .map(|candidate| {
                candidate
                    .get("id")
                    .and_then(Value::as_str)
                    .map(str::to_string)
                    .ok_or_else(|| "candidate has no id".to_string())
            })
            .collect()
    };
    if order(&recorded)? != order(&replayed)? {
        return Err("replay changed the candidate order".into());
    }
    assert_value(
        replayed
            .pointer("/metrics/typo/aggregate/value")
            .and_then(Value::as_f64)
            .ok_or("replay report has no typo aggregate")?,
        0.60,
        "replayed typo recall @1",
    )?;
    Ok(())
}

/// Shared with the in-process S6 scenario, which uses the same step text.
/// The e2e world owns a live fixture endpoint, so it asserts the recorded
/// request boundary; the in-process scenario reports zero external requests.
pub(crate) fn assert_recording_boundary(world: &mut BenchmarkWorld) -> Result<(), String> {
    let endpoint = world
        .e2e_endpoint
        .as_ref()
        .ok_or("the local fixture endpoint is not running")?;
    if !endpoint.base_url().starts_with("http://127.0.0.1:") {
        return Err(format!(
            "fixture endpoint is not loopback: {}",
            endpoint.base_url()
        ));
    }
    let requests = endpoint.requests();
    if requests.len() != world.e2e_record_requests {
        return Err(format!(
            "replay sent {} new request(s)",
            requests.len() - world.e2e_record_requests
        ));
    }
    if let Some(request) = requests.iter().find(|request| !is_loopback(request.peer)) {
        return Err(format!(
            "request reached a non-loopback peer {}",
            request.peer
        ));
    }
    let replayed = report_json(world)?;
    if replayed.get("run").and_then(|run| run.get("mode")).and_then(Value::as_str) != Some("replay")
    {
        return Err("replay report does not state replay mode".into());
    }
    let external = replayed
        .pointer("/run/external_requests")
        .and_then(Value::as_u64)
        .ok_or("replay report has no external request count")?;
    if external != 0 {
        return Err(format!(
            "replay report recorded {external} external request(s)"
        ));
    }
    write_transcript(S21_SLUG, &world.e2e_transcript)?;
    Ok(())
}
