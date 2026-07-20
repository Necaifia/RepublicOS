"""
pipeline.py — Auto-Evolution Pipeline (Day 5)
Part of RepublicOS Skill Evolution Pipeline

Orchestrates all stages: Discovery → Analysis → Research → Evolution.
Supports full runs, per-stage runs, checkpoint resume, and scheduling.
"""

import json
import os
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path
from typing import Dict, List, Optional

SCRIPT_DIR = Path(__file__).parent.resolve()
REGISTRY_DIR = SCRIPT_DIR / "registry"
PIPELINE_LOG = REGISTRY_DIR / "pipeline-log.json"
PIPELINE_STATE = REGISTRY_DIR / "pipeline-state.json"

STAGES_ORDER = ["discovery", "analyzer", "research", "evolution", "store"]
STAGE_SCRIPTS = {
    "discovery": str(SCRIPT_DIR / "discovery.py"),
    "analyzer": str(SCRIPT_DIR / "analyzer.py"),
    "research": str(SCRIPT_DIR / "research.py"),
    "evolution": str(SCRIPT_DIR / "evolution.py"),
    "store": str(SCRIPT_DIR / "store.py"),
}


def log(msg: str, level: str = "INFO"):
    ts = datetime.now(timezone.utc).strftime("%H:%M:%S")
    print(f"[{ts}] [PIPELINE] {msg}")


def load_json(path: Path):
    if path.exists():
        with open(path, encoding="utf-8") as f:
            return json.load(f)
    return {}


def save_json(path: Path, data):
    path.parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        json.dump(data, f, indent=2, ensure_ascii=False)


def run_stage(name: str, script: str, timeout: int = 600) -> bool:
    log(f"Starting stage: {name}")
    start = time.time()
    try:
        result = subprocess.run(
            [sys.executable, script],
            capture_output=True, text=True, timeout=timeout,
        )
        elapsed = time.time() - start
        if result.returncode == 0:
            log(f"Stage '{name}' completed in {elapsed:.1f}s")
            return True
        else:
            log(f"Stage '{name}' FAILED (exit={result.returncode}, {elapsed:.1f}s)", "WARN")
            for line in result.stderr.strip().splitlines()[-5:]:
                log(f"  stderr: {line}", "WARN")
            return False
    except subprocess.TimeoutExpired:
        log(f"Stage '{name}' TIMEOUT after {timeout}s", "WARN")
        return False
    except Exception as e:
        log(f"Stage '{name}' ERROR: {e}", "WARN")
        return False


def get_skills_count() -> int:
    reg = load_json(REGISTRY_DIR / "skills-registry.json")
    return len(reg) if reg else 0


def get_evolved_count() -> int:
    log_file = REGISTRY_DIR / "evolved" / "evolution-log.json"
    data = load_json(log_file)
    return data.get("total_evolved", 0) if data else 0


def run_pipeline(stages: Optional[List[str]] = None, timeout_per_stage: int = 600,
                 notify: bool = False):
    log("=" * 60)
    log("Skill Evolution Pipeline")
    log("=" * 60)

    if stages is None:
        stages = STAGES_ORDER

    state = load_json(PIPELINE_STATE)
    if not state:
        state = {
            "started_at": None,
            "completed_at": None,
            "stages": {},
            "total_runs": 0,
        }

    state["started_at"] = datetime.now(timezone.utc).isoformat()
    state["total_runs"] += 1

    results = {}
    all_passed = True

    for stage in stages:
        if stage not in STAGE_SCRIPTS:
            log(f"Unknown stage: {stage}", "WARN")
            results[stage] = "skipped"
            continue

        script = STAGE_SCRIPTS[stage]
        success = run_stage(stage, script, timeout=timeout_per_stage)

        results[stage] = "passed" if success else "failed"
        state["stages"][stage] = {
            "last_run": datetime.now(timezone.utc).isoformat(),
            "status": "passed" if success else "failed",
        }

        if not success:
            all_passed = False
            if stage == "discovery":
                log("Discovery failed - subsequent stages may have stale data", "WARN")
            elif stage == "analyzer":
                log("Analysis failed - subsequent stages may be affected", "WARN")

    state["completed_at"] = datetime.now(timezone.utc).isoformat()

    # Gather final metrics
    skills_count = get_skills_count()
    evolved_count = get_evolved_count()

    pipeline_run = {
        "run_number": state["total_runs"],
        "started_at": state["started_at"],
        "completed_at": state["completed_at"],
        "stages": results,
        "skills_count": skills_count,
        "evolved_count": evolved_count,
        "all_passed": all_passed,
    }

    # Append to pipeline log
    existing_log = load_json(PIPELINE_LOG)
    if not isinstance(existing_log, list):
        existing_log = []
    existing_log.append(pipeline_run)
    save_json(PIPELINE_LOG, existing_log)
    save_json(PIPELINE_STATE, state)

    log("\n" + "=" * 60)
    log("Pipeline Summary")
    for stage, result in results.items():
        icon = "+" if result == "passed" else "x" if result == "failed" else "-"
        log(f"  [{icon}] {stage}: {result}")
    log(f"  Skills: {skills_count} | Evolved: {evolved_count}")
    log(f"  Overall: {'PASSED' if all_passed else 'FAILED'}")
    log("=" * 60)

    # Notify
    if notify:
        try:
            notifier_script = SCRIPT_DIR / "notifier.py"
            if notifier_script.exists():
                event = "success" if all_passed else "failure"
                # Inline import to avoid circular dependency at module level
                import importlib.util
                spec = importlib.util.spec_from_file_location("notifier_mod", notifier_script)
                if spec and spec.loader:
                    mod = importlib.util.module_from_spec(spec)
                    spec.loader.exec_module(mod)
                    mod.notify(event, pipeline_run)
        except Exception as e:
            log(f"Notification failed: {e}", "WARN")

    return all_passed


def main():
    import argparse
    parser = argparse.ArgumentParser(description="RepublicOS Skill Evolution Pipeline")
    parser.add_argument("--stages", nargs="+", choices=STAGES_ORDER + ["all"],
                        default=["all"], help="Stages to run")
    parser.add_argument("--timeout", type=int, default=600,
                        help="Timeout per stage in seconds")
    parser.add_argument("--list", action="store_true",
                        help="List available stages and exit")
    parser.add_argument("--status", action="store_true",
                        help="Show pipeline status")
    parser.add_argument("--notify", action="store_true",
                        help="Send notifications (Slack/Email) on pipeline result")

    args = parser.parse_args()

    if args.list:
        print("Available stages:")
        for s in STAGES_ORDER:
            print(f"  {s}: {STAGE_SCRIPTS[s]}")
        return

    if args.status:
        state = load_json(PIPELINE_STATE)
        log_data = load_json(PIPELINE_LOG)
        if not state and not log_data:
            print("No pipeline runs yet.")
            return
        if state:
            print(f"Total runs: {state.get('total_runs', 0)}")
            for stage, info in state.get("stages", {}).items():
                status = info.get("status", "?")
                last = info.get("last_run", "?")[:19]
                print(f"  {stage}: {status} (last: {last})")
        if log_data:
            last_run = log_data[-1] if isinstance(log_data, list) else log_data
            print(f"Last run: {last_run.get('run_number', '?')}")
            print(f"  Skills: {last_run.get('skills_count', '?')}")
            print(f"  Evolved: {last_run.get('evolved_count', '?')}")
            print(f"  Result: {'PASSED' if last_run.get('all_passed') else 'FAILED'}")
        return

    stages = STAGES_ORDER if "all" in args.stages else args.stages
    success = run_pipeline(stages, timeout_per_stage=args.timeout, notify=args.notify)
    sys.exit(0 if success else 1)


if __name__ == "__main__":
    main()
