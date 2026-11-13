"""Record expected outcomes from the checked-in reference implementations."""

import json
import subprocess
from pathlib import Path


root = Path(__file__).resolve().parent
manifest = json.loads((root / "legacy-parity.json").read_text(encoding="utf-8"))
inputs = (root / "inputs.jsonl").read_text(encoding="utf-8")
cases = [json.loads(line) for line in inputs.splitlines()]
ids = {case["id"] for case in cases}
if not ids or len(ids) != len(cases):
    raise SystemExit("Reference cases must have unique IDs")

results = {}
for target in manifest["targets"]:
    name = target["name"]
    run = subprocess.run(
        target["command"], cwd=root, input=inputs, capture_output=True,
        check=True, encoding="utf-8", timeout=30,
    )
    output = [json.loads(line) for line in run.stdout.splitlines()]
    outcomes = {entry["id"]: entry["outcome"] for entry in output}
    if set(outcomes) != ids or len(output) != len(cases):
        raise SystemExit(f"Reference runner {name} returned the wrong case IDs")
    results[name] = outcomes

for case in cases:
    expected = {target: outcomes[case["id"]] for target, outcomes in results.items()}
    print(json.dumps({**case, "expected": expected}, separators=(",", ":")))
