"""
analyzer.py — Skill Analyzer (Day 2)
Part of RepublicOS Skill Evolution Pipeline

Analyzes discovered skills: clusters by function, detects patterns,
extracts common instructions, and identifies gaps.
"""

import json
import os
import re
from collections import Counter, defaultdict
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List

SCRIPT_DIR = Path(__file__).parent.resolve()
REGISTRY_DIR = SCRIPT_DIR / "registry"
RAW_DIR = REGISTRY_DIR / "raw"

COMMON_SECTIONS = [
    "Error Handling", "Configuration", "Prerequisites", "Setup",
    "Installation", "Usage", "Examples", "Troubleshooting",
    "Best Practices", "Security", "Testing", "Rollback", "Cleanup",
    "Verification", "Validation", "Monorepo", "Permissions",
]

COMMON_TOOLS = [
    "read_file", "search_codebase", "run_commands", "fetch_web_content",
    "apply_patch", "editor", "use_skill", "ask_question", "execute_command",
    "bash", "powershell", "shell", "npm", "yarn", "pip", "cargo", "docker",
    "kubectl", "gh", "git", "aws", "gcloud", "az",
]


def log(msg: str):
    ts = datetime.now(timezone.utc).strftime("%H:%M:%S")
    print(f"[{ts}] [ANALYZER] {msg}")


def load_json(path: Path) -> Any:
    if path.exists():
        with open(path, encoding="utf-8") as f:
            return json.load(f)
    return None


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


def extract_sections(content: str) -> List[str]:
    return re.findall(r"^##\s+(.+)", content, re.MULTILINE)


def extract_tools_mentioned(content: str) -> List[str]:
    found = []
    for tool in COMMON_TOOLS:
        if tool in content.lower():
            found.append(tool)
    return found


def quality_score(skill: dict) -> dict:
    score = 0
    reasons = []

    if skill.get("frontmatter"):
        fm = skill["frontmatter"]
        if fm.get("name"):
            score += 10
        if fm.get("description"):
            desc = fm["description"]
            score += 10
            if len(desc) > 50:
                score += 5
            if len(desc) > 150:
                score += 5
        if fm.get("tags"):
            score += 10
            reasons.append("has tags")
        if fm.get("version"):
            score += 5
            reasons.append("has version")
        if fm.get("metadata"):
            score += 5
            reasons.append("has metadata")

    body_lines = skill.get("body_lines", 0)
    if body_lines > 20:
        score += 10
    if body_lines > 100:
        score += 10
    if body_lines > 300:
        score += 10

    if skill.get("scripts"):
        score += len(skill["scripts"]) * 5
        reasons.append(f"{len(skill['scripts'])} scripts")
    if skill.get("docs"):
        score += 10
        reasons.append("has docs/")
    if skill.get("templates"):
        score += 10
        reasons.append("has templates")

    if skill.get("complexity") == "high":
        score += 10

    grade = "A" if score >= 80 else "B" if score >= 60 else "C" if score >= 40 else "D"
    return {"score": score, "grade": grade, "reasons": reasons}


def analyze():
    log("=" * 60)
    log("Skill Analyzer - Day 2")
    log("=" * 60)

    registry = load_json(REGISTRY_DIR / "skills-registry.json")
    if not registry:
        log("No skills found. Run discovery.py first.")
        return

    log(f"Loaded {len(registry)} skills")

    # ── 1. Domain Clustering ──
    log("\n-- 1. Domain Clustering --")
    domain_skills = defaultdict(list)
    domain_subdomains = defaultdict(set)
    domain_sources = defaultdict(set)

    for skill in registry:
        for d in skill.get("domains", ["uncategorized"]):
            domain_skills[d].append(skill)
            domain_sources[d].add(skill.get("source", "unknown"))

    # Detect sub-domains from skill names
    for domain, skills in domain_skills.items():
        for s in skills:
            name = s.get("name", "")
            parts = name.replace("-", " ").replace("_", " ").split()
            domain_subdomains[domain].update(parts[:3])

    clusters = {}
    for domain, skills in sorted(domain_skills.items()):
        clusters[domain] = {
            "skill_count": len(skills),
            "sources": sorted(domain_sources[domain]),
            "subdomains": sorted(domain_subdomains[domain])[:15],
            "avg_complexity": sum(
                1 for s in skills if s.get("complexity") == "high"
            ) / max(len(skills), 1),
            "skills": sorted(s.get("name", "") for s in skills),
        }
        log(f"  {domain:20s}: {len(skills):3d} skills, "
            f"{len(domain_sources[domain])} sources")

    # ── 2. Pattern Detection ──
    log("\n-- 2. Pattern Detection --")

    section_counter = Counter()
    tool_counter = Counter()
    domain_sections = defaultdict(Counter)
    domain_tools = defaultdict(Counter)

    for skill in registry:
        content = read_raw_skill(skill)
        if not content:
            continue

        sections = extract_sections(content)
        section_counter.update(sections)
        for d in skill.get("domains", ["uncategorized"]):
            domain_sections[d].update(sections)

        tools = extract_tools_mentioned(content)
        tool_counter.update(tools)
        for d in skill.get("domains", ["uncategorized"]):
            domain_tools[d].update(tools)

    top_sections = section_counter.most_common(20)
    top_tools = tool_counter.most_common(20)

    log("  Top sections across all skills:")
    for name, count in top_sections:
        log(f"    {name:30s}: {count}")

    log("  Top tools mentioned:")
    for name, count in top_tools:
        log(f"    {name:30s}: {count}")

    domain_patterns = {}
    for domain in sorted(domain_sections.keys()):
        secs = domain_sections[domain].most_common(10)
        tools = domain_tools[domain].most_common(10)
        domain_patterns[domain] = {
            "common_sections": [s for s, _ in secs],
            "common_tools": [t for t, _ in tools],
        }

    # ── 3. Gap Analysis ──
    log("\n-- 3. Gap Analysis --")

    expected_minimum = {
        "deploy": 15, "test": 15, "security": 15, "database": 15,
        "api": 15, "infrastructure": 10, "monitoring": 10,
        "code_review": 10, "documentation": 10, "communication": 5,
        "utility": 5,
    }

    gaps = {
        "underdeveloped": [],
        "overcrowded": [],
        "missing_combinations": [],
        "recommendations": [],
    }

    domain_counts = {d: len(skills) for d, skills in domain_skills.items()}
    for domain, expected in sorted(expected_minimum.items()):
        actual = domain_counts.get(domain, 0)
        if actual < expected:
            gaps["underdeveloped"].append({
                "domain": domain,
                "current": actual,
                "expected": expected,
                "deficit": expected - actual,
            })
            gaps["recommendations"].append(
                f"Create more {domain} skills ({actual}/{expected})"
            )
        elif actual > expected * 2:
            gaps["overcrowded"].append({
                "domain": domain,
                "current": actual,
                "suggested_threshold": expected,
            })

    # Check for valuable domain combinations that are missing
    valuable_pairs = [("security", "deploy"), ("database", "monitoring"),
                      ("test", "deploy"), ("api", "security"),
                      ("infrastructure", "monitoring")]
    for d1, d2 in valuable_pairs:
        both = sum(1 for s in registry
                   if d1 in s.get("domains", []) and d2 in s.get("domains", []))
        if both == 0:
            gaps["missing_combinations"].append(f"{d1}+{d2}")

    log(f"  Underdeveloped domains: {len(gaps['underdeveloped'])}")
    for g in gaps["underdeveloped"]:
        log(f"    {g['domain']:20s}: {g['current']}/{g['expected']}")
    log(f"  Missing combinations: {len(gaps['missing_combinations'])}")

    # ── 4. Quality Analysis ──
    log("\n-- 4. Quality Analysis --")

    quality_results = []
    for skill in registry:
        q = quality_score(skill)
        quality_results.append({
            "id": skill["id"],
            "name": skill.get("name", "?"),
            "source": skill.get("source", "?"),
            "score": q["score"],
            "grade": q["grade"],
            "complexity": skill.get("complexity", "?"),
            "body_lines": skill.get("body_lines", 0),
        })

    grades = Counter(q["grade"] for q in quality_results)
    avg_score = sum(q["score"] for q in quality_results) / max(len(quality_results), 1)
    log(f"  Average quality score: {avg_score:.1f}/100")
    log(f"  Grade distribution: {dict(grades)}")

    top_quality = sorted(quality_results, key=lambda x: -x["score"])[:5]
    log("  Top 5 by quality:")
    for q in top_quality:
        log(f"    {q['name']:30s} grade={q['grade']} score={q['score']} "
            f"({q['body_lines']} lines)")

    bottom_quality = sorted(quality_results, key=lambda x: x["score"])[:5]
    log("  Bottom 5 by quality:")
    for q in bottom_quality:
        log(f"    {q['name']:30s} grade={q['grade']} score={q['score']} "
            f"({q['body_lines']} lines)")

    # ── 5. Write Outputs ──
    log("\n-- Writing outputs --")

    save_json(REGISTRY_DIR / "clusters.json", {
        "analysis_date": datetime.now(timezone.utc).isoformat(),
        "total_skills": len(registry),
        "clusters": clusters,
    })

    save_json(REGISTRY_DIR / "patterns.json", {
        "analysis_date": datetime.now(timezone.utc).isoformat(),
        "top_sections": [{"section": s, "count": c} for s, c in top_sections],
        "top_tools": [{"tool": t, "count": c} for t, c in top_tools],
        "domain_patterns": domain_patterns,
    })

    save_json(REGISTRY_DIR / "gaps.json", {
        "analysis_date": datetime.now(timezone.utc).isoformat(),
        "total_skills": len(registry),
        "gaps": gaps,
    })

    save_json(REGISTRY_DIR / "quality.json", {
        "analysis_date": datetime.now(timezone.utc).isoformat(),
        "average_score": round(avg_score, 1),
        "grade_distribution": dict(grades),
        "all_scores": quality_results,
    })

    log("\n" + "=" * 60)
    log("Analysis complete!")
    log("  Output files:")
    log("    clusters.json  - Domain clustering")
    log("    patterns.json  - Recurring patterns")
    log("    gaps.json      - Gap analysis")
    log("    quality.json   - Quality scoring")
    log("=" * 60)


if __name__ == "__main__":
    analyze()
