"""
evolution.py — Skill Evolution MVP (Day 4)
Part of RepublicOS Skill Evolution Pipeline

Takes discovered skills + research findings and evolves them into
improved versions with better structure, descriptions, and best practices.
"""

import json
import os
import re
import shutil
from collections import defaultdict
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional

SCRIPT_DIR = Path(__file__).parent.resolve()
REGISTRY_DIR = SCRIPT_DIR / "registry"
RESEARCH_DIR = REGISTRY_DIR / "research"
EVOLVED_DIR = REGISTRY_DIR / "evolved"
RAW_DIR = REGISTRY_DIR / "raw"


# Enhancement templates per section type
SECTION_ENHANCEMENTS = {
    "Prerequisites": """
## Prerequisites

- Ensure all required tools and dependencies are installed
- Verify you have the necessary permissions and access credentials
- Check that the target environment is in a known good state

""",

    "Error Handling": """
## Error Handling

- Always check the exit code or response status of commands before proceeding
- On failure, log the error details and attempt recovery if a retry strategy exists
- If recovery fails, report the error with context: what was attempted, what went wrong, and suggested next steps
- Never silently ignore errors — treat unexpected output as potential failure

""",

    "Verification": """
## Verification

- After each step, verify the expected outcome before continuing
- Use idempotent checks: running the same action twice produces the same result
- If verification fails, roll back the last change and report the issue
- Log verification results for audit trail

""",

    "Security": """
## Security

- Never hardcode secrets, tokens, or credentials in skill files or scripts
- Use environment variables or secret management tools for sensitive values
- Validate all user inputs before processing
- Follow least-privilege principle: request only the permissions you need
- Log all security-relevant actions for audit

""",

    "Rollback": """
## Rollback

- Every action should have a defined undo procedure
- If a step fails after partial completion, reverse all changes made in this session
- Verify the system is in a known good state after rollback
- Document what was rolled back and why

""",

    "Configuration": """
## Configuration

- Use environment variables with sensible defaults for configuration
- Validate configuration at the start of execution
- Document all configuration options and their effects
- Support loading config from files when appropriate

""",
}

# Domain-specific practice improvements
DOMAIN_IMPROVEMENTS = {
    "communication": {
        "sections": ["Prerequisites", "Error Handling", "Verification", "Security", "Rollback"],
        "description_boost": "Handles multi-channel routing with severity-based dispatch, deduplication, rate limiting, and escalation chains.",
    },
    "monitoring": {
        "sections": ["Prerequisites", "Error Handling", "Verification", "Security", "Configuration"],
        "description_boost": "Implements trace-level observability with OpenTelemetry GenAI conventions, eval-driven monitoring, and cost attribution.",
    },
    "deploy": {
        "sections": ["Prerequisites", "Error Handling", "Verification", "Rollback", "Security"],
        "description_boost": "Follows agent-native CI/CD with eval-gated promotion, canary rollouts, and compensating rollback actions.",
    },
    "test": {
        "sections": ["Prerequisites", "Error Handling", "Verification", "Configuration"],
        "description_boost": "Uses AI-native test strategy with self-healing, intent-based authoring, and PR-time verification gates.",
    },
    "security": {
        "sections": ["Prerequisites", "Error Handling", "Verification", "Configuration"],
        "description_boost": "Enforces least-privilege, secret management, input validation, and audit logging throughout.",
    },
    "database": {
        "sections": ["Prerequisites", "Error Handling", "Verification", "Rollback", "Configuration"],
        "description_boost": "Uses transactional safety with compensating actions, connection pooling, and query validation.",
    },
    "api": {
        "sections": ["Prerequisites", "Error Handling", "Verification", "Security", "Configuration"],
        "description_boost": "Follows API design best practices with schema validation, error standardization, and idempotency support.",
    },
    "infrastructure": {
        "sections": ["Prerequisites", "Error Handling", "Verification", "Rollback", "Security"],
        "description_boost": "Treats infrastructure as code with immutable deployments, health checks, and circuit breakers.",
    },
    "code_review": {
        "sections": ["Prerequisites", "Security", "Configuration"],
        "description_boost": "Applies structured code review with automated linting, security scanning, and quality gates.",
    },
    "documentation": {
        "sections": ["Prerequisites", "Configuration"],
        "description_boost": "Follows documentation-as-code with structured templates, audience adaptation, and automated freshness checks.",
    },
}

DEFAULT_IMPROVEMENT = {
    "sections": ["Prerequisites", "Error Handling", "Verification", "Configuration"],
    "description_boost": "Follows current best practices for reliability, security, and maintainability.",
}


def log(msg: str):
    ts = datetime.now(timezone.utc).strftime("%H:%M:%S")
    print(f"[{ts}] [EVOLVE] {msg}")


def load_json(path: Path) -> Any:
    if path.exists():
        with open(path, encoding="utf-8") as f:
            return json.load(f)
    return {}


def save_json(path: Path, data: Any):
    path.parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        json.dump(data, f, indent=2, ensure_ascii=False)


def read_raw_skill(skill: dict) -> str:
    local_path = skill.get("local_path", "")
    if local_path and os.path.exists(local_path):
        try:
            with open(local_path, "r", encoding="utf-8") as f:
                return f.read()
        except Exception:
            pass
    skill_id = skill.get("id", "unknown")
    safe = re.sub(r'[<>:"/\\|?*]', "_", skill_id)
    raw_path = RAW_DIR / f"{safe}.md"
    if raw_path.exists():
        return raw_path.read_text(encoding="utf-8")
    return ""


def get_domain_priority(domains: List[str]) -> str:
    """Return the highest-priority domain for enhancement ordering."""
    priority_order = [
        "communication", "monitoring", "test", "deploy", "security",
        "infrastructure", "database", "api", "code_review", "documentation",
    ]
    for p in priority_order:
        if p in domains:
            return p
    return domains[0] if domains else "uncategorized"


def enhance_description(old_desc: str, domains: List[str]) -> str:
    """Enhance a skill description using domain research."""
    base = old_desc.strip()
    priority_domain = get_domain_priority(domains)
    imp = DOMAIN_IMPROVEMENTS.get(priority_domain, DEFAULT_IMPROVEMENT)
    boost = imp["description_boost"]

    if len(base) < 50:
        base = base + " " + boost if base else boost
    elif boost not in base:
        base = base.rstrip(".") + ". " + boost

    return base[:1024]


def enhance_frontmatter(fm: Optional[dict], domains: List[str]) -> dict:
    """Add or improve frontmatter fields."""
    if not fm:
        fm = {"name": "unknown", "description": "(evolved)"}

    fm["description"] = enhance_description(
        fm.get("description", ""), domains
    )

    if "version" not in fm:
        fm["version"] = 2
    if "tags" not in fm:
        fm["tags"] = list(set(domains) - {"uncategorized"})
    if "metadata" not in fm:
        fm["metadata"] = {"evolved": True, "evolved_at": None}
    if isinstance(fm.get("metadata"), dict):
        fm["metadata"]["evolved"] = True
        fm["metadata"]["evolved_at"] = datetime.now(timezone.utc).isoformat()

    return fm


def enhance_body(body: str, existing_sections: List[str], domains: List[str]) -> str:
    """Inject missing enhancement sections into the skill body."""
    priority_domain = get_domain_priority(domains)
    imp = DOMAIN_IMPROVEMENTS.get(priority_domain, DEFAULT_IMPROVEMENT)
    needed = imp["sections"]

    body_lines = body.splitlines()
    result_lines = []
    inserted_sections = []

    # Find existing section headers
    existing_headers = set()
    for line in body_lines:
        m = re.match(r"^##\s+(.+)", line)
        if m:
            existing_headers.add(m.group(1).strip())

    for section_name in needed:
        if section_name not in existing_headers:
            enhancement = SECTION_ENHANCEMENTS.get(section_name, "")
            if enhancement:
                inserted_sections.append(enhancement)

    # Insert new sections before the last section or at end
    if inserted_sections:
        # Find a good insertion point (before References / Resources / Appendix)
        insertion_idx = len(body_lines)
        for marker in ["## References", "## Resources", "## Appendix", "## See Also"]:
            for i, line in enumerate(body_lines):
                if line.startswith(marker):
                    insertion_idx = min(insertion_idx, i)

        for i, line in enumerate(body_lines):
            if i == insertion_idx:
                result_lines.extend(["\n" + s.strip() + "\n" for s in inserted_sections])
            result_lines.append(line)

        if insertion_idx == len(body_lines):
            result_lines.extend(["\n" + s.strip() + "\n" for s in inserted_sections])
    else:
        result_lines = body_lines

    return "\n".join(result_lines)


def evolve_skill(skill: dict, content: str) -> tuple:
    """Evolve a single skill. Returns (evolved_content, evolution_record)."""
    domains = skill.get("domains", ["uncategorized"])
    old_name = skill.get("name", "unknown")

    # Parse frontmatter
    fm_match = re.match(r"^---\s*\n(.*?)\n---", content, re.DOTALL)
    fm = None
    if fm_match:
        try:
            import yaml
            fm = yaml.safe_load(fm_match.group(1))
        except Exception:
            pass

    old_desc = fm.get("description", "") if fm else ""

    # Separate frontmatter from body
    body_start = content.find("---", content.find("---") + 3) + 3 if "---" in content else 0
    body = content[body_start:].strip()

    # Extract existing sections
    existing_sections = re.findall(r"^##\s+(.+)", body, re.MULTILINE)

    # Enhance
    import copy
    old_fm_copy = copy.deepcopy(fm) if fm else None
    new_fm = enhance_frontmatter(fm, domains)
    new_body = enhance_body(body, existing_sections, domains)

    import yaml
    fm_yaml = yaml.dump(new_fm, default_flow_style=False, allow_unicode=True).strip()
    evolved = f"---\n{fm_yaml}\n---\n\n{new_body.strip()}\n"

    frontmatter_changed = old_fm_copy != new_fm if old_fm_copy else bool(new_fm)

    # Build evolution record
    record = {
        "skill_id": skill["id"],
        "name": old_name,
        "domains": domains,
        "old_description": old_desc,
        "new_description": new_fm.get("description", ""),
        "sections_added": list(set(s.strip("## ") for s in SECTION_ENHANCEMENTS) - set(existing_sections)),
        "frontmatter_enhanced": frontmatter_changed,
        "complexity": skill.get("complexity", "unknown"),
        "evolved_at": datetime.now(timezone.utc).isoformat(),
    }

    return evolved, record


def evolve():
    log("=" * 60)
    log("Skill Evolution MVP - Day 4")
    log("=" * 60)

    registry = load_json(REGISTRY_DIR / "skills-registry.json")
    if not registry:
        log("No skills found. Run discovery.py first.")
        return

    clusters = load_json(REGISTRY_DIR / "clusters.json")
    gaps = load_json(REGISTRY_DIR / "gaps.json")
    log(f"Loaded {len(registry)} skills to evolve")
    log(f"Loaded analysis from {len(clusters.get('clusters', {}))} clusters")

    EVOLVED_DIR.mkdir(parents=True, exist_ok=True)

    # Select candidates for evolution
    # Priority: underdeveloped domains first, then low-quality skills
    gap_domains = set()
    for g in gaps.get("gaps", {}).get("underdeveloped", []):
        gap_domains.add(g["domain"])

    candidates = []
    for skill in registry:
        domains = skill.get("domains", [])
        priority = 0
        if any(d in gap_domains for d in domains):
            priority += 2
        if skill.get("status") == "analyzed":
            priority += 1
        if skill.get("complexity") == "low":
            priority += 1
        candidates.append((priority, skill))

    candidates.sort(key=lambda x: -x[0])
    log(f"Evolution candidates (sorted by priority): {len(candidates)}")

    evolved_count = 0
    evolution_log = []

    for priority, skill in candidates:
        skill_id = skill["id"]
        content = read_raw_skill(skill)
        if not content:
            continue

        try:
            evolved_content, record = evolve_skill(skill, content)
        except Exception as e:
            log(f"  Failed: {skill.get('name', '?')} - {e}", "WARN")
            continue

        # Save evolved skill
        safe_name = re.sub(r'[<>:"/\\|?*]', "_", skill_id)
        evolved_path = EVOLVED_DIR / f"{safe_name}.md"
        evolved_path.write_text(evolved_content, encoding="utf-8")

        evolution_log.append(record)
        evolved_count += 1

        if evolved_count % 10 == 0:
            log(f"  Evolved: {evolved_count}/{len(candidates)}")

    # Write evolution log
    log_path = EVOLVED_DIR / "evolution-log.json"
    save_json(log_path, {
        "evolution_date": datetime.now(timezone.utc).isoformat(),
        "total_candidates": len(candidates),
        "total_evolved": evolved_count,
        "domain_gaps_targeted": list(gap_domains),
        "evolution_log": evolution_log,
    })

    # Summary stats
    domain_counts = defaultdict(int)
    for rec in evolution_log:
        for d in rec["domains"]:
            domain_counts[d] += 1

    total_added = sum(len(rec["sections_added"]) for rec in evolution_log)

    log(f"\n{'=' * 60}")
    log("Evolution complete!")
    log(f"  Skills evolved: {evolved_count}/{len(candidates)}")
    log(f"  Sections added: {total_added}")
    log(f"  Frontmatters enhanced: {sum(1 for r in evolution_log if r['frontmatter_enhanced'])}")
    log(f"  By domain:")
    for domain, count in sorted(domain_counts.items(), key=lambda x: -x[1])[:10]:
        log(f"    {domain}: {count}")
    log(f"  Evolved skills: {EVOLVED_DIR}/")
    log(f"  Evolution log: {log_path}")
    log("=" * 60)


if __name__ == "__main__":
    evolve()
