"""
api-server.py — REST API for RepublicOS Skill Engine
Exposes skill registry, stats, and pipeline execution over HTTP.
Zero external dependencies — uses Python stdlib only.
"""

import json
import os
import subprocess
import sys
import traceback
from datetime import datetime, timezone
from http.server import HTTPServer, BaseHTTPRequestHandler
from pathlib import Path
from typing import Any

SCRIPT_DIR = Path(__file__).parent.resolve()
REGISTRY_DIR = SCRIPT_DIR / "registry"
EVOLVED_DIR = REGISTRY_DIR / "evolved"
HOST = os.environ.get("SKILL_ENGINE_HOST", "0.0.0.0")
PORT = int(os.environ.get("SKILL_ENGINE_PORT", "8742"))


def load_json(path: Path):
    if path.exists():
        with open(path, encoding="utf-8") as f:
            return json.load(f)
    return {}


CORS_HEADERS = {
    "Access-Control-Allow-Origin": "*",
    "Access-Control-Allow-Methods": "GET, POST, OPTIONS",
    "Access-Control-Allow-Headers": "Content-Type",
}


def json_response(handler, data: Any, status: int = 200):
    body = json.dumps(data, ensure_ascii=False, indent=2).encode("utf-8")
    handler.send_response(status)
    handler.send_header("Content-Type", "application/json; charset=utf-8")
    handler.send_header("Content-Length", len(body))
    for k, v in CORS_HEADERS.items():
        handler.send_header(k, v)
    handler.end_headers()
    handler.wfile.write(body)


def error_response(handler, msg: str, status: int = 400):
    json_response(handler, {"error": msg}, status)


class SkillAPIHandler(BaseHTTPRequestHandler):
    def log_message(self, format, *args):
        ts = datetime.now(timezone.utc).strftime("%H:%M:%S")
        print(f"[{ts}] [API] {args[0]} {args[1]} {args[2]}")

    def do_OPTIONS(self):
        self.send_response(204)
        for k, v in CORS_HEADERS.items():
            self.send_header(k, v)
        self.end_headers()

    def do_GET(self):
        path = self.path.rstrip("/")
        try:
            if path == "/api/stats" or path == "/api/status":
                self.handle_stats()
            elif path == "/api/skills":
                self.handle_list_skills()
            elif path.startswith("/api/skills/") and path.endswith("/diff"):
                name = path[len("/api/skills/"):-len("/diff")]
                self.handle_skill_diff(name)
            elif path.startswith("/api/skills/"):
                name = path[len("/api/skills/"):]
                self.handle_get_skill(name)
            elif path == "/api/domains":
                self.handle_domains()
            elif path == "/api/health":
                json_response(self, {"status": "ok", "timestamp": datetime.now(timezone.utc).isoformat()})
            elif path == "/api/pipeline":
                self.handle_pipeline_status()
            elif path == "/" or path == "" or path == "/dashboard":
                self.handle_dashboard()
            else:
                error_response(self, f"Not found: {path}", 404)
        except Exception as e:
            traceback.print_exc()
            error_response(self, str(e), 500)

    def do_POST(self):
        path = self.path.rstrip("/")
        content_length = int(self.headers.get("Content-Length", 0))
        body = self.rfile.read(content_length) if content_length else b"{}"
        try:
            data = json.loads(body) if body else {}
        except json.JSONDecodeError:
            data = {}

        try:
            if path == "/api/pipeline/run":
                self.handle_run_pipeline(data)
            elif path == "/api/notify/test":
                self.handle_notify_test(data)
            else:
                error_response(self, f"Not found: {path}", 404)
        except Exception as e:
            traceback.print_exc()
            error_response(self, str(e), 500)

    def handle_root(self):
        endpoints = [
            "GET  /", "GET  /api/health",
            "GET  /api/stats", "GET  /api/status",
            "GET  /api/skills", "GET  /api/skills/{name}",
            "GET  /api/skills/{name}/diff", "GET  /api/domains",
            "GET  /api/pipeline", "POST /api/pipeline/run",
            "POST /api/notify/test",
        ]
        json_response(self, {
            "service": "RepublicOS Skill Engine API",
            "version": "1.0.0",
            "endpoints": endpoints,
        })

    def handle_dashboard(self):
        self.send_response(200)
        self.send_header("Content-Type", "text/html; charset=utf-8")
        for k, v in CORS_HEADERS.items():
            self.send_header(k, v)
        self.end_headers()
        self.wfile.write(DASHBOARD_HTML.encode("utf-8"))

    def handle_stats(self):
        registry = load_json(REGISTRY_DIR / "skills-registry.json") or []
        quality = load_json(REGISTRY_DIR / "quality.json") or {}
        clusters_data = load_json(REGISTRY_DIR / "clusters.json") or {}
        pipeline_log = load_json(REGISTRY_DIR / "pipeline-log.json") or []
        log_data = load_json(REGISTRY_DIR / "evolved" / "evolution-log.json") or {}
        bench = load_json(REGISTRY_DIR / "benchmark" / "benchmark.json") or {}

        skill_count = len(registry) if isinstance(registry, list) else 0
        domain_clusters = clusters_data.get("clusters", {}) if isinstance(clusters_data, dict) else {}
        grades = quality.get("grade_distribution", {}) if isinstance(quality, dict) else {}
        bench_summary = bench.get("summary", {}) if isinstance(bench, dict) else {}
        last_pipeline = pipeline_log[-1] if isinstance(pipeline_log, list) and pipeline_log else {}

        json_response(self, {
            "skills": {
                "total": skill_count,
                "evolved": log_data.get("total_evolved", 0),
                "domains": len(domain_clusters),
                "sections_added": log_data.get("total_sections_added", 0),
            },
            "quality": {
                "avg_score": quality.get("avg_score", "N/A"),
                "grades": {"A": grades.get("A", 0), "B": grades.get("B", 0),
                           "C": grades.get("C", 0), "D": grades.get("D", 0)},
            },
            "benchmark": {
                "old_score": bench_summary.get("avg_old_score"),
                "new_score": bench_summary.get("avg_new_score"),
                "delta": bench_summary.get("avg_score_delta"),
            },
            "pipeline": {
                "last_run": last_pipeline.get("run_number"),
                "skills_count": last_pipeline.get("skills_count"),
                "evolved_count": last_pipeline.get("evolved_count"),
                "passed": last_pipeline.get("all_passed"),
            },
        })

    def handle_list_skills(self):
        registry = load_json(REGISTRY_DIR / "skills-registry.json") or []
        quality = load_json(REGISTRY_DIR / "quality.json") or {}
        scores = {}
        if isinstance(quality, dict):
            for s in quality.get("skills", []):
                if isinstance(s, dict):
                    scores[s.get("name", "")] = s.get("quality_score", 0)

        items = []
        for s in registry if isinstance(registry, list) else []:
            name = s.get("name", "")
            items.append({
                "name": name,
                "description": (s.get("description", "") or "")[:120],
                "domains": s.get("domains", []),
                "source": s.get("source", ""),
                "quality_score": scores.get(name, 0),
            })

        items.sort(key=lambda x: -x["quality_score"])
        json_response(self, {"total": len(items), "skills": items})

    def handle_get_skill(self, name: str):
        if not name:
            error_response(self, "Skill name required")
            return

        # Try evolved first
        evolved_content = None
        for f in EVOLVED_DIR.glob(f"*{name}*"):
            if f.suffix == ".md":
                evolved_content = f.read_text(encoding="utf-8")
                break

        registry = load_json(REGISTRY_DIR / "skills-registry.json") or []
        skill = None
        for s in registry if isinstance(registry, list) else []:
            if s.get("name") == name:
                skill = s
                break

        if not skill and not evolved_content:
            error_response(self, f"Skill '{name}' not found", 404)
            return

        quality = load_json(REGISTRY_DIR / "quality.json") or {}
        score = 0
        if isinstance(quality, dict):
            for s in quality.get("skills", []):
                if isinstance(s, dict) and s.get("name") == name:
                    score = s.get("quality_score", 0)

        json_response(self, {
            "name": name,
            "domains": skill.get("domains", []) if skill else [],
            "source": skill.get("source", "") if skill else "",
            "quality_score": score,
            "description": skill.get("description", "") if skill else "",
            "evolved": evolved_content is not None,
            "content": evolved_content,
        })

    def handle_skill_diff(self, name: str):
        if not name:
            error_response(self, "Skill name required")
            return

        registry = load_json(REGISTRY_DIR / "skills-registry.json") or []
        skill = None
        for s in registry if isinstance(registry, list) else []:
            if s.get("name") == name:
                skill = s
                break

        if not skill:
            error_response(self, f"Skill '{name}' not found", 404)
            return

        local_path = skill.get("local_path", "")
        original = ""
        if local_path:
            f = Path(local_path)
            if f.exists():
                original = f.read_text(encoding="utf-8")

        evolved = ""
        for f in EVOLVED_DIR.glob(f"*{name}*"):
            if f.suffix == ".md":
                evolved = f.read_text(encoding="utf-8")
                break

        import difflib
        diff = list(difflib.unified_diff(
            original.split("\n") if original else [],
            evolved.split("\n") if evolved else [],
            fromfile="original", tofile="evolved", lineterm="",
        ))

        json_response(self, {
            "name": name,
            "original_lines": len(original.split("\n")) if original else 0,
            "evolved_lines": len(evolved.split("\n")) if evolved else 0,
            "has_original": bool(original),
            "has_evolved": bool(evolved),
            "diff": diff,
        })

    def handle_domains(self):
        clusters_data = load_json(REGISTRY_DIR / "clusters.json") or {}
        domain_clusters = clusters_data.get("clusters", {}) if isinstance(clusters_data, dict) else {}
        gaps = load_json(REGISTRY_DIR / "gaps.json") or {}
        gaps_list = gaps.get("gaps", {}).get("underdeveloped", []) if isinstance(gaps, dict) else []
        gap_domains = {g.get("domain"): g for g in gaps_list}

        domains = []
        for name, info in sorted(domain_clusters.items()):
            gap = gap_domains.get(name)
            domains.append({
                "name": name,
                "skill_count": info.get("skill_count", 0) if isinstance(info, dict) else 0,
                "gap": gap.get("deficit", 0) if gap else 0,
                "sources": info.get("sources", []) if isinstance(info, dict) else [],
            })

        json_response(self, {"domains": domains})

    def handle_pipeline_status(self):
        pipeline_state = load_json(REGISTRY_DIR / "pipeline-state.json") or {}
        pipeline_log = load_json(REGISTRY_DIR / "pipeline-log.json") or []
        scheduler_state = load_json(REGISTRY_DIR / "scheduler" / "scheduler-state.json") or {}

        last_run = pipeline_log[-1] if isinstance(pipeline_log, list) and pipeline_log else {}
        scheduler_last = scheduler_state.get("last_run", {}) if isinstance(scheduler_state, dict) else {}

        json_response(self, {
            "pipeline": {
                "total_runs": pipeline_state.get("total_runs", 0) if isinstance(pipeline_state, dict) else 0,
                "stages": pipeline_state.get("stages", {}) if isinstance(pipeline_state, dict) else {},
            },
            "last_run": {
                "number": last_run.get("run_number"),
                "skills": last_run.get("skills_count"),
                "evolved": last_run.get("evolved_count"),
                "passed": last_run.get("all_passed"),
                "stages": last_run.get("stages", {}),
            },
            "scheduler": {
                "total_runs": scheduler_state.get("total_runs", 0) if isinstance(scheduler_state, dict) else 0,
                "last_run": {
                    "id": scheduler_last.get("run_id"),
                    "success": scheduler_last.get("success"),
                    "timestamp": scheduler_last.get("timestamp"),
                } if scheduler_last else None,
            },
        })

    def handle_run_pipeline(self, data: dict):
        stages = data.get("stages", [])
        timeout = data.get("timeout", 600)
        notify = data.get("notify", False)

        pipeline_script = SCRIPT_DIR / "pipeline.py"
        cmd = [sys.executable, str(pipeline_script)]
        if stages:
            cmd.extend(["--stages"] + stages)
        cmd.extend(["--timeout", str(timeout)])
        if notify:
            cmd.append("--notify")

        try:
            result = subprocess.run(cmd, capture_output=True, text=True, timeout=timeout + 60)
            json_response(self, {
                "success": result.returncode == 0,
                "returncode": result.returncode,
                "output": result.stdout[-3000:],
                "error": result.stderr[-1000:] if result.stderr else "",
            })
        except subprocess.TimeoutExpired:
            json_response(self, {"success": False, "error": "Timeout"}, 408)

    def handle_notify_test(self, data: dict):
        try:
            notifier_script = SCRIPT_DIR / "notifier.py"
            result = subprocess.run(
                [sys.executable, str(notifier_script), "test", "--type", data.get("type", "slack")],
                capture_output=True, text=True, timeout=30,
            )
            json_response(self, {
                "success": result.returncode == 0,
                "output": result.stdout,
                "error": result.stderr,
            })
        except Exception as e:
            json_response(self, {"success": False, "error": str(e)}, 500)


def main():
    server = HTTPServer((HOST, PORT), SkillAPIHandler)
    print(f"Skill Engine API: http://{HOST}:{PORT}")
    print(f"  GET  /api/health          Health check")
    print(f"  GET  /api/stats           Registry statistics")
    print(f"  GET  /api/skills          List all skills")
    print(f"  GET  /api/skills/{'{name}'}       Get skill details")
    print(f"  GET  /api/skills/{'{name}'}/diff  Skill evolution diff")
    print(f"  GET  /api/domains         Domain breakdown")
    print(f"  GET  /api/pipeline        Pipeline status")
    print(f"  GET  /dashboard           Live web dashboard")
    print(f"  POST /api/pipeline/run    Trigger pipeline run")
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        print("\nShutting down...")
        server.server_close()


DASHBOARD_HTML = r"""<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>RepublicOS Skill Engine</title>
<style>
*{margin:0;padding:0;box-sizing:border-box}
body{font-family:system-ui,-apple-system,sans-serif;background:#0a0a0f;color:#e2e8f0;line-height:1.6;padding:0}
.header{background:linear-gradient(135deg,#1e1b4b,#0f172a);padding:1.25rem 2rem;border-bottom:1px solid #1e293b;display:flex;gap:1rem;align-items:center;flex-wrap:wrap}
.header h1{font-size:1.25rem;font-weight:700;color:#f8fafc}
.header .nav{display:flex;gap:0.5rem;margin-left:auto}
.header .nav a{color:#64748b;text-decoration:none;font-size:0.85rem;padding:0.35rem 0.75rem;border-radius:6px;border:1px solid #1e293b}
.header .nav a:hover{color:#e2e8f0;border-color:#3b82f6}
.stats{display:grid;grid-template-columns:repeat(auto-fit,minmax(160px,1fr));gap:0.75rem;padding:1.25rem 2rem}
.stat-card{background:#13131f;border:1px solid #1e293b;border-radius:8px;padding:1rem}
.stat-card .value{font-size:1.5rem;font-weight:700;color:#f8fafc}
.stat-card .label{font-size:0.75rem;color:#64748b;margin-top:0.15rem}
.stat-card .sub{font-size:0.7rem;color:#475569}
.main{padding:0 2rem 2rem}
.section{background:#13131f;border:1px solid #1e293b;border-radius:8px;padding:1.25rem;margin-bottom:1rem}
.section h2{font-size:0.9rem;font-weight:600;color:#94a3b8;margin-bottom:0.75rem;text-transform:uppercase;letter-spacing:0.05em}
.grade-bar{display:flex;height:20px;border-radius:4px;overflow:hidden;margin:0.5rem 0}
.grade-bar div{display:flex;align-items:center;justify-content:center;font-size:0.65rem;font-weight:700;color:#fff}
.grade-A{background:#22c55e}.grade-B{background:#3b82f6}.grade-C{background:#eab308}.grade-D{background:#ef4444}
table{width:100%;border-collapse:collapse;font-size:0.8rem}
th{text-align:left;padding:0.5rem;color:#64748b;font-weight:500;border-bottom:1px solid #1e293b;font-size:0.7rem;text-transform:uppercase}
td{padding:0.5rem;border-bottom:1px solid #1a1a2e;font-size:0.8rem}
tr:hover td{background:#1a1a2e}
.badge{display:inline-block;padding:0.1rem 0.4rem;border-radius:999px;font-size:0.65rem;font-weight:600}
.badge-A{background:#166534;color:#86efac}.badge-B{background:#1e3a5f;color:#93c5fd}
.badge-C{background:#713f12;color:#fde047}.badge-D{background:#7f1d1d;color:#fca5a5}
.search-bar{display:flex;gap:0.5rem;margin-bottom:0.75rem;flex-wrap:wrap}
.search-bar input,.search-bar select{padding:0.4rem 0.6rem;background:#0a0a0f;border:1px solid #1e293b;border-radius:4px;color:#e2e8f0;font-size:0.8rem}
.search-bar input{flex:1;min-width:160px}.search-bar select{min-width:120px}
.progress-row{display:flex;align-items:center;gap:0.5rem;margin:0.35rem 0}
.progress-row .name{width:120px;font-size:0.75rem}.progress-row .track{flex:1;height:6px;background:#1a1a2e;border-radius:3px;overflow:hidden}
.progress-row .fill{height:100%;border-radius:3px;transition:width 0.5s}
.loading{text-align:center;padding:2rem;color:#64748b}
.error{color:#ef4444;text-align:center;padding:1rem}
#toast{position:fixed;bottom:1rem;right:1rem;padding:0.75rem 1rem;border-radius:8px;font-size:0.85rem;z-index:999;display:none}
.toast-success{background:#166534;color:#86efac;border:1px solid #22c55e}
.toast-error{background:#7f1d1d;color:#fca5a5;border:1px solid #ef4444}
@media(max-width:768px){.stats{grid-template-columns:1fr 1fr;padding:1rem}.main{padding:0 1rem 1rem}.header{padding:1rem;flex-direction:column}.header .nav{margin-left:0}}
</style>
</head>
<body>
<div class="header">
<h1>RepublicOS Skill Engine</h1>
<div class="nav">
<a href="#" onclick="showTab('stats')" id="tab-stats-btn">Stats</a>
<a href="#" onclick="showTab('skills')" id="tab-skills-btn">Skills</a>
<a href="#" onclick="showTab('domains')" id="tab-domains-btn">Domains</a>
<a href="#" onclick="showTab('pipeline')" id="tab-pipeline-btn">Pipeline</a>
</div>
</div>
<div id="toast"></div>

<div id="stats-tab">
<div class="stats" id="stats-cards"></div>
<div class="main">
<div class="section"><h2>Grade Distribution</h2><div id="grade-section"></div></div>
<div class="section"><h2>Top Sections Across Skills</h2><div id="patterns-section"></div></div>
<div class="section"><h2>Domain Gaps</h2><div id="gaps-section"></div></div>
<div class="section"><h2>Benchmark</h2><div id="benchmark-section"></div></div>
</div>
</div>

<div id="skills-tab" style="display:none">
<div class="main" style="padding-top:1.25rem">
<div class="section">
<h2>Skills</h2>
<div class="search-bar">
<input type="text" id="skill-search" placeholder="Search..." oninput="filterSkills()">
<select id="domain-filter" onchange="filterSkills()"><option value="">All domains</option></select>
<select id="grade-filter" onchange="filterSkills()">
<option value="">All grades</option><option value="A">A (80-100)</option>
<option value="B">B (60-79)</option><option value="C">C (40-59)</option><option value="D">D (0-39)</option>
</select>
<span id="skill-count" style="color:#64748b;font-size:0.8rem;align-self:center"></span>
</div>
<table><thead><tr><th>Name</th><th>Domain</th><th>Source</th><th>Score</th></tr></thead>
<tbody id="skills-tbody"></tbody></table>
</div>
</div>
</div>

<div id="domains-tab" style="display:none">
<div class="main" style="padding-top:1.25rem">
<div class="section"><h2>Domains</h2><div id="domains-list"></div></div>
</div>
</div>

<div id="pipeline-tab" style="display:none">
<div class="main" style="padding-top:1.25rem">
<div class="section"><h2>Pipeline Status</h2><div id="pipeline-status"></div></div>
<div class="section"><h2>Run Pipeline</h2>
<p style="color:#64748b;font-size:0.85rem;margin-bottom:0.75rem">Trigger a pipeline run from here.</p>
<button onclick="runPipeline()" style="padding:0.5rem 1rem;background:#3b82f6;color:#fff;border:none;border-radius:6px;cursor:pointer;font-size:0.85rem">Run Full Pipeline</button>
<div id="pipeline-result" style="margin-top:0.75rem"></div>
</div>
</div>
</div>

<script>
const API = '';
let skillsCache = [];
let domainsCache = [];

function $(id) { return document.getElementById(id) }
function show() { return Date.now().toString(36) + Math.random().toString(36).slice(2) }

function showTab(name) {
    ['stats','skills','domains','pipeline'].forEach(t => {
        $(t+'-tab').style.display = t === name ? '' : 'none';
    });
}

async function api(path) {
    const r = await fetch(API + path);
    if (!r.ok) throw new Error(await r.text());
    return r.json();
}

function toast(msg, type) {
    const t = $('toast');
    t.textContent = msg;
    t.className = type === 'error' ? 'toast-error' : 'toast-success';
    t.style.display = 'block';
    setTimeout(() => t.style.display = 'none', 3000);
}

async function loadStats() {
    try {
        const d = await api('/api/stats');
        const s = d.skills || {};
        const q = d.quality || {};
        const b = d.benchmark || {};
        const g = q.grades || {};
        const p = d.pipeline || {};
        $('stats-cards').innerHTML = `
            <div class="stat-card"><div class="value">${s.total||0}</div><div class="label">Skills</div></div>
            <div class="stat-card"><div class="value">${s.evolved||0}</div><div class="label">Evolved</div></div>
            <div class="stat-card"><div class="value">${s.domains||0}</div><div class="label">Domains</div></div>
            <div class="stat-card"><div class="value">${q.avg_score||'?'}</div><div class="label">Avg Score</div></div>
            <div class="stat-card"><div class="value">${b.old_score||'?'}<span style="font-size:0.9rem;color:#64748b">→${b.new_score||'?'}</span></div><div class="label">Benchmark</div><div class="sub" style="color:#22c55e">+${b.delta||'?'}</div></div>
            <div class="stat-card"><div class="value">${p.last_run||'?'}</div><div class="label">Last Pipeline</div><div class="sub">${p.passed ? 'PASSED' : '?'}</div></div>
        `;
        const total = Object.values(g).reduce((a,b)=>a+b,0)||1;
        $('grade-section').innerHTML = `
            <div class="grade-bar">${['A','B','C','D'].map(l => { const c=g[l]||0; return c ? '<div class="grade-'+l+'" style="width:'+(c/total*100)+'%">'+c+'</div>' : ''; }).join('')}</div>
            <div style="display:flex;gap:0.75rem;font-size:0.7rem">${['A','B','C','D'].map(l => '<span><span class="badge badge-'+l+'">'+l+'</span></span>').join('')}</div>
        `;
    } catch(e) { $('stats-cards').innerHTML = '<div class="error">Failed to load: '+e.message+'</div>' }
}

async function loadSkills() {
    try {
        const d = await api('/api/skills');
        skillsCache = d.skills || [];
        const domains = new Set();
        skillsCache.forEach(s => (s.domains||[]).forEach(d => domains.add(d)));
        const sel = $('domain-filter');
        [...domains].sort().forEach(d => { const o = document.createElement('option'); o.value=d; o.textContent=d; sel.appendChild(o); });
        renderSkills(skillsCache);
    } catch(e) { $('skills-tbody').innerHTML = '<tr><td colspan="4" class="error">Failed: '+e.message+'</td></tr>' }
}

function renderSkills(list) {
    $('skill-count').textContent = list.length + ' skills';
    $('skills-tbody').innerHTML = list.map(s => {
        const name = s.name||'?';
        const score = s.quality_score||0;
        const grade = score >= 80 ? 'A' : score >= 60 ? 'B' : score >= 40 ? 'C' : 'D';
        const domains = (s.domains||[]).slice(0,2).join(', ');
        return '<tr><td><strong>' + esc(name) + '</strong></td>'
            + '<td>' + esc(domains) + '</td>'
            + '<td style="color:#64748b">' + esc(s.source||'') + '</td>'
            + '<td>' + score + ' <span class="badge badge-' + grade + '">' + grade + '</span></td></tr>';
    }).join('');
}

function filterSkills() {
    const q = $('skill-search').value.toLowerCase();
    const df = $('domain-filter').value;
    const gf = $('grade-filter').value;
    const filtered = skillsCache.filter(s => {
        const name = (s.name||'').toLowerCase();
        const desc = (s.description||'').toLowerCase();
        const doms = (s.domains||[]).join(' ').toLowerCase();
        const score = s.quality_score||0;
        const grade = score >= 80 ? 'A' : score >= 60 ? 'B' : score >= 40 ? 'C' : 'D';
        if (q && !name.includes(q) && !desc.includes(q) && !doms.includes(q)) return false;
        if (df && !doms.includes(df)) return false;
        if (gf && grade !== gf) return false;
        return true;
    });
    renderSkills(filtered);
}

async function loadDomains() {
    try {
        const d = await api('/api/domains');
        const domains = d.domains || [];
        $('domains-list').innerHTML = '<table><thead><tr><th>Domain</th><th>Skills</th><th>Gap</th><th>Sources</th></tr></thead><tbody>'
            + domains.map(d => '<tr><td><strong>' + esc(d.name) + '</strong></td><td>' + (d.skill_count||0) + '</td>'
                + '<td>' + (d.gap ? '<span style="color:#eab308">-' + d.gap + '</span>' : '<span style="color:#475569">0</span>') + '</td>'
                + '<td style="color:#64748b;font-size:0.75rem">' + (d.sources||[]).slice(0,3).join(', ') + '</td></tr>'
            ).join('') + '</tbody></table>';
    } catch(e) { $('domains-list').innerHTML = '<div class="error">Failed: '+e.message+'</div>' }
}

async function loadPipeline() {
    try {
        const d = await api('/api/pipeline');
        const p = d.pipeline || {};
        const l = d.last_run || {};
        const s = d.scheduler || {};
        const stages = p.stages || {};
        const stageList = ['discovery','analyzer','research','evolution','store'];
        $('pipeline-status').innerHTML = `
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(100px,1fr));gap:0.5rem;margin-bottom:1rem">
            ${stageList.map(st => {
                const info = stages[st] || {};
                const ok = info.status === 'passed';
                return '<div style="text-align:center;padding:0.5rem;background:#1a1a2e;border-radius:6px">'
                    + '<div style="font-size:1.25rem;font-weight:700;color:' + (ok?'#22c55e':'#64748b') + '">' + (ok?'+':'-') + '</div>'
                    + '<div style="font-size:0.7rem;color:#64748b">' + st + '</div>'
                    + (info.last_run ? '<div style="font-size:0.6rem;color:#475569">' + info.last_run.slice(0,10) + '</div>' : '')
                    + '</div>';
            }).join('')}
            </div>
            <div style="font-size:0.8rem;color:#64748b">
                Total runs: ${p.total_runs||0} &middot;
                Last: #${l.number||'?'} &middot;
                Skills: ${l.skills||'?'} &middot;
                ${l.passed ? '<span style="color:#22c55e">PASSED</span>' : '<span style="color:#ef4444">FAILED</span>'}
            </div>
            ${s.last_run ? '<div style="margin-top:0.5rem;font-size:0.75rem;color:#475569">Scheduler: '+s.total_runs+' runs, last: '+(s.last_run.success?'OK':'FAIL')+'</div>' : ''}
        `;
    } catch(e) { $('pipeline-status').innerHTML = '<div class="error">Failed: '+e.message+'</div>' }
}

async function runPipeline() {
    const btn = event.target;
    btn.disabled = true;
    btn.textContent = 'Running...';
    $('pipeline-result').innerHTML = '<div class="loading">Pipeline running...</div>';
    try {
        const r = await fetch(API + '/api/pipeline/run', { method:'POST',
            headers:{'Content-Type':'application/json'}, body:JSON.stringify({timeout:300}) });
        const d = await r.json();
        if (d.success) {
            $('pipeline-result').innerHTML = '<div style="color:#22c55e;font-size:0.85rem">Pipeline completed successfully</div>';
            toast('Pipeline PASSED', 'success');
        } else {
            $('pipeline-result').innerHTML = '<div style="color:#ef4444;font-size:0.85rem">Pipeline failed</div>';
            toast('Pipeline FAILED', 'error');
        }
    } catch(e) {
        $('pipeline-result').innerHTML = '<div class="error">'+e.message+'</div>';
        toast('Pipeline error: '+e.message, 'error');
    }
    btn.disabled = false;
    btn.textContent = 'Run Full Pipeline';
}

function esc(s) { return (s||'').replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;') }

document.addEventListener('DOMContentLoaded', () => {
    loadStats();
    loadSkills();
    loadDomains();
    loadPipeline();
    showTab('stats');
});
</script>
</body>
</html>"""

if __name__ == "__main__":
    main()
