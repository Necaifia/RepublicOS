"""
search.py — Search skills from the command line
Quickly find skills by name, domain, quality, source, or description.
"""

import json
import shutil
from pathlib import Path

SCRIPT_DIR = Path(__file__).parent.resolve()
REGISTRY_DIR = SCRIPT_DIR / "registry"
EVOLVED_DIR = REGISTRY_DIR / "evolved"


def load_json(path: Path):
    if path.exists():
        with open(path, encoding="utf-8") as f:
            return json.load(f)
    return {}


def terminal_width() -> int:
    try:
        return shutil.get_terminal_size().columns
    except Exception:
        return 80


def truncate(text: str, max_len: int) -> str:
    if len(text) <= max_len:
        return text
    return text[: max_len - 3] + "..."


def color(text: str, code: str) -> str:
    colors = {
        "green": "\033[92m",
        "cyan": "\033[96m",
        "yellow": "\033[93m",
        "red": "\033[91m",
        "blue": "\033[94m",
        "bold": "\033[1m",
        "dim": "\033[2m",
        "reset": "\033[0m",
    }
    start = colors.get(code, "")
    end = colors["reset"]
    return f"{start}{text}{end}"


def grade_color(score: int) -> str:
    if score >= 80:
        return "green"
    if score >= 60:
        return "blue"
    if score >= 40:
        return "yellow"
    return "red"


def search(query: str = "", domain: str = "", source: str = "",
           min_score: int = 0, max_results: int = 20,
           json_output: bool = False, show_content: bool = False):
    registry = load_json(REGISTRY_DIR / "skills-registry.json") or []
    quality = load_json(REGISTRY_DIR / "quality.json") or {}

    scores = {}
    if isinstance(quality, dict):
        for s in quality.get("all_scores", []):
            if isinstance(s, dict):
                name = s.get("name", "")
                scores[name] = {
                    "score": s.get("score", s.get("quality_score", 0)),
                    "grade": s.get("grade", ""),
                }

    results = []
    for skill in (registry if isinstance(registry, list) else []):
        name = skill.get("name", "")
        desc = (skill.get("description", "") or "").lower()
        name_lower = name.lower()
        domains = " ".join(skill.get("domains", [])).lower()
        sources = (skill.get("source", "") or "").lower()

        q = query.lower()
        if q and q not in name_lower and q not in desc and q not in domains:
            continue
        if domain and domain.lower() not in domains:
            continue
        if source and source.lower() not in sources:
            continue

        score_info = scores.get(name, {})
        score = score_info.get("score", 0) if isinstance(score_info, dict) else 0
        if score < min_score:
            continue

        evolved_path = EVOLVED_DIR / f"{name}.md"
        if not evolved_path.exists():
            sid = skill.get("id", name)
            safe = sid.replace("/", "_").replace(":", "_")
            evolved_path = EVOLVED_DIR / f"{safe}.md"
        has_evolved = evolved_path.exists()

        results.append({
            "name": name,
            "description": skill.get("description", "") or "",
            "domains": skill.get("domains", []) or [],
            "source": skill.get("source", "") or "",
            "quality_score": score,
            "grade": score_info.get("grade", ""),
            "evolved": has_evolved,
            "body_lines": skill.get("body_lines", 0),
        })

    results.sort(key=lambda r: -r["quality_score"])

    if query or domain or source or min_score > 0:
        results = [r for r in results if True]  # already filtered above

    results = results[:max_results]

    if json_output:
        print(json.dumps({"total": len(results), "results": results}, ensure_ascii=False, indent=2))
        return

    if not results:
        print(color("No skills found matching your criteria.", "yellow"))
        return

    tw = terminal_width()
    name_w = min(30, tw // 4)
    domain_w = min(18, tw // 6)
    score_w = 8
    source_w = min(16, tw // 6)
    desc_w = tw - name_w - domain_w - score_w - source_w - 10

    print()
    header = (
        f"{'Skill':<{name_w}} {'Score':>{score_w}} {'Domain':<{domain_w}} "
        f"{'Source':<{source_w}} {'Description'}"
    )
    print(color(header, "bold"))
    print(color("-" * tw, "dim"))

    for r in results:
        domains_str = ", ".join(r["domains"][:2])
        if len(r["domains"]) > 2:
            domains_str += "..."

        score_str = str(r["quality_score"])
        if r["grade"]:
            score_str = f"{r['quality_score']}{color(r['grade'], grade_color(r['quality_score']))}"

        ev_mark = color("+", "green") if r["evolved"] else " "
        desc = r["description"].replace("\n", " ")

        if tw > 80:
            line = (
                f" {ev_mark} {color(r['name'], 'cyan'):<{name_w-2}} "
                f"{score_str:>{score_w}} "
                f"{color(domains_str, 'blue'):<{domain_w}} "
                f"{truncate(r['source'], source_w):<{source_w}} "
                f"{truncate(desc, desc_w)}"
            )
        else:
            line = (
                f" {ev_mark} {color(r['name'], 'cyan'):<{name_w-2}} "
                f"{score_str:>{score_w}} "
                f"{color(domains_str, 'blue'):<{domain_w}}"
            )
        print(line)

    print(color(f"\n{len(results)} result(s)", "dim"))

    if show_content and results:
        name = results[0]["name"]
        evolved_path = EVOLVED_DIR / f"{results[0]['name']}.md"
        if not evolved_path.exists():
            safe = results[0]["name"].replace("/", "_").replace(":", "_")
            evolved_path = EVOLVED_DIR / f"{safe}.md"
        if evolved_path.exists():
            print(color(f"\n--- {name} ---", "bold"))
            print(evolved_path.read_text(encoding="utf-8"))


def list_domains():
    clusters_data = load_json(REGISTRY_DIR / "clusters.json") or {}
    domain_clusters = clusters_data.get("clusters", {}) if isinstance(clusters_data, dict) else {}
    gaps = load_json(REGISTRY_DIR / "gaps.json") or {}
    gaps_list = gaps.get("gaps", {}).get("underdeveloped", []) if isinstance(gaps, dict) else []
    gap_domains = {g.get("domain"): g.get("deficit", 0) for g in gaps_list}

    print(color(f"\n{'Domain':<22} {'Skills':>7} {'Gap':>5}", "bold"))
    print(color("-" * 40, "dim"))
    for name, info in sorted(domain_clusters.items()):
        count = info.get("skill_count", 0) if isinstance(info, dict) else 0
        gap = gap_domains.get(name, 0)
        gap_str = color(f"-{gap}", "yellow") if gap else color("0", "dim")
        print(f"  {name:<20} {count:>4}  {gap_str:>5}")


def list_sources():
    sources = load_json(REGISTRY_DIR / "sources.json") or {}
    print(color(f"\n{'Source':<35} {'Status':<14} {'Skills':>6}", "bold"))
    print(color("-" * 60, "dim"))
    for name, info in sorted(sources.items()):
        label = info.get("label", name)[:33]
        status = info.get("status", "?")
        skills = info.get("skills_found", 0)
        status_color = "green" if status == "complete" else "yellow" if status == "pending" else "red"
        print(f"  {label:<34} {color(status, status_color):<14} {skills:>4}")


def main():
    import argparse
    parser = argparse.ArgumentParser(description="Search the skill registry")
    parser.add_argument("query", nargs="?", default="", help="Search term")
    parser.add_argument("--domain", "-d", help="Filter by domain")
    parser.add_argument("--source", "-s", help="Filter by source repo")
    parser.add_argument("--min-score", "-m", type=int, default=0, help="Minimum quality score")
    parser.add_argument("--max", "-n", type=int, default=20, help="Max results")
    parser.add_argument("--json", "-j", action="store_true", help="JSON output")
    parser.add_argument("--content", "-c", action="store_true", help="Show full skill content")
    parser.add_argument("--domains", action="store_true", help="List domains and counts")
    parser.add_argument("--sources", action="store_true", help="List sources and status")

    args = parser.parse_args()

    if args.domains:
        list_domains()
        return
    if args.sources:
        list_sources()
        return

    search(
        query=args.query,
        domain=args.domain or "",
        source=args.source or "",
        min_score=args.min_score,
        max_results=args.max,
        json_output=args.json,
        show_content=args.content,
    )


if __name__ == "__main__":
    main()
