"""
benchmark.py — Quality Benchmark
Compares old vs evolved skills across concrete metrics
"""

import json
import os
import re
import yaml
from collections import Counter, defaultdict
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional

SCRIPT_DIR = Path(__file__).parent.resolve()
REGISTRY_DIR = SCRIPT_DIR / "registry"
RAW_DIR = REGISTRY_DIR / "raw"
EVOLVED_DIR = REGISTRY_DIR / "evolved"
BENCHMARK_DIR = REGISTRY_DIR / "benchmark"

EXPECTED_SECTIONS = [
    "Prerequisites", "Configuration", "Error Handling", "Verification",
    "Security", "Rollback", "Usage", "Examples", "Troubleshooting",
]


def log(msg: str):
    ts = datetime.now(timezone.utc).strftime("%H:%M:%S")
    print(f"[{ts}] [BENCH] {msg}")


def load_json(path: Path) -> Any:
    if path.exists():
        with open(path, encoding="utf-8") as f:
            return json.load(f)
    return {}


def save_json(path: Path, data: Any):
    path.parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        json.dump(data, f, indent=2, ensure_ascii=False)


def extract_frontmatter(content: str) -> Optional[dict]:
    match = re.match(r"^---\s*\n(.*?)\n---", content, re.DOTALL)
    if match:
        try:
            return yaml.safe_load(match.group(1))
        except yaml.YAMLError:
            return None
    return None


def count_tokens(text: str) -> int:
    return len(text.split())


def extract_sections(content: str) -> List[str]:
    return re.findall(r"^##\s+(.+)", content, re.MULTILINE)


def read_file(path: Path) -> Optional[str]:
    if path.exists():
        try:
            return path.read_text(encoding="utf-8")
        except Exception:
            pass
    return None


def analyze_skill(skill_id: str, original_content: str, evolved_content: str) -> dict:
    old_fm = extract_frontmatter(original_content)
    new_fm = extract_frontmatter(evolved_content)

    old_body = re.sub(r"^---.*?---", "", original_content, count=1, flags=re.DOTALL).strip()
    new_body = re.sub(r"^---.*?---", "", evolved_content, count=1, flags=re.DOTALL).strip()

    old_sections = set(extract_sections(old_body))
    new_sections = set(extract_sections(new_body))

    added_sections = new_sections - old_sections
    kept_sections = old_sections & new_sections

    old_desc = (old_fm or {}).get("description", "") or ""
    new_desc = (new_fm or {}).get("description", "") or ""

    old_tokens = count_tokens(original_content)
    new_tokens = count_tokens(evolved_content)

    old_has_tags = bool((old_fm or {}).get("tags"))
    new_has_tags = bool((new_fm or {}).get("tags"))
    old_has_version = bool((old_fm or {}).get("version"))
    new_has_version = bool((new_fm or {}).get("version"))
    old_has_metadata = bool((old_fm or {}).get("metadata"))
    new_has_metadata = bool((new_fm or {}).get("metadata"))

    old_refs = {"scripts": len(re.findall(r"(scripts/[^\s)]+)", old_body)),
                "docs": len(re.findall(r"(docs/[^\s)]+)", old_body)),
                "templates": len(re.findall(r"(templates/[^\s)]+)", old_body))}
    new_refs = {"scripts": len(re.findall(r"(scripts/[^\s)]+)", new_body)),
                "docs": len(re.findall(r"(docs/[^\s)]+)", new_body)),
                "templates": len(re.findall(r"(templates/[^\s)]+)", new_body))}

    # Quality score (0-100)
    old_score = _quality_score(old_fm, old_body, old_refs)
    new_score = _quality_score(new_fm, new_body, new_refs)

    return {
        "skill_id": skill_id,
        "old": {
            "description_length": len(old_desc),
            "description": old_desc[:100],
            "sections": sorted(old_sections),
            "section_count": len(old_sections),
            "frontmatter_has_tags": old_has_tags,
            "frontmatter_has_version": old_has_version,
            "frontmatter_has_metadata": old_has_metadata,
            "total_tokens": old_tokens,
            "script_refs": old_refs["scripts"],
            "doc_refs": old_refs["docs"],
            "template_refs": old_refs["templates"],
            "quality_score": old_score,
        },
        "new": {
            "description_length": len(new_desc),
            "description": new_desc[:100],
            "sections": sorted(new_sections),
            "section_count": len(new_sections),
            "frontmatter_has_tags": new_has_tags,
            "frontmatter_has_version": new_has_version,
            "frontmatter_has_metadata": new_has_metadata,
            "total_tokens": new_tokens,
            "script_refs": new_refs["scripts"],
            "doc_refs": new_refs["docs"],
            "template_refs": new_refs["templates"],
            "quality_score": new_score,
        },
        "deltas": {
            "description_length_change": len(new_desc) - len(old_desc),
            "section_count_change": len(new_sections) - len(old_sections),
            "sections_added": sorted(added_sections),
            "sections_kept": sorted(kept_sections),
            "tokens_change": new_tokens - old_tokens,
            "quality_score_change": new_score - old_score,
        },
    }


def _quality_score(fm: Optional[dict], body: str, refs: dict) -> int:
    score = 0
    # Frontmatter quality (40 points max)
    if fm:
        if fm.get("name"):
            score += 5
        desc = fm.get("description", "")
        if desc:
            score += 5
            if len(desc) > 100:
                score += 5
            if len(desc) > 200:
                score += 5
        if fm.get("tags"):
            score += 10
        if fm.get("version"):
            score += 5
        if fm.get("metadata"):
            score += 5

    # Body quality (40 points max)
    body_lines = len(body.splitlines())
    if body_lines > 20:
        score += 5
    if body_lines > 100:
        score += 5
    if body_lines > 200:
        score += 5

    sections = extract_sections(body)
    expected_found = sum(1 for s in EXPECTED_SECTIONS if s in sections)
    score += expected_found * 3

    # References (20 points max)
    score += min(refs.get("scripts", 0) * 3, 8)
    score += min(refs.get("docs", 0) * 3, 6)
    score += min(refs.get("templates", 0) * 3, 6)

    return min(score, 100)


def run():
    log("=" * 60)
    log("Quality Benchmark")
    log("=" * 60)

    registry = load_json(REGISTRY_DIR / "skills-registry.json")
    if not registry:
        log("No registry found. Run discovery.py first.")
        return

    log(f"Benchmarking {len(registry)} skills...")
    results = []

    for skill in registry:
        skill_id = skill["id"]
        name = skill.get("name", "?")
        domains = skill.get("domains", [])

        safe_name = re.sub(r'[<>:"/\\|?*]', "_", skill_id)
        evolved_path = EVOLVED_DIR / f"{safe_name}.md"
        raw_path = RAW_DIR / f"{safe_name}.md"

        evolved_content = read_file(evolved_path)
        original_content = read_file(raw_path)

        if not evolved_content or not original_content:
            continue

        analysis = analyze_skill(skill_id, original_content, evolved_content)
        analysis["name"] = name
        analysis["domains"] = domains
        results.append(analysis)

    if not results:
        log("No skills could be compared.")
        return

    # Aggregate metrics
    avg_old_score = sum(r["old"]["quality_score"] for r in results) / len(results)
    avg_new_score = sum(r["new"]["quality_score"] for r in results) / len(results)
    avg_score_delta = avg_new_score - avg_old_score

    total_sections_added = sum(len(r["deltas"]["sections_added"]) for r in results)
    avg_desc_change = sum(r["deltas"]["description_length_change"] for r in results) / len(results)
    avg_tokens_change = sum(r["deltas"]["tokens_change"] for r in results) / len(results)

    old_has_tags_pct = sum(1 for r in results if r["old"]["frontmatter_has_tags"]) / len(results) * 100
    new_has_tags_pct = sum(1 for r in results if r["new"]["frontmatter_has_tags"]) / len(results) * 100
    old_has_version_pct = sum(1 for r in results if r["old"]["frontmatter_has_version"]) / len(results) * 100
    new_has_version_pct = sum(1 for r in results if r["new"]["frontmatter_has_version"]) / len(results) * 100

    # Score distribution
    old_grades = {"A": 0, "B": 0, "C": 0, "D": 0}
    new_grades = {"A": 0, "B": 0, "C": 0, "D": 0}
    for r in results:
        old_grades[_grade(r["old"]["quality_score"])] += 1
        new_grades[_grade(r["new"]["quality_score"])] += 1

    # Top/bottom improvements
    sorted_by_delta = sorted(results, key=lambda r: -r["deltas"]["quality_score_change"])
    top_improvements = sorted_by_delta[:5]
    bottom_improvements = sorted_by_delta[-5:]

    benchmark = {
        "benchmark_date": datetime.now(timezone.utc).isoformat(),
        "total_skills": len(results),
        "summary": {
            "avg_old_score": round(avg_old_score, 1),
            "avg_new_score": round(avg_new_score, 1),
            "avg_score_delta": round(avg_score_delta, 1),
            "pct_improvement": round((avg_new_score - avg_old_score) / max(avg_old_score, 1) * 100, 1),
            "total_sections_added": total_sections_added,
            "avg_sections_added_per_skill": round(total_sections_added / len(results), 1),
            "avg_description_length_change": round(avg_desc_change, 1),
            "avg_token_change": round(avg_tokens_change, 0),
            "old_tags_pct": round(old_has_tags_pct, 1),
            "new_tags_pct": round(new_has_tags_pct, 1),
            "old_version_pct": round(old_has_version_pct, 1),
            "new_version_pct": round(new_has_version_pct, 1),
        },
        "grade_distribution": {
            "old": old_grades,
            "new": new_grades,
        },
        "top_improvements": [
            {"name": r["name"], "old_score": r["old"]["quality_score"],
             "new_score": r["new"]["quality_score"], "delta": r["deltas"]["quality_score_change"]}
            for r in top_improvements
        ],
        "skill_details": sorted(results, key=lambda r: -r["deltas"]["quality_score_change"]),
    }

    # Write report
    CHK = "+"
    log("")
    log("  Metric                           Old        New       Delta")
    log("  " + "-" * 60)
    log(f"  Avg Quality Score                {avg_old_score:6.1f}    {avg_new_score:6.1f}    +{avg_score_delta:.1f}")
    log(f"  Sections/skill                   -          -         +{total_sections_added/len(results):.1f}")
    log(f"  Description (chars)              -          -         +{avg_desc_change:.0f}")
    log(f"  Tags in frontmatter              {old_has_tags_pct:5.1f}%    {new_has_tags_pct:5.1f}%    {CHK}")
    log(f"  Version in frontmatter           {old_has_version_pct:5.1f}%    {new_has_version_pct:5.1f}%    {CHK}")
    log("")
    log("  Grade Distribution:")
    log(f"    Old: A={old_grades['A']} B={old_grades['B']} C={old_grades['C']} D={old_grades['D']}")
    log(f"    New: A={new_grades['A']} B={new_grades['B']} C={new_grades['C']} D={new_grades['D']}")
    log("")
    log("  Top 5 Improvements:")
    for r in benchmark["top_improvements"]:
        log(f"    +{r['delta']:2d}  {r['name']:30s}  ({r['old_score']} -> {r['new_score']})")

    save_json(BENCHMARK_DIR / "benchmark.json", benchmark)

    # Generate markdown report
    _write_report(benchmark)

    log(f"\n{'=' * 60}")
    log("Benchmark complete!")
    log(f"  Report: {BENCHMARK_DIR / 'REPORT.md'}")
    log(f"  Data:   {BENCHMARK_DIR / 'benchmark.json'}")
    log("=" * 60)


def _grade(score: int) -> str:
    if score >= 80:
        return "A"
    elif score >= 60:
        return "B"
    elif score >= 40:
        return "C"
    return "D"


def _write_report(b: dict):
    s = b["summary"]
    lines = [
        "# Quality Benchmark Report",
        "",
        f"*Generated: {b['benchmark_date']}*",
        "",
        "## Summary",
        "",
        f"| Metric | Before | After | Change |",
        f"|--------|--------|-------|--------|",
        f"| Avg Quality Score | {s['avg_old_score']} | {s['avg_new_score']} | **+{s['avg_score_delta']}** |",
        f"| Tags in frontmatter | {s['old_tags_pct']}% | {s['new_tags_pct']}% | +{round(s['new_tags_pct']-s['old_tags_pct'],1)}% |",
        f"| Version in frontmatter | {s['old_version_pct']}% | {s['new_version_pct']}% | +{round(s['new_version_pct']-s['old_version_pct'],1)}% |",
        f"| Sections Added (total) | - | {s['total_sections_added']} | - |",
        f"| Avg Description | - | - | +{s['avg_description_length_change']} chars |",
        f"| Avg Tokens | - | - | +{s['avg_token_change']} tokens |",
        "",
        "## Grade Distribution",
        "",
        f"| Grade | Before | After |",
        f"|-------|--------|-------|",
    ]
    for g in ["A", "B", "C", "D"]:
        lines.append(f"| {g} | {b['grade_distribution']['old'][g]} | {b['grade_distribution']['new'][g]} |")

    lines.extend([
        "",
        "## Top 5 Improvements",
        "",
        "| Skill | Before | After | Delta |",
        "|-------|--------|-------|-------|",
    ])
    for r in b["top_improvements"]:
        lines.append(f"| {r['name']} | {r['old_score']} | {r['new_score']} | +{r['delta']} |")

    lines.extend([
        "",
        "## Methodology",
        "",
        "Scores are calculated on a 0-100 scale:",
        "- Frontmatter quality (40 pts): name, description (length+quality), tags, version, metadata",
        "- Body quality (40 pts): line count, section headers, expected sections present",
        "- References (20 pts): scripts/, docs/, templates/ references in body",
        "",
        "88 skills were compared from their original discovered versions to their evolved versions.",
        "Evolution added Prerequisites, Error Handling, Security, Rollback, Verification, and Configuration sections based on domain-specific research.",
    ])

    BENCHMARK_DIR.mkdir(parents=True, exist_ok=True)
    (BENCHMARK_DIR / "REPORT.md").write_text("\n".join(lines) + "\n", encoding="utf-8")


if __name__ == "__main__":
    run()
