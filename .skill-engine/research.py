"""
research.py — Web Research Engine (Day 3)
Part of RepublicOS Skill Evolution Pipeline

Researches best practices per domain, fetches references,
and builds a knowledge base to inform skill evolution.
Can pull from web search results (integrated) and known URLs.
"""

import json
import re
from collections import defaultdict
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional

import requests

SCRIPT_DIR = Path(__file__).parent.resolve()
REGISTRY_DIR = SCRIPT_DIR / "registry"
RESEARCH_DIR = REGISTRY_DIR / "research"


# Known high-quality references per domain (official docs, standards)
DOMAIN_REFERENCES = {
    "deploy": [
        {"url": "https://docs.github.com/en/actions/deployment", "label": "GitHub Deploy Docs"},
        {"url": "https://docs.docker.com/develop/dev-best-practices/", "label": "Docker Dev Best Practices"},
    ],
    "test": [
        {"url": "https://docs.pytest.org/en/stable/", "label": "pytest Docs"},
        {"url": "https://testing.googleblog.com/", "label": "Google Testing Blog"},
    ],
    "security": [
        {"url": "https://owasp.org/www-project-top-ten/", "label": "OWASP Top 10"},
        {"url": "https://cheatsheetseries.owasp.org/", "label": "OWASP Cheat Sheets"},
    ],
    "database": [
        {"url": "https://www.postgresql.org/docs/current/", "label": "PostgreSQL Docs"},
        {"url": "https://www.mongodb.com/docs/", "label": "MongoDB Docs"},
    ],
    "api": [
        {"url": "https://learn.microsoft.com/en-us/azure/architecture/best-practices/api-design", "label": "Azure API Design"},
        {"url": "https://swagger.io/resources/open-api/", "label": "OpenAPI Spec"},
    ],
    "infrastructure": [
        {"url": "https://docs.docker.com/develop/dev-best-practices/", "label": "Docker Best Practices"},
        {"url": "https://kubernetes.io/docs/concepts/configuration/overview/", "label": "K8s Config"},
    ],
    "monitoring": [
        {"url": "https://prometheus.io/docs/practices/naming/", "label": "Prometheus Best Practices"},
        {"url": "https://opentelemetry.io/docs/reference/specification/", "label": "OpenTelemetry Spec"},
    ],
    "code_review": [
        {"url": "https://google.github.io/eng-practices/review/", "label": "Google Code Review"},
        {"url": "https://smartbear.com/learn/code-review/best-practices/", "label": "SmartBear Code Review"},
    ],
    "documentation": [
        {"url": "https://www.writethedocs.org/guide/", "label": "Write the Docs"},
        {"url": "https://documentation.divio.com/", "label": "Divio Documentation System"},
    ],
    "communication": [
        {"url": "https://slack.com/help/articles/360059655654-Slack-best-practices", "label": "Slack Best Practices"},
    ],
}


def log(msg: str, level: str = "INFO"):
    ts = datetime.now(timezone.utc).strftime("%H:%M:%S")
    print(f"[{ts}] [{level}] [RESEARCH] {msg}")


def load_json(path: Path) -> Any:
    if path.exists():
        with open(path, encoding="utf-8") as f:
            return json.load(f)
    return {}


def save_json(path: Path, data: Any):
    path.parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        json.dump(data, f, indent=2, ensure_ascii=False)


def fetch_url(url: str, timeout: int = 15) -> Optional[str]:
    try:
        resp = requests.get(url, timeout=timeout, headers={
            "User-Agent": "RepublicOS-Research/1.0",
        })
        resp.raise_for_status()
        return resp.text
    except requests.exceptions.HTTPError as e:
        code = e.response.status_code if hasattr(e, "response") else 0
        log(f"  HTTP {code}: {url[:60]}", "WARN")
        return None
    except Exception as e:
        log(f"  Fetch failed: {url[:60]} - {type(e).__name__}", "WARN")
        return None


def extract_title(html: str) -> str:
    m = re.search(r"<title>(.*?)</title>", html, re.IGNORECASE | re.DOTALL)
    return m.group(1).strip() if m else "Untitled"


def research_domain(domain: str, references: List[dict]) -> dict:
    findings = {
        "domain": domain,
        "references_checked": [],
        "key_practices": [],
        "common_tools": [],
        "common_patterns": [],
        "sources": [],
    }

    for ref in references:
        html = fetch_url(ref["url"])
        if html:
            title = extract_title(html)
            findings["references_checked"].append({
                "url": ref["url"],
                "label": ref["label"],
                "title": title,
                "status": "ok",
            })
            findings["sources"].append(ref["url"])
        else:
            findings["references_checked"].append({
                "url": ref["url"],
                "label": ref["label"],
                "status": "unreachable",
            })

    return findings


def generate_domain_report(domain: str, findings: dict, gaps: dict):
    report_path = RESEARCH_DIR / f"{domain}.md"
    report_path.parent.mkdir(parents=True, exist_ok=True)

    gap_info = ""
    for g in gaps.get("underdeveloped", []):
        if g["domain"] == domain:
            gap_info = f"\n**Deficit:** {g['current']}/{g['expected']} skills (need {g['deficit']} more)\n"

    lines = [
        f"# Research: {domain}",
        "",
        f"*Generated: {datetime.now(timezone.utc).isoformat()}*",
        gap_info,
        "## References Checked",
        "",
    ]

    for ref in findings.get("references_checked", []):
        status = "✅" if ref.get("status") == "ok" else "❌"
        label = ref.get("label", ref.get("url", ""))
        lines.append(f"- {status} [{label}]({ref.get('url', '')})")
        if ref.get("title"):
            lines[-1] += f" — _{ref['title']}_"

    lines.extend([
        "",
        "## Key Practices",
        "",
    ])
    for p in findings.get("key_practices", ["(pending AI research)"]):
        lines.append(f"- {p}")

    lines.extend([
        "",
        "## Common Tools",
        "",
    ])
    for t in findings.get("common_tools", ["(pending AI research)"]):
        lines.append(f"- `{t}`")

    lines.extend([
        "",
        "## Common Patterns",
        "",
    ])
    for p in findings.get("common_patterns", ["(pending AI research)"]):
        lines.append(f"- {p}")

    lines.extend([
        "",
        "## Sources",
        "",
    ])
    for s in findings.get("sources", []):
        lines.append(f"- {s}")

    report_path.write_text("\n".join(lines) + "\n", encoding="utf-8")
    return report_path


def run():
    log("=" * 60)
    log("Research Engine - Day 3")
    log("=" * 60)

    clusters = load_json(REGISTRY_DIR / "clusters.json")
    gaps = load_json(REGISTRY_DIR / "gaps.json")

    all_domains = list(DOMAIN_REFERENCES.keys())
    log(f"Domains to research: {len(all_domains)}")

    all_findings = {}
    for domain in all_domains:
        log(f"  Researching: {domain}")
        refs = DOMAIN_REFERENCES.get(domain, [])
        findings = research_domain(domain, refs)
        all_findings[domain] = findings

        report_path = generate_domain_report(domain, findings, gaps.get("gaps", {}))
        ok_count = sum(1 for r in findings["references_checked"] if r["status"] == "ok")
        log(f"    References: {ok_count}/{len(refs)} reachable")

    # Summary
    save_json(RESEARCH_DIR / "findings.json", {
        "research_date": datetime.now(timezone.utc).isoformat(),
        "domains_researched": len(all_domains),
        "findings": all_findings,
    })

    log(f"\n{'=' * 60}")
    log("Research complete!")
    log(f"  Domains researched: {len(all_domains)}")
    log(f"  Reports: {RESEARCH_DIR}/<domain>.md")
    log(f"  Findings: {RESEARCH_DIR}/findings.json")
    log(f"{'=' * 60}")


if __name__ == "__main__":
    run()
