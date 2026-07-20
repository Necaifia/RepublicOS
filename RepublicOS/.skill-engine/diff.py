"""
diff.py — Show before/after evolution for any skill
Displays sections added, quality change, and content diff.
"""

import difflib
import json
from pathlib import Path

SCRIPT_DIR = Path(__file__).parent.resolve()
REGISTRY_DIR = SCRIPT_DIR / "registry"
EVOLVED_DIR = REGISTRY_DIR / "evolved"
RAW_DIR = REGISTRY_DIR / "raw"


def load_json(path: Path):
    if path.exists():
        with open(path, encoding="utf-8") as f:
            return json.load(f)
    return {}


def find_skill(name: str) -> dict:
    registry = load_json(REGISTRY_DIR / "skills-registry.json")
    if isinstance(registry, list):
        for s in registry:
            if s.get("name") == name:
                return s
    return {}


def get_original_content(skill: dict) -> str:
    local_path = skill.get("local_path", "")
    if local_path:
        f = Path(local_path)
        if f.exists():
            return f.read_text(encoding="utf-8")
    raw_id = skill.get("id", "")
    if raw_id:
        safe = raw_id.replace("/", "_").replace(":", "_")
        for f in RAW_DIR.rglob(f"*{safe}*"):
            if f.is_file():
                return f.read_text(encoding="utf-8")
    return ""


def get_evolved_content(skill: dict) -> str:
    skill_id = skill.get("id", skill.get("name", ""))
    safe = skill_id.replace("/", "_").replace(":", "_").replace("\\", "_")
    # Try direct name first
    path = EVOLVED_DIR / f"{skill.get('name', '')}.md"
    if path.exists():
        return path.read_text(encoding="utf-8")
    # Try id-based name
    path = EVOLVED_DIR / f"{safe}.md"
    if path.exists():
        return path.read_text(encoding="utf-8")
    # Try glob
    matches = list(EVOLVED_DIR.glob(f"*{skill.get('name', '')}*"))
    if matches:
        return matches[0].read_text(encoding="utf-8")
    return ""


def color_diff(diff_lines: list) -> str:
    out = []
    for line in diff_lines:
        if line.startswith("+++") or line.startswith("---"):
            out.append(f"\033[1;36m{line}\033[0m")
        elif line.startswith("@@"):
            out.append(f"\033[1;33m{line}\033[0m")
        elif line.startswith("+"):
            out.append(f"\033[1;32m{line}\033[0m")
        elif line.startswith("-"):
            out.append(f"\033[1;31m{line}\033[0m")
        else:
            out.append(line)
    return "\n".join(out)


def show_diff(name: str, html: bool = False):
    skill = find_skill(name)
    if not skill:
        print(f"Skill '{name}' not found in registry.")
        return

    original = get_original_content(skill)
    evolved = get_evolved_content(skill)

    if not original and not evolved:
        print(f"No content found for '{name}'.")
        return

    if not evolved:
        print(f"Evolved version not found for '{name}'. Run evolution first.")
        return

    if not original:
        print(f"Original version not found for '{name}'.")

    quality = load_json(REGISTRY_DIR / "quality.json")
    scores = {}
    if isinstance(quality, dict):
        skills_list = quality.get("skills", [])
        if isinstance(skills_list, list):
            for s in skills_list:
                if isinstance(s, dict):
                    scores[s.get("name", "")] = s.get("quality_score", 0)

    old_score = scores.get(name, "?")
    quality_evolved = load_json(REGISTRY_DIR / "evolved" / f"{name}-quality.json")
    new_score = quality_evolved.get("quality_score", old_score) if isinstance(quality_evolved, dict) else old_score

    old_sections = len([l for l in original.split("\n") if l.startswith("#") or l.startswith("##") or l.startswith("###")]) if original else 0
    new_sections = len([l for l in evolved.split("\n") if l.startswith("#") or l.startswith("##") or l.startswith("###")])

    old_lines = len(original.split("\n")) if original else 0
    new_lines = len(evolved.split("\n"))

    if html:
        diff = list(difflib.unified_diff(
            original.split("\n") if original else [],
            evolved.split("\n"),
            fromfile=f"original/{name}.md",
            tofile=f"evolved/{name}.md",
            lineterm="",
        ))
        diff_html = "\n".join(diff)
        diff_html = diff_html.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;")
        page = f"""<!DOCTYPE html>
<html lang="en">
<head><meta charset="UTF-8">
<title>Diff: {name}</title>
<style>
body{{font-family:SFMono-Regular,Consolas,monospace;background:#0a0a0f;color:#e2e8f0;padding:2rem;font-size:0.85rem;line-height:1.5}}
h1{{color:#f8fafc;border-bottom:1px solid #1e293b;padding-bottom:0.5rem}}
.meta{{display:grid;grid-template-columns:repeat(auto-fit,minmax(140px,1fr));gap:0.75rem;margin:1rem 0;padding:1rem;background:#13131f;border-radius:8px;border:1px solid #1e293b}}
.meta div{{text-align:center}}
.meta .v{{font-size:1.5rem;font-weight:700}}
.meta .l{{font-size:0.75rem;color:#64748b}}
pre{{background:#13131f;padding:1rem;border-radius:8px;border:1px solid #1e293b;overflow-x:auto}}
.add{{color:#22c55e}}.rem{{color:#ef4444}}.hdr{{color:#3b82f6}}.pos{{color:#eab308}}
</style>
</head>
<body>
<h1>Diff: {name}</h1>
<div class="meta">
<div><div class="v" style="color:#22c55e">{new_lines - old_lines}</div><div class="l">Lines added</div></div>
<div><div class="v" style="color:#22c55e">{new_sections - old_sections}</div><div class="l">Sections added</div></div>
<div><div class="v">{old_score} → {new_score}</div><div class="l">Quality score</div></div>
<div><div class="v">{old_lines} → {new_lines}</div><div class="l">Total lines</div></div>
<div><div class="v">{old_sections} → {new_sections}</div><div class="l">Sections</div></div>
</div>
<pre>
"""
        for line in diff:
            css = ""
            if line.startswith("+++") or line.startswith("---"):
                css = ' class="hdr"'
            elif line.startswith("@@"):
                css = ' class="pos"'
            elif line.startswith("+"):
                css = ' class="add"'
            elif line.startswith("-"):
                css = ' class="rem"'
            page += f'<div{css}>{line}</div>\n'
        page += "</pre></body></html>"
        out_path = EVOLVED_DIR / f"{name}-diff.html"
        out_path.write_text(page, encoding="utf-8")
        print(f"Diff HTML: {out_path}")
    else:
        print(f"\n{'='*60}")
        print(f"  Diff: {name}")
        print(f"  Score: {old_score} -> {new_score}")
        print(f"  Lines: {old_lines} -> {new_lines} ({new_lines - old_lines:+d})")
        print(f"  Sections: {old_sections} -> {new_sections} ({new_sections - old_sections:+d})")
        print(f"{'='*60}")

        if original:
            diff = list(difflib.unified_diff(
                original.split("\n"), evolved.split("\n"),
                fromfile=f"original/{name}.md", tofile=f"evolved/{name}.md",
                lineterm="",
            ))
            print(color_diff(diff))
        else:
            print(f"\nEvolved content ({new_lines} lines):")
            print(evolved)


def list_all():
    registry = load_json(REGISTRY_DIR / "skills-registry.json")
    if not isinstance(registry, list):
        return
    quality = load_json(REGISTRY_DIR / "quality.json")
    scores = {}
    if isinstance(quality, dict):
        for s in quality.get("skills", []):
            if isinstance(s, dict):
                scores[s.get("name", "")] = s.get("quality_score", 0)

    evolved_ids = set()
    for s in registry:
        if get_evolved_content(s):
            evolved_ids.add(s.get("name", ""))

    print(f"{'Skill':35s} {'Old':>5s} {'Domain':20s} {'Evolved':>8s}")
    print("-" * 72)
    for s in sorted(registry, key=lambda x: x.get("name", "")):
        name = s.get("name", "")
        domain = (s.get("domains", []) or ["?"])[0][:18]
        score = scores.get(name, 0)
        has_evolved = name in evolved_ids
        print(f"  {name:33s} {score:3d}  {domain:20s} {'[evolved]' if has_evolved else ''}")


def main():
    import argparse
    parser = argparse.ArgumentParser(description="Skill diff tool")
    parser.add_argument("name", nargs="?", help="Skill name to diff")
    parser.add_argument("--html", action="store_true", help="Generate HTML diff page")
    parser.add_argument("--list", action="store_true", help="List all skills with status")
    parser.add_argument("--all", action="store_true", help="Generate diffs for all skills")

    args = parser.parse_args()

    if args.list:
        list_all()
        return

    if args.all:
        registry = load_json(REGISTRY_DIR / "skills-registry.json")
        if isinstance(registry, list):
            for s in registry:
                name = s.get("name", "")
                if name:
                    show_diff(name, html=args.html)
        return

    if args.name:
        show_diff(args.name, html=args.html)
    else:
        parser.print_help()


if __name__ == "__main__":
    main()
