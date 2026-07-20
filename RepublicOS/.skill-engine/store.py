"""
store.py — Integration + Skill Store (Day 6)
Part of RepublicOS Skill Evolution Pipeline

Exports evolved skills to standard agent formats:
- Cline (.cline/skills/)
- Claude Code (.claude/skills/)
- OpenCode (.opencode/skills/)
- Generic agent-agnostic format
"""

import json
import os
import re
import shutil
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional

SCRIPT_DIR = Path(__file__).parent.resolve()
REGISTRY_DIR = SCRIPT_DIR / "registry"
EVOLVED_DIR = REGISTRY_DIR / "evolved"
STORE_DIR = SCRIPT_DIR / "store"

AGENT_FORMATS = {
    "cline": {
        "path": "agent-skills/cline/",
        "label": "Cline Skills",
    },
    "claude": {
        "path": "agent-skills/claude/",
        "label": "Claude Code Skills",
    },
    "opencode": {
        "path": "agent-skills/opencode/",
        "label": "OpenCode Skills",
    },
    "generic": {
        "path": "agent-skills/generic/",
        "label": "Generic Agent Skills",
    },
}

TOP_DOMAINS = [
    "deploy", "test", "security", "monitoring", "database",
    "api", "infrastructure", "communication", "code_review",
    "documentation", "utility",
]


def log(msg: str):
    ts = datetime.now(timezone.utc).strftime("%H:%M:%S")
    print(f"[{ts}] [STORE] {msg}")


def load_json(path: Path) -> Any:
    if path.exists():
        with open(path, encoding="utf-8") as f:
            return json.load(f)
    return {}


def save_json(path: Path, data: Any):
    path.parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        json.dump(data, f, indent=2, ensure_ascii=False)


def make_skill_name(skill_id: str) -> str:
    parts = skill_id.replace(":", "/").split("/")
    name = parts[-1].replace("SKILL.md", "").strip("/")
    if not name:
        name = parts[-2] if len(parts) >= 2 else "unnamed"
    return re.sub(r"[^a-z0-9-]", "", name.lower().replace("_", "-")) or "unnamed"


def get_domain_dir(domains: List[str]) -> str:
    for d in TOP_DOMAINS:
        if d in domains:
            return d
    return "other"


def export_skill(skill_content: str, skill_name: str, skill_id: str,
                 domains: List[str], output_root: Path):
    """Export a single skill to all agent formats."""
    domain_dir = get_domain_dir(domains)
    for agent_key, fmt in AGENT_FORMATS.items():
        skill_dir = output_root / fmt["path"] / domain_dir / skill_name
        skill_dir.mkdir(parents=True, exist_ok=True)
        (skill_dir / "SKILL.md").write_text(skill_content, encoding="utf-8")


def build_catalog(output_root: Path, catalog: List[dict]):
    """Generate a catalog/index of all exported skills."""
    catalog_path = output_root / "agent-skills" / "catalog.json"
    save_json(catalog_path, {
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "total_skills": len(catalog),
        "agents": list(AGENT_FORMATS.keys()),
        "skills": catalog,
    })

    # Generate README
    lines = [
        "# RepublicOS Skill Store",
        "",
        f"*Generated: {datetime.now(timezone.utc).isoformat()}*",
        "",
        f"**{len(catalog)} evolved skills** available for AI coding agents.",
        "",
        "## Supported Agents",
        "",
    ]
    for key, fmt in AGENT_FORMATS.items():
        lines.append(f"- **{fmt['label']}**: `{fmt['path']}`")
    lines.extend([
        "",
        "## Skill Catalog",
        "",
        "| Skill | Domain | Complexity | Description |",
        "|-------|--------|------------|-------------|",
    ])

    for entry in sorted(catalog, key=lambda x: (x["domain"], x["name"])):
        desc = entry["description"][:80].replace("\n", " ").replace("|", "/")
        lines.append(
            f"| {entry['name']} | {entry['domain']} | "
            f"{entry['complexity']} | {desc} |"
        )

    readme_path = output_root / "agent-skills" / "README.md"
    readme_path.write_text("\n".join(lines) + "\n", encoding="utf-8")

    return catalog_path


def export_all():
    log("=" * 60)
    log("Skill Store Export - Day 6")
    log("=" * 60)

    registry = load_json(REGISTRY_DIR / "skills-registry.json")
    evolution_log = load_json(EVOLVED_DIR / "evolution-log.json")

    if not registry:
        log("No skills found. Run discovery.py first.")
        return

    evolved_log_entries = {}
    if evolution_log:
        for entry in evolution_log.get("evolution_log", []):
            evolved_log_entries[entry["skill_id"]] = entry

    log(f"Exporting {len(registry)} skills to {len(AGENT_FORMATS)} agent formats")

    shutil.rmtree(STORE_DIR / "agent-skills", ignore_errors=True)

    catalog = []
    export_count = 0

    for skill in registry:
        skill_id = skill["id"]
        domains = skill.get("domains", ["other"])
        name = skill.get("name", "unnamed")
        complexity = skill.get("complexity", "unknown")

        # Get the most recent version (evolved > raw)
        safe_name = re.sub(r'[<>:"/\\|?*]', "_", skill_id)
        evolved_path = EVOLVED_DIR / f"{safe_name}.md"

        if evolved_path.exists():
            content = evolved_path.read_text(encoding="utf-8")
            source = "evolved"
        else:
            log(f"  No evolved version for {skill_id}, skipping", "WARN")
            continue

        skill_name = make_skill_name(skill_id)
        export_skill(content, skill_name, skill_id, domains, STORE_DIR)

        evo = evolved_log_entries.get(skill_id, {})
        description = evo.get("new_description", skill.get("description", ""))
        old_description = evo.get("old_description", "")

        catalog.append({
            "id": skill_id,
            "name": skill_name,
            "domain": get_domain_dir(domains),
            "domains": domains,
            "complexity": complexity,
            "description": description[:200],
            "source": source,
            "agent_paths": {
                key: f"{fmt['path']}{get_domain_dir(domains)}/{skill_name}/"
                for key, fmt in AGENT_FORMATS.items()
            },
        })
        export_count += 1

    # Write catalog
    catalog_path = build_catalog(STORE_DIR, catalog)

    log(f"\n{'=' * 60}")
    log("Export complete!")
    log(f"  Skills exported: {export_count}")
    log(f"  Agent formats:   {len(AGENT_FORMATS)}")
    log(f"  Catalog:         {catalog_path}")
    log(f"  Store root:      {STORE_DIR / 'agent-skills/'}")
    log("=" * 60)


if __name__ == "__main__":
    export_all()
