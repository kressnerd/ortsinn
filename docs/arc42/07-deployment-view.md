# 7. Deployment View

The benchmark runs on the maintainer's Linux workstation as a local project
tool. It deploys no product service and uses no container runtime.

## Infrastructure level 1

```mermaid
flowchart TB
    subgraph WS[Maintainer workstation - Linux x86_64]
        TC[gnu Rust toolchain 1.99.1<br/>RUSTUP_TOOLCHAIN + Nix zlib + LLVM 23]
        FX[benchmark/tests/fixtures/e2e<br/>manifest, repo, gold, cassettes]
        BIN[target/debug/benchmark]
        COV[target/coverage/non-e2e + e2e<br/>separate instrumentation runs]
        REC[Loopback fake endpoint<br/>127.0.0.1 ephemeral port]
        BIN --> TC
        BIN --> FX
        BIN --> REC
        COV --> BIN
    end
    REC -->|record mode| MODEL[Deterministic fake model endpoint]
    FX -->|read-only| REPO[(Fixture repository at fixture-1)]
```

| Node / Environment | Description |
|---|---|
| Workstation | Linux x86_64 with `rustup`, Cargo, Nix for zlib and LLVM, and Python 3 for the Cobertura helper. |
| Toolchain | `stable-x86_64-unknown-linux-gnu`; the default musl toolchain is unusable here. `env.sh` also resolves `LLVM_COV`/`LLVM_PROFDATA` from `nixpkgs#llvmPackages_23.llvm`. |
| Fixture | `benchmark/tests/fixtures/e2e/manifest.yaml`, `repo/`, `gold/`, and `cassettes/`; committed and fixed. |
| Report output | The caller's `--out` directory (default `reports/benchmark`), not committed. |
| Recordings | The returned-temporary record scenario writes model and result cassettes under its output directory; the committed fixture cassettes are read-only input. |
| Model endpoint | The `@e2e` scenario's fake OpenAI-compatible endpoint on an ephemeral loopback port; used in record mode only. |

The real candidate installs and tool prefixes (`benchmark/.tools/`), the
public repository work area (`benchmark/.work/`), and the committed public
cassettes are `benchmark-run`'s deployment, not this change's.

## Infrastructure level 2

### Workstation

| Property | Value |
|---|---|
| Hardware / VM | The maintainer's development machine; no dedicated benchmark host. |
| OS | Linux x86_64 (kernel 6.18 on the development host). |
| Middleware / Runtime | Cargo/rustc 1.99.1 (gnu), `cargo-llvm-cov` 0.9.0, LLVM 23.1.3, Python 3 for the Cobertura helper. |
| Key software deployed | The `benchmark` binary, the fixture inputs, the loopback fake endpoint, and the coverage scripts. |

### Fixture and recordings

| Path | Content | Pin / identity |
|---|---|---|
| `benchmark/tests/fixtures/e2e/repo` | Tiny Python checkout | `fixture-repo`, revision `fixture-1`, commit `1111…`, 120 code lines |
| `benchmark/tests/fixtures/e2e/gold` | Four JSONL gold files | Versions 1; fingerprints `sha256:fixture-typo-v1`, `sha256:fixture-nl-v1`, `sha256:fixture-explanation-v1`, `sha256:fixture-concept-v1` |
| `benchmark/tests/fixtures/e2e/cassettes` | Result and model cassettes | `sha256:fixture-results-v1`, `sha256:fixture-model-v1` |
| `benchmark/evidence/visual` | Committed terminal transcripts | Two `@e2e` scenario transcripts |
| `target/coverage/{non-e2e,e2e}` | Per-scope cargo-llvm-cov targets | Distinct instrumentation runs |

### Coverage and evidence

`env.sh` supplies the toolchain, zlib, and LLVM 23 environment to
`run-tests.sh`, `setup.sh`, `measure-coverage.sh`, and `merge-coverages.sh`.
`measure-coverage.sh` runs the regular and e2e scopes with separate
`CARGO_TARGET_DIR`s and writes `GIVN_COVERAGE_NON_E2E_PATH` and
`GIVN_COVERAGE_E2E_PATH`; `merge-coverages.sh` merges the two Cobertura files
into `GIVN_COVERAGE_OUTPUT_PATH` and probes production files. Visual
transcripts are written to `${GIVN_EVIDENCE_DIR:-benchmark/evidence/visual}` so
they survive change archival.
