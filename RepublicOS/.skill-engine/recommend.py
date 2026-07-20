"""
recommend.py — Recommend skills based on project context
Analyzes a project directory and recommends relevant skills.
"""

import json
import os
import re
import subprocess
import sys
from pathlib import Path

SCRIPT_DIR = Path(__file__).parent.resolve()
REGISTRY_DIR = SCRIPT_DIR / "registry"


def load_json(path: Path):
    if path.exists():
        with open(path, encoding="utf-8") as f:
            return json.load(f)
    return {}


def log(msg: str):
    print(f"[RECOMMEND] {msg}")


def analyze_project(path: Path) -> dict:
    context = {
        "files": [],
        "languages": set(),
        "frameworks": set(),
        "has_tests": False,
        "has_docker": False,
        "has_ci": False,
        "has_api": False,
        "has_database": False,
        "has_cli": False,
        "total_files": 0,
    }

    if not path.exists():
        return context

    for f in path.rglob("*"):
        if f.is_file() and not f.name.startswith("."):
            rel = f.relative_to(path)
            context["files"].append(str(rel))
            context["total_files"] += 1

            ext = f.suffix.lower()
            if ext in {".py"}:
                context["languages"].add("python")
            elif ext in {".js", ".ts", ".jsx", ".tsx"}:
                context["languages"].add("javascript")
            elif ext in {".rs"}:
                context["languages"].add("rust")
            elif ext in {".go"}:
                context["languages"].add("go")
            elif ext in {".java", ".kt"}:
                context["languages"].add("jvm")
            elif ext in {".rb"}:
                context["languages"].add("ruby")

            name = f.name.lower()
            if name in {"test_*.py", "*_test.py", "*.test.js", "*.spec.ts"} or "test" in name:
                context["has_tests"] = True
            if name == "dockerfile" or name.startswith("docker-compose"):
                context["has_docker"] = True
            if ".github" in str(rel) or ".gitlab" in str(rel) or "jenkins" in str(rel):
                context["has_ci"] = True
            if "api" in name or "route" in name or "endpoint" in name:
                context["has_api"] = True
            if "db" in name or "sql" in name or "schema" in name or "model" in name or "migration" in name:
                context["has_database"] = True
            if "cli" in name or "main.py" in name or "entry" in name:
                context["has_cli"] = True

            if "requirements.txt" in name or "pyproject.toml" in name:
                try:
                    content = f.read_text(encoding="utf-8", errors="ignore")
                    for lib in ["django", "flask", "fastapi", "pytest", "sqlalchemy",
                                "celery", "redis", "docker", "boto3", "tensorflow",
                                "torch", "pandas", "numpy", "scikit"]:
                        if lib in content.lower():
                            context["frameworks"].add(lib)
                except Exception:
                    pass

            if ext == ".json" and ("package" in name or "composer" in name):
                try:
                    content = f.read_text(encoding="utf-8", errors="ignore")
                    data = json.loads(content)
                    deps = {}
                    for section in ["dependencies", "devDependencies", "peerDependencies"]:
                        deps.update(data.get(section, {}))
                    for lib in ["react", "vue", "angular", "next", "express",
                                "jest", "mocha", "webpack", "tailwind", "prisma"]:
                        if lib in deps:
                            context["frameworks"].add(lib)
                except Exception:
                    pass

    context["files"] = context["files"][:100]
    return context


def score_relevance(context: dict, skill: dict) -> float:
    score = 0.0
    name = skill.get("name", "").lower()
    desc = (skill.get("description", "") or "").lower()
    domains = [d.lower() for d in skill.get("domains", [])]

    # Language match
    for lang in context["languages"]:
        if lang in name or lang in desc:
            score += 15
        if lang in domains:
            score += 10

    # Framework match
    for fw in context["frameworks"]:
        if fw in name or fw in desc:
            score += 20
        if fw in domains:
            score += 10

    # Context signals
    if context["has_tests"] and ("test" in domains or "testing" in desc):
        score += 25
    if context["has_docker"] and ("deploy" in domains or "docker" in desc or "infrastructure" in domains):
        score += 20
    if context["has_ci"] and ("deploy" in domains or "ci" in desc):
        score += 15
    if context["has_api"] and ("api" in domains or "rest" in desc):
        score += 25
    if context["has_database"] and ("database" in domains or "sql" in desc or "db" in desc):
        score += 25
    if context["has_cli"] and ("cli" in domains or "command" in desc):
        score += 15

    # Domain boost
    for domain in domains:
        if domain == "utility":
            score += 5
        elif domain == "documentation":
            score += 5
        elif domain == "security":
            if any(s in context["files"] for s in [".env", "secret", ".gitignore", "auth"]):
                score += 20
        elif domain == "monitoring":
            if context["total_files"] > 50:
                score += 15

    # Quality boost
    quality_score = skill.get("quality_score", 0)
    score += quality_score * 0.2

    return score


def recommend(project_path: str = None, top_n: int = 10, min_score: float = 10):
    registry = load_json(REGISTRY_DIR / "skills-registry.json") or []
    quality = load_json(REGISTRY_DIR / "quality.json") or {}

    scores_map = {}
    if isinstance(quality, dict):
        for s in quality.get("all_scores", []):
            if isinstance(s, dict):
                scores_map[s.get("name", "")] = s.get("score", 0)

    for s in (registry if isinstance(registry, list) else []):
        s["quality_score"] = scores_map.get(s.get("name", ""), 0)

    if project_path:
        proj = Path(project_path)
        log(f"Analyzing project: {proj.resolve()}")
        context = analyze_project(proj)
        log(f"  Languages: {', '.join(context['languages']) or 'none'}")
        log(f"  Frameworks: {', '.join(context['frameworks']) or 'none'}")
        log(f"  Tests: {context['has_tests']}, Docker: {context['has_docker']}, CI: {context['has_ci']}")
        log(f"  API: {context['has_api']}, DB: {context['has_database']}, CLI: {context['has_cli']}")
        log(f"  Total files: {context['total_files']}")

        scored = []
        for s in (registry if isinstance(registry, list) else []):
            rel = score_relevance(context, s)
            if rel >= min_score:
                scored.append((rel, s))
        scored.sort(key=lambda x: -x[0])
    else:
        scored = []
        for s in (registry if isinstance(registry, list) else []):
            q = s.get("quality_score", 0)
            if q >= min_score:
                scored.append((q, s))
        scored.sort(key=lambda x: -x[0])
        context = None

    results = scored[:top_n]

    if not results:
        log("No relevant skills found")
        return

    print(f"\n{'='*60}")
    if project_path:
        print(f"  Recommended Skills for: {Path(project_path).resolve()}")
    else:
        print(f"  Top Skills by Quality")
    print(f"{'='*60}")
    print(f"\n{'Rank':<5} {'Skill':<30} {'Score':>6} {'Domain':<20}")
    print("-" * 65)

    for i, (score, skill) in enumerate(results, 1):
        name = skill.get("name", "?")
        domains = ", ".join(skill.get("domains", [])[:2])
        quality_score = skill.get("quality_score", 0)
        rank_mark = "+" if score > 50 else " "
        domain_str = domains[:18]
        print(f"  {rank_mark} {i:<2}  {name:<28} {score:>4.0f}  {domain_str:<18}")

    print(f"\n{len(results)} recommendations")

    return results


def main():
    import argparse
    parser = argparse.ArgumentParser(description="Recommend skills for a project")
    parser.add_argument("project_path", nargs="?", help="Path to project directory to analyze")
    parser.add_argument("--top", "-n", type=int, default=10, help="Number of recommendations")
    parser.add_argument("--min-score", "-m", type=float, default=10, help="Minimum relevance score")
    parser.add_argument("--json", "-j", action="store_true", help="JSON output")
    args = parser.parse_args()

    results = recommend(
        project_path=args.project_path,
        top_n=args.top,
        min_score=args.min_score,
    )

    if args.json and results:
        data = [
            {"name": s.get("name"), "score": round(sc, 1),
             "domains": s.get("domains"), "source": s.get("source")}
            for sc, s in results
        ]
        print(json.dumps(data, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
