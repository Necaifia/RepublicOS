#!/usr/bin/env python3
"""Governor Core — single source of truth for all enforcement decisions.

Usage:
  python governor.py check --action <name> [--target <path>] [--diff-lines <n>] [--new-files <n>]

Exit codes:
  0 = allow
  1 = deny (hard block)
  2 = pending (needs human approval)
  3 = budget exhausted
  4 = anomaly detected
"""

import json
import os
import sys
import yaml
from pathlib import Path
from datetime import datetime, timezone

ROOT = Path(__file__).resolve().parent.parent
GOVERNOR_PATH = ROOT / ".governor" / "GOVERNOR.yaml"
PENDING_PATH = ROOT / ".governor" / "pending_approval.json"
HUMAN_VETO_PATH = ROOT / ".governor" / "human_veto"
SANDBOX_PREFIX = ROOT / ".sandbox"


def load_config():
    if not GOVERNOR_PATH.exists():
        print(f"GOVERNOR.yaml not found at {GOVERNOR_PATH}", file=sys.stderr)
        sys.exit(1)
    with open(GOVERNOR_PATH) as f:
        return yaml.safe_load(f)


def check_filesystem_guard():
    """Hard enforcement: if governor config is writable, warn (but don't block)."""
    governor_path = GOVERNOR_PATH
    if os.access(governor_path, os.W_OK):
        print("Warning: GOVERNOR.yaml is writable by this process.", file=sys.stderr)

    if HUMAN_VETO_PATH.exists():
        reason = HUMAN_VETO_PATH.read_text().strip()
        print(f"Human veto active: {reason}", file=sys.stderr)
        sys.exit(4)


def resolve_action(cfg, action_name):
    actions = cfg.get("actions", {})
    if action_name in actions:
        return actions[action_name]
    default = cfg.get("default_policy", {})
    unknown_level = default.get("unknown_action_level", 2)
    return {"level": unknown_level, "scope": "project", "rationale": "Unknown action — default policy applied"}


def check_budget(cfg):
    budget = cfg.get("budget", {})
    pending = PENDING_PATH
    if pending.exists():
        try:
            data = json.loads(pending.read_text())
            requests = data.get("requests", [])
            session_cycles = sum(1 for r in requests if r.get("status") == "approved")
            max_cycles = budget.get("max_cycles_per_session", 10)
            if session_cycles >= max_cycles:
                print(f"Budget exhausted: {session_cycles} cycles used, max {max_cycles}", file=sys.stderr)
                sys.exit(3)
        except (json.JSONDecodeError, KeyError):
            pass


def check_anomaly(cfg, diff_lines, new_files):
    anomaly = cfg.get("anomaly", {})
    max_diff = anomaly.get("max_diff_lines", 500)
    max_new = anomaly.get("max_new_files_per_cycle", 15)

    if diff_lines is not None and diff_lines > max_diff:
        print(f"Anomaly: {diff_lines} diff lines exceeds max {max_diff}", file=sys.stderr)
        sys.exit(4)

    if new_files is not None and new_files > max_new:
        print(f"Anomaly: {new_files} new files exceeds max {max_new}", file=sys.stderr)
        sys.exit(4)


def check_level(level, action_cfg, action_name, target, scope_info):
    if level == 0:
        print(f"ALLOW: {action_name} (level 0)")
        sys.exit(0)

    elif level == 1:
        scope = action_cfg.get("scope", "")
        if scope == "cycle_directory_only":
            if target:
                target_path = Path(target).resolve()
                if not str(target_path).startswith(str(SANDBOX_PREFIX.resolve())):
                    print(f"DENY: {action_name} target '{target}' is outside sandbox", file=sys.stderr)
                    sys.exit(1)
        print(f"ALLOW: {action_name} (level 1)")
        sys.exit(0)

    elif level == 2:
        rationale = action_cfg.get("rationale", "")
        request = {
            "action": action_name,
            "target": target or "",
            "rationale": rationale,
            "requested_at": datetime.now(timezone.utc).isoformat().replace("+00:00", "Z"),
            "status": "pending",
            "response_date": None,
        }
        pending = PENDING_PATH
        if pending.exists():
            data = json.loads(pending.read_text())
        else:
            data = {"requests": [], "instructions": "To approve or deny, set 'status' to 'approved' or 'denied'."}
        data["requests"].append(request)
        pending.write_text(json.dumps(data, indent=2))
        print(f"PENDING: {action_name} — request written to {PENDING_PATH}", file=sys.stderr)
        print(f"  Rationale: {rationale}", file=sys.stderr)
        sys.exit(2)

    elif level == 3:
        rationale = action_cfg.get("rationale", "Level 3 action — never allowed")
        print(f"DENY: {action_name} (level 3)", file=sys.stderr)
        print(f"  Rationale: {rationale}", file=sys.stderr)
        sys.exit(1)

    else:
        print(f"DENY: {action_name} has unknown level {level}", file=sys.stderr)
        sys.exit(1)


def cmd_check(args):
    cfg = load_config()
    check_filesystem_guard()
    check_budget(cfg)
    check_anomaly(cfg, args.diff_lines, args.new_files)

    action_name = args.action
    target = args.target
    action_cfg = resolve_action(cfg, action_name)
    level = action_cfg.get("level", 2)
    check_level(level, action_cfg, action_name, target, cfg)


def cmd_pending(args):
    if not PENDING_PATH.exists():
        print("No pending approvals.")
        sys.exit(0)
    data = json.loads(PENDING_PATH.read_text())
    pending = [r for r in data.get("requests", []) if r.get("status") == "pending"]
    if not pending:
        print("No pending approvals.")
        sys.exit(0)
    for r in pending:
        print(f"[pending] {r['action']} target='{r['target']}' at {r['requested_at']}")
    sys.exit(0 if args.quiet else len(pending))


def cmd_status(args):
    cfg = load_config()
    check_filesystem_guard()
    print("Governor Status")
    print(f"  Config: {'OK' if GOVERNOR_PATH.exists() else 'MISSING'}")

    pending = PENDING_PATH
    if pending.exists():
        try:
            data = json.loads(pending.read_text())
            pending_count = sum(1 for r in data.get("requests", []) if r.get("status") == "pending")
            print(f"  Pending approvals: {pending_count}")
        except json.JSONDecodeError:
            print("  Pending approvals: (corrupt file)")
    else:
        print("  Pending approvals: 0")

    veto = HUMAN_VETO_PATH
    if veto.exists():
        print(f"  Human veto: ACTIVE — {veto.read_text().strip()}")

    budget = cfg.get("budget", {})
    print(f"  Max cycles/session: {budget.get('max_cycles_per_session', '?')}")
    print(f"  Max diff lines: {budget.get('max_diff_lines', 500)}")


if __name__ == "__main__":
    import argparse

    parser = argparse.ArgumentParser(prog="governor")
    sub = parser.add_subparsers(dest="command", required=True)

    check_p = sub.add_parser("check")
    check_p.add_argument("--action", required=True)
    check_p.add_argument("--target", default=None)
    check_p.add_argument("--diff-lines", type=int, default=None)
    check_p.add_argument("--new-files", type=int, default=None)

    pending_p = sub.add_parser("pending")
    pending_p.add_argument("--quiet", action="store_true")

    sub.add_parser("status")

    args = parser.parse_args()
    if args.command == "check":
        cmd_check(args)
    elif args.command == "pending":
        cmd_pending(args)
    elif args.command == "status":
        cmd_status(args)
