"""
mcp-server.py — MCP Server for RepublicOS Skill Engine
Exposes skill discovery, analysis, evolution, and search as MCP tools
for any MCP-compatible agent (Claude Code, Cline, OpenCode, etc.).
"""

import json
import os
import subprocess
import sys
import traceback
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional

SCRIPT_DIR = Path(__file__).parent.resolve()
REGISTRY_DIR = SCRIPT_DIR / "registry"
EVOLVED_DIR = REGISTRY_DIR / "evolved"
STORE_DIR = SCRIPT_DIR / "store" / "agent-skills"

STAGES_ORDER = ["discovery", "analyzer", "research", "evolution", "store"]
STAGE_SCRIPTS = {s: str(SCRIPT_DIR / f"{s}.py") for s in STAGES_ORDER}
PIPELINE_SCRIPT = str(SCRIPT_DIR / "pipeline.py")

PROTOCOL_VERSION = "2024-11-05"
SERVER_NAME = "RepublicOS-SkillEngine"
SERVER_VERSION = "1.0.0"


def load_json(path: Path):
    if path.exists():
        with open(path, encoding="utf-8") as f:
            return json.load(f)
    return {}


def log(msg: str):
    ts = datetime.now(timezone.utc).strftime("%H:%M:%S")
    print(f"[{ts}] [MCP] {msg}", file=sys.stderr, flush=True)


def send_response(id: Optional[int], result: Any = None, error: Optional[dict] = None):
    msg: Dict[str, Any] = {"jsonrpc": "2.0"}
    if id is not None:
        msg["id"] = id
    if error:
        msg["error"] = error
    else:
        msg["result"] = result
    line = json.dumps(msg, ensure_ascii=True)
    sys.stdout.write(line + "\n")
    sys.stdout.flush()


def send_event(method: str, params: dict):
    msg = {"jsonrpc": "2.0", "method": method, "params": params}
    line = json.dumps(msg, ensure_ascii=False)
    sys.stdout.write(line + "\n")
    sys.stdout.flush()


def format_tool_result(text: str, is_error: bool = False) -> dict:
    content = [{"type": "text", "text": text}]
    if is_error:
        return {"content": content, "isError": True}
    return {"content": content}


def run_script(script: str, timeout: int = 300) -> Dict[str, Any]:
    try:
        result = subprocess.run(
            [sys.executable, script],
            capture_output=True, text=True, timeout=timeout,
        )
        return {
            "success": result.returncode == 0,
            "returncode": result.returncode,
            "stdout": result.stdout[-5000:] if result.stdout else "",
            "stderr": result.stderr[-2000:] if result.stderr else "",
        }
    except subprocess.TimeoutExpired:
        return {"success": False, "returncode": -1, "stdout": "", "stderr": "Timeout"}
    except Exception as e:
        return {"success": False, "returncode": -1, "stdout": "", "stderr": str(e)}


# ---- Tools ----

def tool_list_tools() -> list:
    return [
        {
            "name": "discover_skills",
            "description": "Discover AI agent skills from GitHub repos",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "force": {
                        "type": "boolean",
                        "description": "Force full rediscovery (ignore cache)",
                        "default": False,
                    }
                },
            },
        },
        {
            "name": "analyze_skills",
            "description": "Analyze discovered skills for patterns, gaps, and quality scores",
            "inputSchema": {
                "type": "object",
                "properties": {},
            },
        },
        {
            "name": "research_domain",
            "description": "Research best practices for a specific domain",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "domain": {
                        "type": "string",
                        "description": "Domain to research (test, deploy, security, monitoring, etc.)",
                    }
                },
                "required": ["domain"],
            },
        },
        {
            "name": "evolve_skills",
            "description": "Evolve all discovered skills with domain research",
            "inputSchema": {
                "type": "object",
                "properties": {},
            },
        },
        {
            "name": "export_skills",
            "description": "Export evolved skills to agent-specific formats",
            "inputSchema": {
                "type": "object",
                "properties": {},
            },
        },
        {
            "name": "run_pipeline",
            "description": "Run the full skill evolution pipeline (all stages)",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "stages": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "Stages to run (default: all)",
                    },
                    "timeout": {
                        "type": "number",
                        "description": "Timeout per stage in seconds (default: 600)",
                        "default": 600,
                    },
                },
            },
        },
        {
            "name": "search_skills",
            "description": "Search the skill registry for skills matching criteria",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Search term to match against skill name or description",
                    },
                    "domain": {
                        "type": "string",
                        "description": "Filter by domain",
                    },
                    "min_quality": {
                        "type": "number",
                        "description": "Minimum quality score (0-100)",
                    },
                    "limit": {
                        "type": "number",
                        "description": "Max results (default: 20)",
                        "default": 20,
                    },
                },
            },
        },
        {
            "name": "get_skill",
            "description": "Get full details of a specific skill by name",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Skill name (from the registry)",
                    },
                    "version": {
                        "type": "string",
                        "description": "Version: 'original' or 'evolved' (default: evolved)",
                        "default": "evolved",
                    },
                },
                "required": ["name"],
            },
        },
        {
            "name": "get_stats",
            "description": "Get registry statistics (counts, domains, quality, grades)",
            "inputSchema": {"type": "object", "properties": {}},
        },
        {
            "name": "get_status",
            "description": "Get pipeline and scheduler status",
            "inputSchema": {"type": "object", "properties": {}},
        },
        {
            "name": "list_stages",
            "description": "List available pipeline stages and their scripts",
            "inputSchema": {"type": "object", "properties": {}},
        },
    ]


def handle_discover_skills(params: dict) -> dict:
    force = params.get("force", False)
    extra_args = []
    if force:
        extra_path = SCRIPT_DIR / ".full-rerun"
        extra_path.touch()

    result = run_script(STAGE_SCRIPTS["discovery"])
    if force:
        extra_path.unlink(missing_ok=True)

    registry = load_json(REGISTRY_DIR / "skills-registry.json")
    count = len(registry) if registry else 0

    text = f"Discovery {'succeeded' if result['success'] else 'failed'}\n"
    text += f"Skills found: {count}\n"
    if result["stdout"]:
        text += f"\n{result['stdout'][-2000:]}"
    return format_tool_result(text, is_error=not result["success"])


def handle_analyze_skills(params: dict) -> dict:
    result = run_script(STAGE_SCRIPTS["analyzer"])
    quality = load_json(REGISTRY_DIR / "quality.json")
    clusters = load_json(REGISTRY_DIR / "clusters.json")

    text = f"Analysis {'succeeded' if result['success'] else 'failed'}\n"
    if quality:
        text += f"Avg quality score: {quality.get('avg_score', '?')}\n"
        grades = quality.get("grade_distribution", {})
        text += f"Grades: A={grades.get('A', 0)} B={grades.get('B', 0)} C={grades.get('C', 0)} D={grades.get('D', 0)}\n"
    if clusters:
        domains = list(clusters.keys())
        text += f"Domains: {len(domains)} ({', '.join(domains[:8])}{'...' if len(domains) > 8 else ''})"
    if result["stdout"]:
        text += f"\n\n{result['stdout'][-2000:]}"
    return format_tool_result(text, is_error=not result["success"])


def handle_research_domain(params: dict) -> dict:
    domain = params.get("domain", "").lower().strip()
    if not domain:
        return format_tool_result("Error: 'domain' parameter is required", is_error=True)

    research_file = REGISTRY_DIR / "research" / f"{domain}.md"
    if research_file.exists():
        text = f"Research for domain '{domain}' (cached):\n\n"
        text += research_file.read_text(encoding="utf-8")
        return format_tool_result(text)

    research_script = SCRIPT_DIR / "research.py"
    result = run_script(research_script)

    if research_file.exists():
        text = f"Research for domain '{domain}':\n\n"
        text += research_file.read_text(encoding="utf-8")
    else:
        domains_dir = REGISTRY_DIR / "research"
        available = [f.stem for f in domains_dir.glob("*.md") if f.stem != "README"] if domains_dir.exists() else []
        text = f"Research for '{domain}' not available.\nAvailable domains: {', '.join(available) or 'none'}\n"
        text += f"\nResearch script output:\n{result['stdout'][-1000:]}"
    return format_tool_result(text, is_error=not result["success"])


def handle_evolve_skills(params: dict) -> dict:
    result = run_script(STAGE_SCRIPTS["evolution"])
    log_data = load_json(REGISTRY_DIR / "evolved" / "evolution-log.json")

    text = f"Evolution {'succeeded' if result['success'] else 'failed'}\n"
    if log_data:
        text += f"Total evolved: {log_data.get('total_evolved', 0)}\n"
        text += f"Domains: {log_data.get('domains_count', 0)}\n"
        text += f"Total sections added: {log_data.get('total_sections_added', 0)}"
    if result["stdout"]:
        text += f"\n\n{result['stdout'][-2000:]}"
    return format_tool_result(text, is_error=not result["success"])


def handle_export_skills(params: dict) -> dict:
    result = run_script(STAGE_SCRIPTS["store"])
    agents = []
    store_path = SCRIPT_DIR / "store" / "agent-skills"
    if store_path.exists():
        agents = [d.name for d in store_path.iterdir() if d.is_dir()]

    catalog_path = store_path / "catalog.json"
    catalog = load_json(catalog_path) if catalog_path.exists() else {}

    text = f"Export {'succeeded' if result['success'] else 'failed'}\n"
    text += f"Agent formats: {', '.join(agents) if agents else 'none'}\n"
    if isinstance(catalog, dict):
        text += f"Total exports: {catalog.get('total_exports', 0)}"
    if result["stdout"]:
        text += f"\n\n{result['stdout'][-2000:]}"
    return format_tool_result(text, is_error=not result["success"])


def handle_run_pipeline(params: dict) -> dict:
    stages = params.get("stages")
    timeout = params.get("timeout", 600)
    args = [sys.executable, PIPELINE_SCRIPT]
    if stages:
        args.extend(["--stages"] + stages)
    if timeout:
        args.extend(["--timeout", str(timeout)])
    try:
        result = subprocess.run(args, capture_output=True, text=True, timeout=timeout + 60)
        text = f"Pipeline {'succeeded' if result.returncode == 0 else 'failed'}\n\n"
        text += result.stdout[-5000:] if result.stdout else ""
        if result.stderr:
            text += f"\nStderr:\n{result.stderr[-2000:]}"
        return format_tool_result(text, is_error=result.returncode != 0)
    except subprocess.TimeoutExpired:
        return format_tool_result("Pipeline timed out", is_error=True)


def handle_search_skills(params: dict) -> dict:
    query = params.get("query", "").lower()
    domain = params.get("domain", "").lower()
    min_quality = params.get("min_quality", 0)
    limit = params.get("limit", 20)

    registry = load_json(REGISTRY_DIR / "skills-registry.json")
    quality = load_json(REGISTRY_DIR / "quality.json")
    scores = {s.get("name"): s for s in quality.get("skills", [])} if isinstance(quality, dict) else {}

    if not isinstance(registry, list) or not registry:
        return format_tool_result("No skills in registry. Run discovery first.", is_error=True)

    results = []
    for skill in registry:
        name = skill.get("name", "")
        desc = skill.get("description", "").lower()
        tags_list = skill.get("tags", [])
        if isinstance(tags_list, str):
            tags_list = [tags_list]
        tags = " ".join(tags_list).lower()
        domains_list = skill.get("domains", [])
        if isinstance(domains_list, str):
            domains_list = [domains_list]
        domains = " ".join(domains_list).lower()
        haystack = f"{name} {desc} {tags} {domains}"

        if query and query not in haystack:
            continue
        if domain and domain not in domains:
            continue

        score_entry = scores.get(name, {}) if isinstance(scores, dict) else {}
        score = score_entry.get("quality_score", 0) if isinstance(score_entry, dict) else 0
        if score < min_quality:
            continue

        results.append({
            "name": name,
            "description": skill.get("description", ""),
            "domains": domains_list,
            "quality_score": score,
            "source": skill.get("source", ""),
        })

    results.sort(key=lambda r: -r["quality_score"])
    results = results[:limit]

    if not results:
        return format_tool_result("No matching skills found.")

    text = f"Found {len(results)} skill(s):\n\n"
    for r in results:
        domains_str = ", ".join(r["domains"][:3])
        text += f"  {r['name']:35s}  Q:{r['quality_score']:3d}  [{domains_str}]\n"
        text += f"  {'':35s}  {r['description'][:80]}\n\n"
    return format_tool_result(text)


def handle_get_skill(params: dict) -> dict:
    name = params.get("name", "")
    version = params.get("version", "evolved")

    if not name:
        return format_tool_result("Error: 'name' parameter is required", is_error=True)

    if version == "evolved":
        skill_path = EVOLVED_DIR / f"{name}.md"
        if skill_path.exists():
            text = skill_path.read_text(encoding="utf-8")
            return format_tool_result(text)

    registry = load_json(REGISTRY_DIR / "skills-registry.json")
    if isinstance(registry, list):
        for skill in registry:
            if skill.get("name") == name:
                text = f"# {name}\n\n"
                text += f"**Description:** {skill.get('description', 'N/A')}\n"
                text += f"**Domains:** {', '.join(skill.get('domains', []))}\n"
                text += f"**Source:** {skill.get('source', 'N/A')}\n"
                raw_path = skill.get("local_path", "")
                if raw_path:
                    raw_file = Path(raw_path)
                    if raw_file.exists():
                        text += f"\n---\n\n{raw_file.read_text(encoding='utf-8')}"
                return format_tool_result(text)

    return format_tool_result(f"Skill '{name}' not found in registry.", is_error=True)


def handle_get_stats(params: dict) -> dict:
    registry = load_json(REGISTRY_DIR / "skills-registry.json")
    quality = load_json(REGISTRY_DIR / "quality.json")
    clusters_data = load_json(REGISTRY_DIR / "clusters.json")
    patterns = load_json(REGISTRY_DIR / "patterns.json")
    gaps = load_json(REGISTRY_DIR / "gaps.json")
    log_data = load_json(REGISTRY_DIR / "evolved" / "evolution-log.json")

    skill_count = len(registry) if isinstance(registry, list) else 0
    domain_clusters = clusters_data.get("clusters", {}) if isinstance(clusters_data, dict) else {}
    domains = list(domain_clusters.keys())
    avg_score = quality.get("avg_score", "N/A") if quality else "N/A"
    grades = quality.get("grade_distribution", {}) if quality else {}
    top_patterns = patterns.get("top_sections", [])[:5] if patterns else []

    text = f"Registry Statistics:\n"
    text += f"  Skills:     {skill_count}\n"
    text += f"  Domains:    {len(domains)} ({', '.join(domains[:8])}{'...' if len(domains) > 8 else ''})\n"
    text += f"  Avg Score:  {avg_score}\n"
    text += f"  Grades:     A={grades.get('A', 0)} B={grades.get('B', 0)} "
    text += f"C={grades.get('C', 0)} D={grades.get('D', 0)}\n"

    if log_data:
        text += f"\nEvolution:\n"
        text += f"  Evolved:    {log_data.get('total_evolved', 0)}\n"
        text += f"  Sections:   +{log_data.get('total_sections_added', 0)}\n"

    if top_patterns:
        text += f"\nTop sections across skills:\n"
        for p in top_patterns:
            text += f"  - {p.get('section', '?')}: {p.get('count', 0)} skills\n"

    gaps_list = gaps.get("gaps", {}).get("underdeveloped", []) if isinstance(gaps, dict) else []
    if gaps_list:
        text += f"\nGaps (domains below expected):\n"
        for g in gaps_list:
            domain = g.get("domain", "?")
            deficit = g.get("deficit", 0)
            text += f"  - {domain}: {deficit} skills below expected\n"

    return format_tool_result(text)


def handle_get_status(params: dict) -> dict:
    pipeline_log = load_json(REGISTRY_DIR / "pipeline-log.json")
    pipeline_state = load_json(REGISTRY_DIR / "pipeline-state.json")
    scheduler_state = load_json(REGISTRY_DIR / "scheduler" / "scheduler-state.json")

    text = "Pipeline Status:\n"
    if pipeline_state:
        text += f"  Total runs: {pipeline_state.get('total_runs', 0)}\n"
        for stage, info in pipeline_state.get("stages", {}).items():
            status = info.get("status", "?")
            last = info.get("last_run", "?")[:19]
            text += f"  {stage}: {status} (last: {last})\n"
    else:
        text += "  No pipeline runs yet.\n"

    if isinstance(pipeline_log, list) and pipeline_log:
        last = pipeline_log[-1]
        text += f"\nLast pipeline run: #{last.get('run_number', '?')}\n"
        text += f"  Skills: {last.get('skills_count', '?')} | Evolved: {last.get('evolved_count', '?')}\n"
        text += f"  Result: {'PASSED' if last.get('all_passed') else 'FAILED'}\n"
        text += f"  Stages: {json.dumps(last.get('stages', {}))}\n"

    if scheduler_state:
        sr = scheduler_state.get("last_run", {})
        text += f"\nScheduler:\n"
        text += f"  Runs: {scheduler_state.get('total_runs', 0)}\n"
        text += f"  Last: {sr.get('run_id', 'N/A')} "
        text += f"({'SUCCESS' if sr.get('success') else 'FAILED'})\n"
        text += f"  Time: {sr.get('timestamp', '?')[:19]}\n"
    else:
        text += f"\nScheduler: No runs yet.\n"

    return format_tool_result(text)


def handle_list_stages(params: dict) -> dict:
    text = "Available pipeline stages:\n"
    for stage, script in STAGE_SCRIPTS.items():
        exists = "+" if os.path.exists(script) else "x"
        text += f"  [{exists}] {stage}: {script}\n"
    stages_str = ", ".join(STAGES_ORDER)
    text += f"\nRun with: run_pipeline stages=[{stages_str}]"
    return format_tool_result(text)


# ---- Router ----

TOOL_HANDLERS = {
    "discover_skills": handle_discover_skills,
    "analyze_skills": handle_analyze_skills,
    "research_domain": handle_research_domain,
    "evolve_skills": handle_evolve_skills,
    "export_skills": handle_export_skills,
    "run_pipeline": handle_run_pipeline,
    "search_skills": handle_search_skills,
    "get_skill": handle_get_skill,
    "get_stats": handle_get_stats,
    "get_status": handle_get_status,
    "list_stages": handle_list_stages,
}


def handle_request(msg: dict):
    method = msg.get("method", "")
    params = msg.get("params", {})
    msg_id = msg.get("id")

    if method == "initialize":
        client_info = params.get("clientInfo", {})
        log(f"Client connected: {client_info.get('name', '?')} v{client_info.get('version', '?')}")
        send_response(msg_id, {
            "protocolVersion": PROTOCOL_VERSION,
            "capabilities": {"tools": {}},
            "serverInfo": {"name": SERVER_NAME, "version": SERVER_VERSION},
        })
        return

    if method == "notifications/initialized":
        log("Client initialized")
        return

    if method == "notifications/cancelled":
        return

    if method == "tools/list":
        tools = tool_list_tools()
        send_response(msg_id, {"tools": tools})
        return

    if method == "tools/call":
        tool_name = params.get("name", "")
        tool_args = params.get("arguments", {})

        if tool_name in TOOL_HANDLERS:
            log(f"Tool call: {tool_name}")
            try:
                result = TOOL_HANDLERS[tool_name](tool_args)
                send_response(msg_id, result)
            except Exception as e:
                log(f"Error in {tool_name}: {e}")
                traceback.print_exc(file=sys.stderr)
                send_response(msg_id, None, {
                    "code": -32603,
                    "message": f"Internal error: {e}",
                })
        else:
            send_response(msg_id, None, {
                "code": -32601,
                "message": f"Tool not found: {tool_name}",
            })
        return

    if method == "ping":
        send_response(msg_id, {})
        return

    log(f"Unknown method: {method}")
    if msg_id is not None:
        send_response(msg_id, None, {
            "code": -32601,
            "message": f"Method not found: {method}",
        })


def main():
    log(f"{SERVER_NAME} v{SERVER_VERSION} starting (stdio)")
    log(f"Registry: {REGISTRY_DIR}")
    log(f"Waiting for requests...")

    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue
        try:
            msg = json.loads(line)
            handle_request(msg)
        except json.JSONDecodeError as e:
            log(f"Invalid JSON: {e}")
        except Exception as e:
            log(f"Fatal error: {e}")
            traceback.print_exc(file=sys.stderr)


if __name__ == "__main__":
    main()
