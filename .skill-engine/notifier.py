"""
notifier.py — Slack and Email notifications for pipeline events
Integrates with pipeline.py via post-stage hooks.
"""

import json
import os
import smtplib
import ssl
from datetime import datetime, timezone
from email.message import EmailMessage
from pathlib import Path
from typing import Optional

SCRIPT_DIR = Path(__file__).parent.resolve()
CONFIG_FILE = SCRIPT_DIR / "notifier-config.json"


def load_config() -> dict:
    defaults = {
        "slack_webhook_url": "",
        "slack_enabled": False,
        "email_smtp_host": "",
        "email_smtp_port": 587,
        "email_username": "",
        "email_password": "",
        "email_from": "",
        "email_to": "",
        "email_use_tls": True,
        "email_enabled": False,
        "notify_on": ["failure"],
        "notify_on_success": False,
    }
    if CONFIG_FILE.exists():
        with open(CONFIG_FILE, encoding="utf-8") as f:
            defaults.update(json.load(f))
    return defaults


def save_config(cfg: dict):
    CONFIG_FILE.parent.mkdir(parents=True, exist_ok=True)
    with open(CONFIG_FILE, "w", encoding="utf-8") as f:
        json.dump(cfg, f, indent=2, ensure_ascii=False)
    print(f"Config saved: {CONFIG_FILE}")


def send_slack(webhook_url: str, message: str) -> bool:
    try:
        import urllib.request
        payload = json.dumps({"text": message}).encode("utf-8")
        req = urllib.request.Request(webhook_url, data=payload,
                                     headers={"Content-Type": "application/json"})
        urllib.request.urlopen(req, timeout=10)
        return True
    except Exception as e:
        print(f"Slack notification failed: {e}")
        return False


def send_email(cfg: dict, subject: str, body: str) -> bool:
    try:
        msg = EmailMessage()
        msg["Subject"] = subject
        msg["From"] = cfg["email_from"]
        msg["To"] = cfg["email_to"]
        msg.set_content(body)

        context = ssl.create_default_context() if cfg.get("email_use_tls", True) else None
        with smtplib.SMTP(cfg["email_smtp_host"], cfg["email_smtp_port"], timeout=30) as server:
            if cfg.get("email_use_tls", True):
                server.starttls(context=context)
            if cfg.get("email_username") and cfg.get("email_password"):
                server.login(cfg["email_username"], cfg["email_password"])
            server.send_message(msg)
        return True
    except Exception as e:
        print(f"Email notification failed: {e}")
        return False


def notify(event: str, pipeline_result: dict):
    cfg = load_config()
    event_types = cfg.get("notify_on", ["failure"])
    should_notify = event in event_types
    if event == "success" and not cfg.get("notify_on_success", False):
        should_notify = False
    if not should_notify:
        return

    timestamp = datetime.now(timezone.utc).strftime("%Y-%m-%d %H:%M UTC")
    skills = pipeline_result.get("skills_count", "?")
    evolved = pipeline_result.get("evolved_count", "?")
    stages = pipeline_result.get("stages", {})
    passed_stages = [k for k, v in stages.items() if v == "passed"]
    failed_stages = [k for k, v in stages.items() if v == "failed"]

    summary = (f"RepublicOS Skill Engine - Pipeline {event.upper()}\n"
               f"Time: {timestamp}\n"
               f"Skills: {skills} | Evolved: {evolved}\n"
               f"Passed: {', '.join(passed_stages) or 'none'}\n")
    if failed_stages:
        summary += f"Failed: {', '.join(failed_stages)}\n"

    slack_msg = f"*RepublicOS Skill Engine*\n*Pipeline: {event.upper()}*\n{summary}"
    slack_msg += f"\n<{str(SCRIPT_DIR.parent)}|Open Repository>"

    email_subject = f"RepublicOS Pipeline: {event.upper()} - {timestamp[:13]}"
    email_body = f"RepublicOS Skill Engine - Pipeline Report\n\n{summary}"

    results = []
    if cfg.get("slack_enabled") and cfg.get("slack_webhook_url"):
        ok = send_slack(cfg["slack_webhook_url"], slack_msg)
        results.append(f"Slack: {'OK' if ok else 'FAILED'}")
    if cfg.get("email_enabled") and cfg.get("email_smtp_host"):
        ok = send_email(cfg, email_subject, email_body)
        results.append(f"Email: {'OK' if ok else 'FAILED'}")

    if results:
        print(f"Notification ({event}): {', '.join(results)}")
    else:
        print(f"No notifiers configured for event '{event}'. Run `notifier.py --setup` to configure.")


def setup():
    print("=== RepublicOS Notifier Setup ===\n")
    cfg = load_config()

    print("Slack Webhook (optional)")
    current = cfg.get("slack_webhook_url", "")
    val = input(f"  Webhook URL [{current}]: ").strip()
    if val:
        cfg["slack_webhook_url"] = val
        cfg["slack_enabled"] = True
    elif current:
        cfg["slack_enabled"] = bool(input("  Enable Slack? (y/n): ").strip().lower() == "y")

    print("\nEmail (optional)")
    for key, label in [("email_smtp_host", "SMTP host"), ("email_smtp_port", "SMTP port"),
                       ("email_username", "Username"), ("email_password", "Password (won't display)"),
                       ("email_from", "From address"), ("email_to", "To address")]:
        current = cfg.get(key, "")
        prompt = f"  {label}"
        if key != "email_password":
            prompt += f" [{current}]"
        val = input(f"{prompt}: ").strip()
        if val:
            cfg[key] = val

    if cfg.get("email_smtp_host"):
        cfg["email_enabled"] = True
        val = input("  Use TLS? (Y/n): ").strip().lower()
        if val == "n":
            cfg["email_use_tls"] = False

    print("\nNotify on events")
    print("  1) Failure only (default)")
    print("  2) Success and failure")
    val = input("  Choice [1]: ").strip()
    if val == "2":
        cfg["notify_on_success"] = True
    cfg["notify_on"] = ["failure", "success"] if cfg["notify_on_success"] else ["failure"]

    save_config(cfg)
    print("\nNotifier configured!")
    print(f"  Slack: {'enabled' if cfg['slack_enabled'] else 'disabled'}")
    print(f"  Email: {'enabled' if cfg['email_enabled'] else 'disabled'}")
    print(f"  Events: {', '.join(cfg.get('notify_on', ['failure']))}")


def main():
    import argparse
    parser = argparse.ArgumentParser(description="RepublicOS Notifier")
    sub = parser.add_subparsers(dest="command")

    p_setup = sub.add_parser("setup", help="Interactive configuration")
    p_test = sub.add_parser("test", help="Send test notification")
    p_test.add_argument("--type", choices=["slack", "email"], default="slack",
                        help="Notification type to test")

    args = parser.parse_args()
    if args.command == "setup":
        setup()
    elif args.command == "test":
        cfg = load_config()
        test_result = {"skills_count": 88, "evolved_count": 88,
                       "stages": {"discovery": "passed", "analyzer": "passed",
                                  "research": "passed", "evolution": "passed", "store": "passed"}}
        notify("test", test_result)
    else:
        parser.print_help()


if __name__ == "__main__":
    main()
