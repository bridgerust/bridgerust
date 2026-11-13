# Date parsing compatibility pilot

Replace two example date parsers with one Rust core while keeping the public
`normalize_date` and `normalizeDate` functions and their error classes. The
Python reference interprets a missing UTC offset as UTC. The TypeScript
reference rejects it with `MISSING_OFFSET`. The Rust-backed wrappers preserve
that difference. The references are sample legacy code for this pilot.

`inputs.jsonl` defines the scenarios. `cases.jsonl` records each reference
implementation's observed result or public error code. The 33 cases cover leap
years, UTC offset conversion, calendar boundaries, and malformed inputs. Both
manifests use the same recorded cases. Review changes to the recorded results as
changes to the public API.

## Run the pilot

Use the workspace Rust toolchain, Node.js 24 with npm, Python 3.12 or later,
and Maturin in the build environment. The workflow builds a wheel and a native
npm tarball for the current platform, then installs them into separate test
environments.

1. Set `CARGO_PROFILE_RELEASE_STRIP=none`, then run
   `maturin build --locked --release --out <wheel-dir>` from
   `examples/date-parity/python`. The override keeps the wheel loadable on macOS.
2. Run `npm run build` and `npm pack --pack-destination <npm-dir>` from
   `examples/date-parity/nodejs`.
3. Install the wheel into a clean virtual environment and activate it so
   `python` uses that interpreter. Install the tarball in a separate temporary
   npm directory with `npm install --ignore-scripts <tarball>`. Set `NODE_PATH`
   to that directory's `node_modules` so the example wrapper uses the installed
   package.
4. Run the two manifests from the workspace root.

```sh
cargo run --locked -p bridge -- parity --manifest examples/date-parity/legacy-parity.json
cargo run --locked -p bridge -- parity --manifest examples/date-parity/parity.json
```

The first command checks the snapshots against the references. The second
exercises the public wrappers against the installed Rust packages. To record
updated expectations from the references, run the following and review the diff.

```sh
python examples/date-parity/capture_reference.py > examples/date-parity/cases.jsonl
```

To see a failing comparison, set `BRIDGE_PARITY_DEMO_REGRESSION=1` for the
second command. The Node runner then accepts the `missing-offset` input,
and the CLI exits unsuccessfully with the expected error and actual value.

## Use this pattern on a real module

Run the baseline capture against pinned versions of the existing packages.
Cover the results, public errors, and serialization details that callers rely
on. Install candidate artifacts in clean environments and replay the same
cases there. Benchmark the actual Python and Node calls on representative
inputs before making any performance claim. Keep the old implementation
available through the first rollout so one service or worker can switch at a
time and roll back independently.
