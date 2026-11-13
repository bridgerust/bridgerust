use anyhow::{Context, Result, bail, ensure};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWriteExt};
use tokio::process::Command;
use tokio::time::timeout;

const VERSION: u32 = 1;
const DEFAULT_TIMEOUT_MS: u64 = 30_000;
const STDOUT_LIMIT: u64 = 32 * 1024 * 1024;
const STDERR_LIMIT: u64 = 1024 * 1024;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    version: u32,
    cases: PathBuf,
    targets: Vec<Target>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Target {
    name: String,
    command: Vec<String>,
    cwd: Option<PathBuf>,
    timeout_ms: Option<u64>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
enum Outcome {
    Ok(Value),
    Error(ReportedError),
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct ReportedError {
    code: String,
    #[serde(flatten)]
    details: BTreeMap<String, Value>,
}

impl Outcome {
    fn validate(&self) -> Result<()> {
        if let Self::Error(error) = self {
            ensure!(!error.code.trim().is_empty(), "error code cannot be empty");
        }
        Ok(())
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    id: String,
    input: Value,
    expected: BTreeMap<String, Outcome>,
}

#[derive(Serialize)]
struct Request<'a> {
    id: &'a str,
    input: &'a Value,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Response {
    id: String,
    outcome: Outcome,
}

#[derive(Serialize)]
struct Report<'a> {
    version: u32,
    case_count: usize,
    passed: bool,
    targets: Vec<TargetReport<'a>>,
}

#[derive(Serialize)]
struct TargetReport<'a> {
    target: &'a str,
    passed: bool,
    checked: usize,
    mismatches: Vec<Mismatch<'a>>,
    errors: Vec<String>,
}

#[derive(Serialize)]
struct Mismatch<'a> {
    id: &'a str,
    input: &'a Value,
    expected: &'a Outcome,
    actual: Outcome,
}

pub async fn handle(manifest_path: &Path, report_path: Option<&Path>) -> Result<()> {
    let manifest: Manifest = serde_json::from_str(&read(manifest_path)?)
        .with_context(|| format!("invalid manifest {}", manifest_path.display()))?;
    validate_manifest(&manifest)?;
    let root = manifest_path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let cases_path = root.join(&manifest.cases);
    let cases = load_cases(&read(&cases_path)?, &manifest.targets)
        .with_context(|| format!("invalid cases file {}", cases_path.display()))?;
    if let Some(path) = report_path {
        check_report_path(path, manifest_path, &cases_path)?;
    }

    let mut input = Vec::new();
    for case in &cases {
        serde_json::to_writer(
            &mut input,
            &Request {
                id: &case.id,
                input: &case.input,
            },
        )?;
        input.push(b'\n');
    }

    let mut targets = Vec::with_capacity(manifest.targets.len());
    for target in &manifest.targets {
        let result = match run_target(target, root, &input).await {
            Ok(output) => compare(&target.name, &cases, &output),
            Err(error) => TargetReport {
                target: &target.name,
                passed: false,
                checked: 0,
                mismatches: Vec::new(),
                errors: vec![format!("{error:#}")],
            },
        };
        print_target(&result, cases.len())?;
        targets.push(result);
    }

    let report = Report {
        version: VERSION,
        case_count: cases.len(),
        passed: targets.iter().all(|target| target.passed),
        targets,
    };
    if let Some(path) = report_path {
        let mut json = serde_json::to_vec_pretty(&report)?;
        json.push(b'\n');
        fs::write(path, json)
            .with_context(|| format!("could not write report {}", path.display()))?;
    }
    ensure!(report.passed, "parity check failed");
    Ok(())
}

fn read(path: &Path) -> Result<String> {
    fs::read_to_string(path).with_context(|| format!("could not read {}", path.display()))
}

fn validate_manifest(manifest: &Manifest) -> Result<()> {
    ensure!(
        manifest.version == VERSION,
        "unsupported manifest version {}",
        manifest.version
    );
    ensure!(
        !manifest.cases.as_os_str().is_empty(),
        "cases path cannot be empty"
    );
    ensure!(!manifest.targets.is_empty(), "manifest has no targets");
    let mut names = HashSet::new();
    for target in &manifest.targets {
        ensure!(
            !target.name.trim().is_empty(),
            "target name cannot be empty"
        );
        ensure!(
            names.insert(&target.name),
            "duplicate target {:?}",
            target.name
        );
        ensure!(
            target
                .command
                .first()
                .is_some_and(|command| !command.trim().is_empty()),
            "target {:?} has no command",
            target.name
        );
        ensure!(
            target.timeout_ms != Some(0),
            "target {:?} timeout must be positive",
            target.name
        );
    }
    Ok(())
}

fn load_cases(content: &str, targets: &[Target]) -> Result<Vec<Case>> {
    let mut cases = Vec::new();
    let mut ids = HashSet::new();
    for (index, line) in content.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let case: Case = serde_json::from_str(line)
            .with_context(|| format!("invalid JSONL case on line {}", index + 1))?;
        ensure!(
            !case.id.trim().is_empty(),
            "case ID on line {} is empty",
            index + 1
        );
        ensure!(
            ids.insert(case.id.clone()),
            "duplicate case ID {:?}",
            case.id
        );
        for target in targets {
            ensure!(
                case.expected.contains_key(&target.name),
                "case {:?} has no expectation for target {:?}",
                case.id,
                target.name
            );
        }
        for (target, outcome) in &case.expected {
            outcome.validate().with_context(|| {
                format!("case {:?} has invalid expectation for {target:?}", case.id)
            })?;
        }
        cases.push(case);
    }
    ensure!(!cases.is_empty(), "cases file has no cases");
    Ok(cases)
}

fn check_report_path(path: &Path, manifest: &Path, cases: &Path) -> Result<()> {
    if let Ok(resolved) = path.canonicalize() {
        ensure!(
            resolved != manifest.canonicalize()? && resolved != cases.canonicalize()?,
            "report path must differ from the manifest and cases paths"
        );
    }
    Ok(())
}

async fn run_target(target: &Target, root: &Path, input: &[u8]) -> Result<String> {
    let cwd = root.join(target.cwd.as_deref().unwrap_or_else(|| Path::new(".")));
    let cwd = cwd
        .canonicalize()
        .with_context(|| format!("could not resolve runner directory {}", cwd.display()))?;
    let program = Path::new(&target.command[0]);
    let program = if program.is_relative() && program.components().count() > 1 {
        cwd.join(program)
    } else {
        program.to_path_buf()
    };
    let mut child = Command::new(&program)
        .args(&target.command[1..])
        .current_dir(cwd)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .with_context(|| format!("could not start runner {}", program.display()))?;

    let mut stdin = child.stdin.take().expect("piped stdin");
    let stdout = child.stdout.take().expect("piped stdout");
    let stderr = child.stderr.take().expect("piped stderr");
    let timeout_ms = target.timeout_ms.unwrap_or(DEFAULT_TIMEOUT_MS);
    let result = timeout(Duration::from_millis(timeout_ms), async {
        tokio::try_join!(
            async move { Ok(stdin.write_all(input).await) },
            read_output(stdout, "stdout", STDOUT_LIMIT),
            read_output(stderr, "stderr", STDERR_LIMIT),
            async { child.wait().await.context("could not wait for runner") }
        )
    })
    .await;
    let (written, stdout, stderr, status) = match result {
        Ok(Ok(output)) => output,
        Ok(Err(error)) => {
            let _ = child.kill().await;
            return Err(error);
        }
        Err(_) => {
            let _ = child.kill().await;
            bail!("runner timed out after {timeout_ms} ms");
        }
    };

    if !status.success() {
        let stderr = String::from_utf8_lossy(&stderr);
        if stderr.trim().is_empty() {
            bail!("runner exited with {status}");
        }
        bail!("runner exited with {status}\n{}", stderr.trim_end());
    }
    written.context("could not send all cases to the runner")?;
    String::from_utf8(stdout).context("runner stdout was not UTF-8")
}

async fn read_output(stream: impl AsyncRead + Unpin, name: &str, limit: u64) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    stream
        .take(limit + 1)
        .read_to_end(&mut bytes)
        .await
        .with_context(|| format!("could not read runner {name}"))?;
    ensure!(
        bytes.len() as u64 <= limit,
        "runner {name} exceeded the {limit}-byte limit"
    );
    Ok(bytes)
}

fn compare<'a>(target: &'a str, cases: &'a [Case], output: &str) -> TargetReport<'a> {
    let ids: HashSet<&str> = cases.iter().map(|case| case.id.as_str()).collect();
    let mut outcomes = HashMap::new();
    let mut errors = Vec::new();

    for (index, line) in output.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let response = serde_json::from_str::<Response>(line)
            .map_err(anyhow::Error::from)
            .and_then(|response| {
                response.outcome.validate()?;
                Ok(response)
            });
        match response {
            Ok(response) if !ids.contains(response.id.as_str()) => {
                errors.push(format!("unknown case ID {:?}", response.id));
            }
            Ok(response) => {
                let id = response.id;
                match outcomes.entry(id) {
                    std::collections::hash_map::Entry::Occupied(entry) => {
                        errors.push(format!("duplicate response for case {:?}", entry.key()));
                    }
                    std::collections::hash_map::Entry::Vacant(entry) => {
                        entry.insert(response.outcome);
                    }
                }
            }
            Err(error) => {
                errors.push(format!(
                    "invalid JSONL response on line {}: {error}",
                    index + 1
                ));
            }
        }
    }

    let checked = outcomes.len();
    let mut mismatches = Vec::new();
    for case in cases {
        match outcomes.remove(&case.id) {
            Some(actual) if actual != case.expected[target] => mismatches.push(Mismatch {
                id: &case.id,
                input: &case.input,
                expected: &case.expected[target],
                actual,
            }),
            Some(_) => {}
            None => errors.push(format!("missing response for case {:?}", case.id)),
        }
    }
    TargetReport {
        target,
        passed: errors.is_empty() && mismatches.is_empty(),
        checked,
        mismatches,
        errors,
    }
}

fn print_target(report: &TargetReport<'_>, case_count: usize) -> Result<()> {
    let status = if report.passed { "PASS" } else { "FAIL" };
    println!(
        "{status} {} ({}/{case_count} cases checked)",
        report.target, report.checked
    );
    for error in &report.errors {
        println!("  {}: {error}", report.target);
    }
    for mismatch in &report.mismatches {
        println!("  {}: case {:?}", report.target, mismatch.id);
        println!("    input:    {}", mismatch.input);
        println!(
            "    expected: {}",
            serde_json::to_string(mismatch.expected)?
        );
        println!("    actual:   {}", serde_json::to_string(&mismatch.actual)?);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const CASES: &str = r#"
{"id":"value","input":{"timestamp":"1970-01-01"},"expected":{"python":{"ok":{"a":1,"b":2}}}}
{"id":"error","input":null,"expected":{"python":{"error":{"code":"BAD_DATE","message":"invalid date"}}}}
"#;

    fn target() -> Target {
        Target {
            name: "python".into(),
            command: vec!["python".into()],
            cwd: None,
            timeout_ms: None,
        }
    }

    #[test]
    fn validates_cases_before_starting_runners() {
        let targets = [target()];
        assert!(load_cases("", &targets).is_err());
        assert!(load_cases(&CASES.repeat(2), &targets).is_err());
        assert!(load_cases(&CASES.replace("python", "nodejs"), &targets).is_err());
        assert!(load_cases(&CASES.replace("BAD_DATE", ""), &targets).is_err());
    }

    #[test]
    fn comparison_is_independent_of_object_and_response_order() {
        let cases = load_cases(CASES, &[target()]).unwrap();
        let output = r#"
{"id":"error","outcome":{"error":{"message":"invalid date","code":"BAD_DATE"}}}
{"id":"value","outcome":{"ok":{"b":2,"a":1}}}
"#;
        let report = compare("python", &cases, output);
        assert!(report.passed);
        assert_eq!(report.checked, 2);
    }

    #[test]
    fn reports_difference_and_bad_protocol_together() {
        let cases = load_cases(CASES, &[target()]).unwrap();
        let output = r#"
{"id":"value","outcome":{"ok":{"a":2,"b":2}}}
{"id":"value","outcome":{"ok":{"a":1,"b":2}}}
{"id":"other","outcome":{"ok":null}}
runner debug output
"#;
        let report = compare("python", &cases, output);
        assert!(!report.passed);
        assert_eq!(report.checked, 1);
        assert_eq!(report.mismatches[0].id, "value");
        assert_eq!(report.mismatches[0].input, &cases[0].input);
        assert!(
            report
                .errors
                .iter()
                .any(|error| error.contains("duplicate"))
        );
        assert!(report.errors.iter().any(|error| error.contains("unknown")));
        assert!(
            report
                .errors
                .iter()
                .any(|error| error.contains("invalid JSONL"))
        );
        assert!(report.errors.iter().any(|error| error.contains("missing")));
    }

    #[test]
    fn outcome_requires_a_single_variant_and_error_code() {
        assert!(serde_json::from_str::<Outcome>(r#"{"ok":null}"#).is_ok());
        assert!(serde_json::from_str::<Outcome>(r#"{"ok":1,"error":{"code":"x"}}"#).is_err());
        assert!(serde_json::from_str::<Outcome>(r#"{"error":{"message":"x"}}"#).is_err());
    }

    #[tokio::test]
    async fn runner_output_is_bounded() {
        assert_eq!(read_output(&b"abc"[..], "stdout", 3).await.unwrap(), b"abc");
        let error = read_output(&b"abcd"[..], "stdout", 3).await.unwrap_err();
        assert!(error.to_string().contains("runner stdout exceeded"));
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn runner_receives_all_input_and_stdin_closes() {
        let mut target = target();
        target.command = vec!["sh".into(), "-c".into(), "exec cat".into()];
        let input = b"one\ntwo\n";
        let output = run_target(&target, Path::new("."), input).await.unwrap();
        assert_eq!(output.as_bytes(), input);
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn timeout_and_nonzero_runner_exit_fail() {
        let mut target = target();
        target.command = vec!["sh".into(), "-c".into(), "exec sleep 10".into()];
        target.timeout_ms = Some(25);
        let error = run_target(&target, Path::new("."), b"").await.unwrap_err();
        assert!(error.to_string().contains("timed out after 25 ms"));

        target.timeout_ms = None;
        target.command = vec!["sh".into(), "-c".into(), "echo broken >&2; exit 7".into()];
        let error = run_target(&target, Path::new("."), b"").await.unwrap_err();
        assert!(error.to_string().contains("exit status: 7"));
        assert!(error.to_string().contains("broken"));
    }
}
