"""
validate.py — Validation for evolved skills
Checks syntax, structure, frontmatter, section completeness, and quality.
"""

import json
import re
from pathlib import Path

SCRIPT_DIR = Path(__file__).parent.resolve()
REGISTRY_DIR = SCRIPT_DIR / "registry"
EVOLVED_DIR = REGISTRY_DIR / "evolved"

REQUIRED_FRONTMATTER = ["name", "description"]
RECOMMENDED_FRONTMATTER = ["version", "tags", "metadata"]
STANDARD_SECTIONS = ["Prerequisites", "Error Handling", "Verification", "Rollback"]
OPTIONAL_SECTIONS = ["Security", "Configuration", "Workflow", "Usage", "Examples"]


def load_json(path: Path):
    if path.exists():
        with open(path, encoding="utf-8") as f:
            return json.load(f)
    return {}


def log(msg: str):
    print(f"[VALIDATE] {msg}")


def parse_frontmatter(content: str) -> tuple:
    fm = {}
    rest = content
    if content.startswith("---"):
        parts = content.split("---", 2)
        if len(parts) >= 3:
            try:
                import yaml
                fm = yaml.safe_load(parts[1]) or {}
            except Exception:
                pass
            rest = parts[2]
    return fm, rest


def check_frontmatter(fm: dict) -> list:
    issues = []
    for field in REQUIRED_FRONTMATTER:
        if field not in fm or not fm[field]:
            issues.append(f"Missing required frontmatter: {field}")
    for field in RECOMMENDED_FRONTMATTER:
        if field not in fm:
            issues.append(f"Missing recommended frontmatter: {field}")
    desc = fm.get("description", "")
    if len(desc) < 20:
        issues.append(f"Description too short ({len(desc)} chars, min 20)")
    return issues


def check_sections(body: str) -> list:
    issues = []
    found = set()
    for line in body.split("\n"):
        line = line.strip()
        if line.startswith("## ") or line.startswith("# "):
            sec = line.lstrip("#").strip()
            found.add(sec)

    for section in STANDARD_SECTIONS:
        if section not in found:
            issues.append(f"Missing standard section: {section}")

    for section in OPTIONAL_SECTIONS:
        if section not in found:
            issues.append(f"Missing optional section: {section} (recommended)")

    return issues


def check_content(body: str) -> list:
    issues = []
    lines = body.split("\n")
    if len(lines) < 30:
        issues.append(f"Body too short ({len(lines)} lines, min 30)")

    code_blocks = len(re.findall(r"```", body)) // 2
    if code_blocks == 0:
        issues.append("No code blocks found (recommended for actionable skills)")

    has_bullets = any(line.strip().startswith("- ") for line in lines)
    if not has_bullets:
        issues.append("No bullet points found (recommended for readability)")

    has_tables = "|" in body and "---" in body
    if not has_tables:
        issues.append("No markdown tables found (useful for structured info)")

    return issues


def check_links(body: str) -> list:
    issues = []
    urls = re.findall(r"https?://[^\s)]+", body)
    placeholder_urls = [u for u in urls if "DUMMY" in u or "example" in u or "your-" in u]
    if placeholder_urls:
        issues.append(f"Placeholder URLs found: {', '.join(placeholder_urls[:3])}")
    return issues


def validate_skill(name: str, content: str) -> dict:
    fm, body = parse_frontmatter(content)
    issues = []
    warnings = []

    fm_issues = check_frontmatter(fm)
    for i in fm_issues:
        issues.append(i)

    sec_issues = check_sections(body)
    for i in sec_issues:
        if any(s in i for s in STANDARD_SECTIONS):
            issues.append(i)
        else:
            warnings.append(i)

    content_issues = check_content(body)
    for i in content_issues:
        warnings.append(i)

    link_issues = check_links(body)
    for i in link_issues:
        warnings.append(i)

    lines = len(content.split("\n"))
    sections = len(re.findall(r"^##?\s", content, re.MULTILINE))
    has_frontmatter = bool(fm)

    return {
        "name": name,
        "lines": lines,
        "sections": sections,
        "has_frontmatter": has_frontmatter,
        "frontmatter_fields": len(fm),
        "code_blocks": len(re.findall(r"```", content)) // 2,
        "issues": issues,
        "warnings": warnings,
        "valid": len(issues) == 0,
        "score": max(0, 100 - len(issues) * 15 - len(warnings) * 5),
    }


def validate_all():
    evolved_files = sorted(EVOLVED_DIR.glob("*.md"))
    if not evolved_files:
        log("No evolved files found")
        return

    results = []
    total_issues = 0
    total_warnings = 0
    valid_count = 0

    log(f"Validating {len(evolved_files)} skills...")

    for f in evolved_files:
        content = f.read_text(encoding="utf-8")
        name = f.stem.replace("-diff", "")
        result = validate_skill(name, content)
        results.append(result)
        total_issues += len(result["issues"])
        total_warnings += len(result["warnings"])
        if result["valid"]:
            valid_count += 1

        if result["issues"]:
            log(f"Issues in {name}:")
            for i in result["issues"]:
                log(f"  [x] {i}")
        if result["warnings"]:
            for w in result["warnings"]:
                pass

    avg_score = sum(r["score"] for r in results) / len(results) if results else 0

    print(f"\n{'='*50}")
    print(f"  Validation Results")
    print(f"{'='*50}")
    print(f"  Total skills:    {len(results)}")
    print(f"  Valid:           {valid_count} ({valid_count*100//len(results)}%)")
    print(f"  With issues:     {len(results) - valid_count}")
    print(f"  Total issues:    {total_issues}")
    print(f"  Total warnings:  {total_warnings}")
    print(f"  Avg quality:     {avg_score:.0f}/100")
    print(f"{'='*50}")

    if results:
        print(f"\n{'Name':<30} {'Score':>6} {'Lines':>6} {'Sections':>9} {'Issues':>7}")
        print("-" * 60)
        for r in sorted(results, key=lambda x: -x["score"])[:10]:
            mark = "+" if r["valid"] else "x"
            print(f"  {mark} {r['name']:<28} {r['score']:>3}  {r['lines']:>5}  {r['sections']:>3}  {len(r['issues']):>3}i/{len(r['warnings'])}w")

    save_json(EVOLVED_DIR / "validation-report.json", {
        "date": __import__("datetime").datetime.now().isoformat(),
        "total": len(results),
        "valid": valid_count,
        "total_issues": total_issues,
        "total_warnings": total_warnings,
        "avg_score": round(avg_score, 1),
        "results": results,
    })
    log("Report saved to registry/evolved/validation-report.json")

    return valid_count == len(results)


def validate_one(name: str):
    for f in EVOLVED_DIR.glob(f"*{name}*"):
        if f.suffix == ".md" and "diff" not in f.stem:
            content = f.read_text(encoding="utf-8")
            result = validate_skill(name, content)
            print(f"\nValidation: {name}")
            print(f"  Score: {result['score']}/100")
            print(f"  Lines: {result['lines']}, Sections: {result['sections']}")
            print(f"  Frontmatter: {'yes' if result['has_frontmatter'] else 'no'} ({result['frontmatter_fields']} fields)")
            print(f"  Code blocks: {result['code_blocks']}")
            if result["issues"]:
                print(f"  Issues ({len(result['issues'])}):")
                for i in result["issues"]:
                    print(f"    [x] {i}")
            if result["warnings"]:
                print(f"  Warnings ({len(result['warnings'])}):")
                for w in result["warnings"]:
                    print(f"    [!] {w}")
            if result["valid"]:
                print(f"  Result: PASS")
            else:
                print(f"  Result: FAIL")
            return

    print(f"Skill '{name}' not found")


def main():
    import argparse
    parser = argparse.ArgumentParser(description="Validate evolved skills")
    parser.add_argument("name", nargs="?", help="Validate a specific skill")
    parser.add_argument("--all", "-a", action="store_true", help="Validate all skills")
    args = parser.parse_args()

    if args.name:
        validate_one(args.name)
    elif args.all:
        validate_all()
    else:
        validate_all()


if __name__ == "__main__":
    main()
