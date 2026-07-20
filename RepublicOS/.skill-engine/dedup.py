"""
dedup.py — Skill deduplication and merging
Detects duplicate skills across sources and merges them intelligently.
"""

import json
from pathlib import Path

SCRIPT_DIR = Path(__file__).parent.resolve()
REGISTRY_DIR = SCRIPT_DIR / "registry"
EVOLVED_DIR = REGISTRY_DIR / "evolved"


def load_json(path: Path):
    if path.exists():
        with open(path, encoding="utf-8") as f:
            return json.load(f)
    return {}


def save_json(path: Path, data):
    path.parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        json.dump(data, f, indent=2, ensure_ascii=False)


def log(msg: str):
    print(f"[DEDUP] {msg}")


def find_duplicates(registry: list) -> dict:
    by_name = {}
    for i, s in enumerate(registry):
        name = s.get("name", "")
        if name not in by_name:
            by_name[name] = []
        by_name[name].append((i, s))

    return {name: items for name, items in by_name.items() if len(items) > 1}


def merge_skill(entries: list) -> dict:
    primary = entries[0][1].copy()
    duplicates = [e[1] for e in entries[1:]]

    all_domains = set(primary.get("domains", []))
    all_tags = set()
    all_sources = [primary.get("source", "")]
    best_desc = primary.get("description", "")
    best_body = primary.get("body_lines", 0)
    best_complexity = primary.get("complexity", "")

    for dup in duplicates:
        for d in dup.get("domains", []):
            all_domains.add(d)
        src = dup.get("source", "")
        if src and src not in all_sources:
            all_sources.append(src)
        desc = dup.get("description", "")
        if len(desc) > len(best_desc):
            best_desc = desc
        if dup.get("body_lines", 0) > best_body:
            best_body = dup.get("body_lines", 0)
        if dup.get("complexity", "") and not best_complexity:
            best_complexity = dup.get("complexity", "")

    primary["domains"] = sorted(all_domains)
    primary["description"] = best_desc
    primary["body_lines"] = best_body
    primary["complexity"] = best_complexity or "medium"
    primary["sources"] = all_sources
    primary["dedup"] = True
    primary["dedup_merged"] = [d.get("source", "") for d in duplicates]
    primary["dedup_count"] = len(entries)

    return primary


def deduplicate(dry_run: bool = False, interactive: bool = False):
    registry = load_json(REGISTRY_DIR / "skills-registry.json")
    if not isinstance(registry, list):
        log("No registry found")
        return

    dups = find_duplicates(registry)
    if not dups:
        log("No duplicates found")
        return

    log(f"Found {len(dups)} duplicate groups:")
    for name, entries in sorted(dups.items()):
        sources = [e[1].get("source", "?") for e in entries]
        log(f"  {name}: {', '.join(sources)}")

    if dry_run:
        log("Dry run - no changes made")
        return

    merged = []
    skip_indices = set()

    for name, entries in sorted(dups.items()):
        if interactive:
            print(f"\nDuplicate: {name}")
            for i, (idx, s) in enumerate(entries):
                src = s.get("source", "?")
                dom = ", ".join(s.get("domains", []))
                desc = (s.get("description", "") or "")[:80]
                print(f"  [{i}] {src} | domains: {dom}")
                print(f"       {desc}...")
            choice = input(f"Merge all into one? (Y/n): ").strip().lower()
            if choice == "n":
                log(f"Skipping {name}")
                for idx, _ in entries:
                    merged.append(registry[idx])
                continue

        merged_skill = merge_skill(entries)
        merged.append(merged_skill)
        for idx, _ in entries:
            skip_indices.add(idx)
        log(f"Merged {name}: {len(entries)} sources -> 1")

    for i, s in enumerate(registry):
        if i not in skip_indices:
            merged.append(s)

    log(f"Before: {len(registry)} skills")
    log(f"After:  {len(merged)} skills")
    log(f"Removed: {len(registry) - len(merged)} duplicates")

    save_json(REGISTRY_DIR / "skills-registry.json", merged)
    log("Updated skills-registry.json")

    # Also clean up evolved files
    removed_ids = set()
    for name, entries in dups.items():
        for idx, s in entries[1:]:
            sid = s.get("id", "")
            if sid:
                removed_ids.add(sid)

    cleaned = 0
    for sid in removed_ids:
        safe = sid.replace("/", "_").replace(":", "_")
        for f in EVOLVED_DIR.glob(f"{safe}*"):
            f.unlink()
            cleaned += 1
    log(f"Cleaned up {cleaned} duplicate evolved files")


def main():
    import argparse
    parser = argparse.ArgumentParser(description="Skill deduplication")
    parser.add_argument("--dry-run", "-n", action="store_true", help="Show duplicates without merging")
    parser.add_argument("--interactive", "-i", action="store_true", help="Confirm each merge")
    args = parser.parse_args()
    deduplicate(dry_run=args.dry_run, interactive=args.interactive)


if __name__ == "__main__":
    main()
