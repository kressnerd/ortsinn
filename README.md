# ortsinn
A sense of place for code agents.

<!-- givn:begin:development -->
## Quick start

Prerequisites: Rust with the pinned GNU toolchain and `libz.so.1`; `env.sh`
resolves both. Run the executable specification from the repository root:

```sh
./run-tests.sh
./run-tests.sh --e2e
```
<!-- givn:end:development -->

<!-- givn:begin:run -->
## Usage

The benchmark harness reads a fixed manifest and writes `report.json` and
`report.md`:

```sh
. ./env.sh
cargo run -p benchmark -- run \
  --manifest benchmark/tests/fixtures/e2e/manifest.yaml \
  --mode replay \
  --out reports/benchmark
```

`--mode record --model-endpoint <url>` records model responses against a local
OpenAI-compatible endpoint. `--width` sets the terminal summary width.

The committed fixture is the input set available in this change; real
reference repositories and candidate adapters are delivered by the follow-up
change `benchmark-run`.
<!-- givn:end:run -->

<!-- givn:begin:coverage -->
Coverage measures the regular and e2e scopes separately (the e2e scope
includes the CLI child) and merges the two Cobertura reports:

```sh
./measure-coverage.sh
./merge-coverages.sh
```

`env.sh` resolves the LLVM 23 tools that match rustc; the 21.1.8 package
cannot read rustc's raw profile format.
<!-- givn:end:coverage -->
