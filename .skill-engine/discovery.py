"""
discovery.py — Skill Discovery Engine (Day 1)
Part of RepublicOS Skill Evolution Pipeline

Crawls 25+ sources for AI agent skills (Cline, Claude Code, etc.),
classifies by domain, and maintains a searchable registry.
Uses git shallow clones instead of GitHub API to avoid rate limits.
"""

import json
import os
import re
import subprocess
import sys
import time
import yaml
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional

# ── Constants ────────────────────────────────────────────────────────────────

SCRIPT_DIR = Path(__file__).parent.resolve()
REGISTRY_DIR = SCRIPT_DIR / "registry"
RAW_DIR = REGISTRY_DIR / "raw"
CACHE_DIR = SCRIPT_DIR / ".cache"
CONFIG_PATH = SCRIPT_DIR / "discovery-config.json"

DEFAULT_CONFIG = {
    "checkpoint_interval": 20,
    "known_repos": [
        {"name": "anthropics/skills", "path": "skills", "label": "Anthropic"},
        {"name": "cline/cline", "path": ".agents/skills", "label": "Cline"},
        {"name": "cline/skills", "path": "skills", "label": "Cline Skills"},
        {"name": "ClickHouse/agent-skills", "path": "skills", "label": "ClickHouse"},
        {"name": "supabase/agent-skills", "path": "skills", "label": "Supabase"},
        {"name": "ichuan/skills", "path": "skills", "label": "ichuan"},
        {"name": "bakaschwarz/agent-skills", "path": "skills", "label": "bakaschwarz"},
        {"name": "marcfargas/skills", "path": "skills", "label": "marcfargas"},
        {"name": "achu-s/skills", "path": "skills", "label": "achu-s"},
        {"name": "manifold-dev/skills", "path": "skills", "label": "Manifold"},
        {"name": "openai/openai-agents-python", "path": "examples/agent-skills", "label": "OpenAI Agents"},
        {"name": "crewAI/crewAI", "path": "src/crewai/utilities/skills", "label": "CrewAI"},
        {"name": "langchain-ai/langchain-skills", "path": "skills", "label": "LangChain"},
    ],
    "domains": {
        "deploy": ["deploy", "release", "rollout", "publish", "cd", "continuous delivery"],
        "test": ["test", "testing", "qa", "quality", "coverage", "assert"],
        "security": ["security", "audit", "vulnerability", "scan", "auth", "oauth", "secret"],
        "database": ["database", "db", "sql", "query", "migration", "schema"],
        "api": ["api", "rest", "graphql", "endpoint", "webhook"],
        "infrastructure": ["infra", "terraform", "docker", "kubernetes", "k8s", "cloud"],
        "monitoring": ["monitor", "observability", "logging", "metric", "alert", "trace"],
        "code_review": ["review", "lint", "format", "style", "static analysis"],
        "documentation": ["doc", "documentation", "readme", "wiki"],
        "communication": ["email", "slack", "notification", "message", "webhook"],
        "utility": ["utility", "helper", "tool", "script", "automation"],
        "agent": ["agent", "autonomous", "multi-agent", "crew", "swarm"],
        "ai": ["ai", "llm", "model", "gpt", "claude", "openai", "langchain"],
    },
}

CLONE_URL = "https://github.com/{}.git"


# ── Utilities ────────────────────────────────────────────────────────────────

def log(msg: str, level: str = "INFO"):
    ts = datetime.now(timezone.utc).strftime("%H:%M:%S")
    print(f"[{ts}] [{level}] {msg}")


def load_config() -> dict:
    if CONFIG_PATH.exists():
        with open(CONFIG_PATH, encoding="utf-8") as f:
            user_config = json.load(f)
        merged = dict(DEFAULT_CONFIG)
        merged.update(user_config)
        return merged
    return dict(DEFAULT_CONFIG)


def save_json(path: Path, data: Any):
    path.parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        json.dump(data, f, indent=2, ensure_ascii=False)


def load_json(path: Path) -> Any:
    if path.exists():
        with open(path, encoding="utf-8") as f:
            return json.load(f)
    return None


def classify_skill(name: str, description: str, domains: dict) -> List[str]:
    text = f"{name} {description}".lower()
    matched = []
    for domain, keywords in domains.items():
        for kw in keywords:
            if kw.lower() in text:
                matched.append(domain)
                break
    return matched if matched else ["uncategorized"]


def extract_frontmatter(content: str) -> Optional[dict]:
    match = re.match(r"^---\s*\n(.*?)\n---", content, re.DOTALL)
    if match:
        try:
            return yaml.safe_load(match.group(1))
        except yaml.YAMLError:
            return None
    return None


def compute_complexity(skill: dict) -> str:
    score = skill.get("body_lines", 0) + len(skill.get("scripts", [])) * 30
    if skill.get("docs"):
        score += 20
    if skill.get("templates"):
        score += 30
    if score < 50:
        return "low"
    elif score < 200:
        return "medium"
    return "high"


# ── Git clone / scan ─────────────────────────────────────────────────────────

def clone_repo(repo_name: str, dest: Path) -> bool:
    """Shallow clone a repo. Returns True if successful."""
    if dest.exists():
        log(f"    Using cached: {repo_name}")
        return True
    url = CLONE_URL.format(repo_name)
    log(f"    Cloning: {repo_name}")
    try:
        result = subprocess.run(
            ["git", "clone", "--depth", "1", "--single-branch", url, str(dest)],
            capture_output=True, text=True, timeout=120,
        )
        if result.returncode != 0:
            log(f"    Clone failed: {result.stderr.strip()[:120]}", "WARN")
            return False
        return True
    except subprocess.TimeoutExpired:
        log(f"    Clone timed out: {repo_name}", "WARN")
        return False
    except Exception as e:
        log(f"    Clone error: {e}", "WARN")
        return False


def scan_skills(repo_path: Path, root_path: str) -> List[dict]:
    """Walk a cloned repo directory and find all SKILL.md files."""
    search_root = repo_path / root_path if root_path else repo_path
    if not search_root.exists():
        return []

    found = []
    for root, dirs, files in os.walk(str(search_root)):
        if "SKILL.md" in files:
            skill_path = Path(root) / "SKILL.md"
            rel_path = skill_path.relative_to(repo_path).as_posix()
            found.append({
                "path": rel_path,
                "local_path": str(skill_path),
                "url": f"https://github.com/DUMMY/blob/main/{rel_path}",
            })
    return found


# ── Main Discovery Pipeline ──────────────────────────────────────────────────

class DiscoveryPipeline:
    def __init__(self, config: dict):
        self.config = config
        self.registry: Dict[str, dict] = {}
        self.sources: dict = {}
        self.stats = {
            "total_discovered": 0,
            "total_analyzed": 0,
            "per_domain": {},
            "per_source": {},
            "per_complexity": {"low": 0, "medium": 0, "high": 0},
            "started_at": None,
            "completed_at": None,
            "errors": [],
        }

    def run(self):
        log("=" * 60)
        log("Skill Discovery Engine - Day 1")
        log("=" * 60)

        self.stats["started_at"] = datetime.now(timezone.utc).isoformat()

        existing = load_json(REGISTRY_DIR / "skills-registry.json")
        if existing:
            self.registry = {s["id"]: s for s in existing}
            log(f"Loaded checkpoint: {len(self.registry)} skills")

        existing_sources = load_json(REGISTRY_DIR / "sources.json")
        if existing_sources:
            self.sources = existing_sources
            # Merge any new repos from config not yet in sources
            for repo in self.config["known_repos"]:
                if repo["name"] not in self.sources:
                    self.sources[repo["name"]] = {
                        "label": repo.get("label", repo["name"]),
                        "last_fetched": None,
                        "status": "pending",
                        "skills_found": 0,
                    }
        else:
            self.sources = self._init_sources()

        self._stage_discover()
        self._stage_fetch_and_classify()
        self._stage_finalize()

    def _init_sources(self) -> dict:
        sources = {}
        for repo in self.config["known_repos"]:
            sources[repo["name"]] = {
                "label": repo.get("label", repo["name"]),
                "last_fetched": None,
                "status": "pending",
                "skills_found": 0,
            }
        return sources

    def _stage_discover(self):
        log("\n-- Stage 1: Clone Repos & Discover Skills --")
        CACHE_DIR.mkdir(parents=True, exist_ok=True)

        for repo_info in self.config["known_repos"]:
            repo_name = repo_info["name"]
            root_path = repo_info["path"]
            label = repo_info.get("label", repo_name)
            log(f"  Scanning: {label}")
            self.sources[repo_name]["status"] = "in_progress"

            dest = CACHE_DIR / repo_name.replace("/", "_")
            if not clone_repo(repo_name, dest):
                self.sources[repo_name]["status"] = "clone_failed"
                continue

            skills = scan_skills(dest, root_path)
            for s in skills:
                skill_id = f"{repo_name}:{s['path']}"
                if skill_id not in self.registry:
                    self.registry[skill_id] = {
                        "id": skill_id,
                        "name": Path(s["path"]).parent.name,
                        "source": repo_name,
                        "label": label,
                        "path": s["path"],
                        "url": s["url"],
                        "local_path": s["local_path"],
                        "frontmatter": None,
                        "description": "",
                        "domains": [],
                        "complexity": "unknown",
                        "body_lines": 0,
                        "scripts": [],
                        "docs": [],
                        "templates": [],
                        "status": "discovered",
                        "discovered_at": datetime.now(timezone.utc).isoformat(),
                    }

            self.sources[repo_name]["status"] = "completed"
            self.sources[repo_name]["skills_found"] = len(skills)
            self.sources[repo_name]["last_fetched"] = datetime.now(timezone.utc).isoformat()
            log(f"    Found {len(skills)} skills")

    def _stage_fetch_and_classify(self):
        log("\n-- Stage 2: Parse & Classify Skills --")
        to_process = [
            s for s in self.registry.values()
            if s["status"] == "discovered"
        ]
        log(f"Processing {len(to_process)} skills...")

        for idx, skill in enumerate(to_process):
            skill_id = skill["id"]
            local_path = skill.get("local_path", "")

            if not local_path or not os.path.exists(local_path):
                skill["status"] = "file_missing"
                continue

            try:
                with open(local_path, "r", encoding="utf-8") as f:
                    raw = f.read()
            except Exception:
                skill["status"] = "read_failed"
                continue

            safe_name = re.sub(r'[<>:"/\\|?*]', "_", skill_id)
            raw_path = RAW_DIR / f"{safe_name}.md"
            raw_path.write_text(raw, encoding="utf-8")

            fm = extract_frontmatter(raw)
            if fm:
                skill["frontmatter"] = fm
                skill["description"] = fm.get("description", "") or ""
                name = fm.get("name", "") or Path(skill["path"]).parent.name
                skill["name"] = name
            else:
                skill["description"] = Path(skill["path"]).parent.name

            body_start = raw.find("---", raw.find("---") + 3) + 3 if "---" in raw else 0
            body = raw[body_start:].strip()
            skill["body_lines"] = len(body.splitlines())

            skill["scripts"] = self._extract_refs(raw, r'(scripts/[^\s)]+)')
            skill["docs"] = self._extract_refs(raw, r'(docs/[^\s)]+)')
            skill["templates"] = self._extract_refs(raw, r'(templates/[^\s)]+)')

            skill["domains"] = classify_skill(
                skill["name"], skill["description"], self.config["domains"],
            )
            skill["complexity"] = compute_complexity(skill)
            skill["status"] = "analyzed"
            self.stats["total_analyzed"] += 1

            if (idx + 1) % self.config["checkpoint_interval"] == 0:
                self._save_checkpoint()
                log(f"  Checkpoint: {idx + 1}/{len(to_process)} processed")

        log(f"Stage 2 complete: {self.stats['total_analyzed']} skills analyzed")

    def _extract_refs(self, content: str, pattern: str) -> List[str]:
        refs = re.findall(pattern, content)
        return list(set(refs))[:10]

    def _stage_finalize(self):
        log("\n-- Stage 3: Finalize --")
        registry_list = list(self.registry.values())
        self.stats["total_discovered"] = len(registry_list)
        self.stats["total_analyzed"] = sum(
            1 for s in registry_list if s["status"] == "analyzed"
        )

        per_domain = {}
        for s in registry_list:
            for d in s.get("domains", ["uncategorized"]):
                per_domain[d] = per_domain.get(d, 0) + 1
        self.stats["per_domain"] = per_domain

        per_source = {}
        for s in registry_list:
            src = s.get("source", "unknown")
            per_source[src] = per_source.get(src, 0) + 1
        self.stats["per_source"] = per_source

        per_complexity = {"low": 0, "medium": 0, "high": 0}
        for s in registry_list:
            c = s.get("complexity", "low")
            if c in per_complexity:
                per_complexity[c] += 1
        self.stats["per_complexity"] = per_complexity

        self.stats["completed_at"] = datetime.now(timezone.utc).isoformat()

        self._save_checkpoint()
        save_json(REGISTRY_DIR / "categories.json", {
            "domains": sorted(per_domain.keys()),
            "per_domain": per_domain,
            "per_source": per_source,
            "per_complexity": per_complexity,
        })
        save_json(REGISTRY_DIR / "sources.json", self.sources)
        save_json(REGISTRY_DIR / "stats.json", self.stats)

        log("\n" + "=" * 60)
        log("Discovery Complete!")
        log("  Total discovered: %d" % self.stats['total_discovered'])
        log("  Total analyzed:   %d" % self.stats['total_analyzed'])
        log("  Domains found:    %d" % len(per_domain))
        log("  Sources:          %d" % len(per_source))
        log("  Low/Med/High:     %d/%d/%d" % (per_complexity['low'], per_complexity['medium'], per_complexity['high']))
        log("=" * 60)

    def _save_checkpoint(self):
        save_json(REGISTRY_DIR / "skills-registry.json", list(self.registry.values()))


# ── Entry Point ──────────────────────────────────────────────────────────────

if __name__ == "__main__":
    config = load_config()
    log("Config: %d repos, checkpoint_interval=%d" % (
        len(config["known_repos"]), config["checkpoint_interval"]))
    pipeline = DiscoveryPipeline(config)
    pipeline.run()
