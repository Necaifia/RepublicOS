"""
scheduler.py — Auto-Scheduler for Skill Evolution Pipeline
Orchestrates daily discovery + evolution runs with lock safety,
logging, and status reporting.
"""

import argparse
import json
import os
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path

SCRIPT_DIR = Path(__file__).parent.resolve()
REGISTRY_DIR = SCRIPT_DIR / "registry"
SCHEDULER_DIR = REGISTRY_DIR / "scheduler"
LOCK_FILE = SCHEDULER_DIR / "scheduler.lock"
STATE_FILE = SCHEDULER_DIR / "scheduler-state.json"
LOG_DIR = SCHEDULER_DIR / "logs"


def log(msg: str, level: str = "INFO"):
    ts = datetime.now(timezone.utc).strftime("%Y-%m-%d %H:%M:%S")
    print(f"[{ts}] [SCHEDULER] [{level}] {msg}")


def load_json(path: Path):
    if path.exists():
        with open(path, encoding="utf-8") as f:
            return json.load(f)
    return {}


def save_json(path: Path, data):
    path.parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        json.dump(data, f, indent=2, ensure_ascii=False)


def acquire_lock() -> bool:
    SCHEDULER_DIR.mkdir(parents=True, exist_ok=True)
    if LOCK_FILE.exists():
        lock_data = load_json(LOCK_FILE)
        pid = lock_data.get("pid")
        if pid:
            try:
                os.kill(pid, 0)
                log(f"Lock held by PID {pid} since {lock_data.get('since', '?')}", "WARN")
                return False
            except OSError:
                log("Stale lock file found, removing", "WARN")
                LOCK_FILE.unlink(missing_ok=True)
    save_json(LOCK_FILE, {"pid": os.getpid(), "since": datetime.now(timezone.utc).isoformat()})
    return True


def release_lock():
    if LOCK_FILE.exists():
        try:
            LOCK_FILE.unlink(missing_ok=True)
        except Exception:
            pass


def run_pipeline(stages=None, timeout=600) -> bool:
    pipeline_script = SCRIPT_DIR / "pipeline.py"
    cmd = [sys.executable, str(pipeline_script)]
    if stages:
        cmd.extend(["--stages"] + stages)
    if timeout:
        cmd.extend(["--timeout", str(timeout)])
    log(f"Running: {' '.join(cmd)}")
    result = subprocess.run(cmd, capture_output=False, timeout=timeout + 60)
    return result.returncode == 0


def read_pipeline_log() -> dict:
    log_data = load_json(REGISTRY_DIR / "pipeline-log.json")
    if isinstance(log_data, list) and log_data:
        return log_data[-1]
    return {}


def run_scheduled(full: bool = False, notify: bool = True, stages=None, timeout: int = 600):
    if not acquire_lock():
        return False

    try:
        now = datetime.now(timezone.utc)
        run_id = now.strftime("%Y%m%d-%H%M%S")

        LOG_DIR.mkdir(parents=True, exist_ok=True)
        log_file = LOG_DIR / f"run-{run_id}.log"
        log_handler = open(log_file, "w", encoding="utf-8")
        log_handler.write(f"Scheduled run {run_id} started at {now.isoformat()}\n")
        log_handler.flush()

        log(f"Starting scheduled run {run_id}")
        log(f"Pipeline stages: {stages or 'all'}")
        log(f"Full discovery: {full}")
        log(f"Log file: {log_file}")

        start = time.time()
        if full:
            RERUN_FILE = SCRIPT_DIR / ".full-rerun"
            RERUN_FILE.touch()
        success = run_pipeline(stages=stages, timeout=timeout)
        if full:
            RERUN_FILE.unlink(missing_ok=True)
        elapsed = time.time() - start

        pipeline_result = read_pipeline_log()

        state = load_json(STATE_FILE)
        run_record = {
            "run_id": run_id,
            "timestamp": now.isoformat(),
            "elapsed_seconds": round(elapsed, 1),
            "success": success,
            "stages": stages or "all",
            "full_discovery": full,
            "skills_count": pipeline_result.get("skills_count", 0),
            "evolved_count": pipeline_result.get("evolved_count", 0),
            "log_file": str(log_file),
        }

        if not state:
            state = {"total_runs": 0, "last_run": None, "runs": []}
        state["total_runs"] += 1
        state["last_run"] = run_record
        state["runs"].append(run_record)
        state["runs"] = state["runs"][-100:]
        save_json(STATE_FILE, state)

        log_handler.write(f"Run {run_id} completed: {'SUCCESS' if success else 'FAILED'} ({elapsed:.1f}s)\n")
        log_handler.close()

        if success:
            log(f"Run {run_id} completed successfully in {elapsed:.1f}s")
        else:
            log(f"Run {run_id} FAILED after {elapsed:.1f}s", "ERROR")

        return success

    finally:
        release_lock()


def show_status():
    state = load_json(STATE_FILE)
    if not state:
        print("No scheduled runs yet.")
        return
    last = state.get("last_run", {})
    print(f"Total scheduled runs: {state.get('total_runs', 0)}")
    print(f"Last run: {last.get('run_id', 'N/A')}")
    print(f"  Timestamp:    {last.get('timestamp', '?')[:19]}")
    print(f"  Duration:     {last.get('elapsed_seconds', '?')}s")
    print(f"  Success:      {'Yes' if last.get('success') else 'No'}")
    print(f"  Skills:       {last.get('skills_count', '?')}")
    print(f"  Evolved:      {last.get('evolved_count', '?')}")
    print(f"  Full discover: {last.get('full_discovery', False)}")
    print(f"  Log:          {last.get('log_file', 'N/A')}")
    print()
    recent = state.get("runs", [])[-5:]
    if recent:
        print("Recent runs:")
        for r in reversed(recent):
            icon = "+" if r.get("success") else "x"
            ts = r.get("timestamp", "?")[:16]
            print(f"  [{icon}] {r.get('run_id', '?')}  {ts}  "
                  f"{r.get('elapsed_seconds', '?')}s  "
                  f"{r.get('skills_count', '?')} skills  "
                  f"{'FULL' if r.get('full_discovery') else 'incremental'}")


def main():
    parser = argparse.ArgumentParser(description="RepublicOS Skill Engine Scheduler")
    parser.add_argument("action", nargs="?", default="run",
                        choices=["run", "status", "list"],
                        help="Action to perform (default: run)")
    parser.add_argument("--stages", nargs="+", help="Stages to run (default: all)")
    parser.add_argument("--timeout", type=int, default=600,
                        help="Timeout per stage in seconds (default: 600)")
    parser.add_argument("--full", action="store_true",
                        help="Force full rediscovery (ignore cache)")
    parser.add_argument("--no-notify", action="store_true",
                        help="Disable notifications")

    args = parser.parse_args()

    if args.action == "status":
        show_status()
        return

    if args.action == "list":
        print("Scheduled runs:")
        state = load_json(STATE_FILE)
        for r in state.get("runs", [])[-20:]:
            icon = "+" if r.get("success") else "x"
            print(f"  [{icon}] {r.get('run_id', '?'):20s} "
                  f"{r.get('timestamp', '?')[:19]}  "
                  f"{r.get('elapsed_seconds', '?'):5.0f}s  "
                  f"{r.get('skills_count', '?'):3d} skills")
        return

    success = run_scheduled(
        full=args.full,
        notify=not args.no_notify,
        stages=args.stages,
        timeout=args.timeout,
    )
    sys.exit(0 if success else 1)


if __name__ == "__main__":
    main()
