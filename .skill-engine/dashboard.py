"""
dashboard.py — Generate static HTML dashboard for Skill Engine
Embeds all registry data into a single self-contained HTML file.
"""

import json
from datetime import datetime, timezone
from pathlib import Path

SCRIPT_DIR = Path(__file__).parent.resolve()
REGISTRY_DIR = SCRIPT_DIR / "registry"
OUTPUT_FILE = SCRIPT_DIR / "dashboard.html"


def load_json(path: Path):
    if path.exists():
        with open(path, encoding="utf-8") as f:
            return json.load(f)
    return {}


def esc(text: str) -> str:
    return text.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;").replace('"', "&quot;")


def color_for_score(score: int) -> str:
    if score >= 80:
        return "#22c55e"
    if score >= 60:
        return "#3b82f6"
    if score >= 40:
        return "#eab308"
    return "#ef4444"


def build_html():
    registry = load_json(REGISTRY_DIR / "skills-registry.json") or []
    quality = load_json(REGISTRY_DIR / "quality.json") or {}
    clusters_data = load_json(REGISTRY_DIR / "clusters.json") or {}
    gaps = load_json(REGISTRY_DIR / "gaps.json") or {}
    patterns = load_json(REGISTRY_DIR / "patterns.json") or {}
    pipeline_log = load_json(REGISTRY_DIR / "pipeline-log.json") or []
    pipeline_state = load_json(REGISTRY_DIR / "pipeline-state.json") or {}
    scheduler_state = load_json(REGISTRY_DIR / "scheduler" / "scheduler-state.json") or {}
    bench = load_json(REGISTRY_DIR / "benchmark" / "benchmark.json") or {}

    skill_count = len(registry) if isinstance(registry, list) else 0
    domain_clusters = clusters_data.get("clusters", {}) if isinstance(clusters_data, dict) else {}
    domain_count = len(domain_clusters)
    sources = set()
    for s in (registry if isinstance(registry, list) else []):
        src = s.get("source", "")
        if src:
            sources.add(src)

    grades = quality.get("grade_distribution", {}) if isinstance(quality, dict) else {}
    avg_score = quality.get("avg_score", "N/A")
    top_patterns = patterns.get("top_sections", [])[:8] if isinstance(patterns, dict) else []
    gaps_list = gaps.get("gaps", {}).get("underdeveloped", []) if isinstance(gaps, dict) else []

    bench_summary = bench.get("summary", {}) if isinstance(bench, dict) else {}
    bench_grades = bench.get("grade_distribution", {}) if isinstance(bench, dict) else {}
    bench_top = bench.get("top_improvements", [])[:10] if isinstance(bench, dict) else []

    last_pipeline = pipeline_log[-1] if isinstance(pipeline_log, list) and pipeline_log else {}
    scheduler_last = scheduler_state.get("last_run", {}) if isinstance(scheduler_state, dict) else {}

    evolved_count = 0
    sections_added = 0
    log_data = load_json(REGISTRY_DIR / "evolved" / "evolution-log.json")
    if isinstance(log_data, dict):
        evolved_count = log_data.get("total_evolved", 0)
        sections_added = log_data.get("total_sections_added", 0)

    skills_json = json.dumps(registry, ensure_ascii=True)

    html = r"""<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>RepublicOS Skill Engine</title>
<style>
*{margin:0;padding:0;box-sizing:border-box}
body{font-family:system-ui,-apple-system,sans-serif;background:#0a0a0f;color:#e2e8f0;line-height:1.6}
h1{font-size:1.5rem;font-weight:700;color:#f8fafc}
h2{font-size:1.1rem;font-weight:600;color:#94a3b8;margin-bottom:0.75rem;letter-spacing:0.05em;text-transform:uppercase}
.header{background:linear-gradient(135deg,#1e1b4b,#0f172a);padding:1.5rem 2rem;border-bottom:1px solid #1e293b}
.header h1{display:flex;align-items:center;gap:0.75rem}
.header h1 span{font-size:0.75rem;color:#64748b;font-weight:400;margin-left:auto}
.stats{display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:1rem;padding:1.5rem 2rem}
.stat-card{background:#13131f;border:1px solid #1e293b;border-radius:8px;padding:1.25rem;transition:border-color 0.2s}
.stat-card:hover{border-color:#3b82f6}
.stat-card .value{font-size:2rem;font-weight:700;color:#f8fafc}
.stat-card .label{font-size:0.8rem;color:#64748b;margin-top:0.25rem}
.stat-card .sub{font-size:0.75rem;color:#475569;margin-top:0.25rem}
.metric-grid{display:grid;grid-template-columns:repeat(auto-fit,minmax(140px,1fr));gap:0.5rem;margin-top:0.75rem}
.metric{text-align:center;padding:0.5rem;background:#1a1a2e;border-radius:6px}
.metric .val{font-size:1.25rem;font-weight:700}
.metric .lbl{font-size:0.7rem;color:#64748b}
.main{padding:0 2rem 2rem}
.section{background:#13131f;border:1px solid #1e293b;border-radius:8px;padding:1.5rem;margin-bottom:1.5rem}
table{width:100%;border-collapse:collapse;font-size:0.875rem}
th{text-align:left;padding:0.625rem 0.75rem;color:#64748b;font-weight:500;border-bottom:1px solid #1e293b;font-size:0.75rem;text-transform:uppercase;letter-spacing:0.05em}
td{padding:0.625rem 0.75rem;border-bottom:1px solid #1a1a2e}
tr:hover td{background:#1a1a2e}
.badge{display:inline-block;padding:0.15rem 0.5rem;border-radius:999px;font-size:0.7rem;font-weight:600}
.badge-A{background:#166534;color:#86efac}
.badge-B{background:#1e3a5f;color:#93c5fd}
.badge-C{background:#713f12;color:#fde047}
.badge-D{background:#7f1d1d;color:#fca5a5}
.score-bar{display:inline-block;height:6px;border-radius:3px;width:60px;vertical-align:middle;margin-right:6px}
.search-bar{display:flex;gap:0.75rem;margin-bottom:1rem;flex-wrap:wrap}
.search-bar input,.search-bar select{padding:0.5rem 0.75rem;background:#0a0a0f;border:1px solid #1e293b;border-radius:6px;color:#e2e8f0;font-size:0.875rem}
.search-bar input{flex:1;min-width:200px}
.search-bar select{min-width:140px}
.grade-bar{display:flex;height:24px;border-radius:6px;overflow:hidden;margin:0.75rem 0}
.grade-bar div{display:flex;align-items:center;justify-content:center;font-size:0.7rem;font-weight:700;color:#fff;transition:width 0.3s}
.grade-A{background:#22c55e}
.grade-B{background:#3b82f6}
.grade-C{background:#eab308}
.grade-D{background:#ef4444}
.progress-row{display:flex;align-items:center;gap:0.75rem;margin:0.5rem 0}
.progress-row .name{width:160px;font-size:0.8rem}
.progress-row .track{flex:1;height:8px;background:#1a1a2e;border-radius:4px;overflow:hidden}
.progress-row .fill{height:100%;border-radius:4px;transition:width 0.5s}
@media(max-width:768px){.stats{grid-template-columns:1fr 1fr}.stat-card .value{font-size:1.5rem}.main{padding:0 1rem 1rem}.header{padding:1rem}}
</style>
</head>
<body>
<div class="header">
<h1>RepublicOS Skill Engine <span>v1.0</span></h1>
</div>

<div class="stats" id="stats">
</div>

<div class="main">
<div class="section" id="grade-section">
<h2>Grade Distribution</h2>
<div id="grade-bars"></div>
<div id="benchmark-comparison" style="margin-top:1rem;display:none"></div>
</div>

<div class="section" id="top-improvements-section" style="display:none">
<h2>Top Improvements (Evolution)</h2>
<div id="top-improvements"></div>
</div>

<div class="section" id="gaps-section">
<h2>Domain Gaps</h2>
<div id="gaps"></div>
</div>

<div class="section" id="patterns-section">
<h2>Top Sections</h2>
<div id="patterns"></div>
</div>

<div class="section" id="pipeline-section">
<h2>Pipeline Status</h2>
<div id="pipeline-status"></div>
</div>

<div class="section" id="skills-section">
<h2>Skills</h2>
<div class="search-bar">
<input type="text" id="skill-search" placeholder="Search by name, description, domain..." oninput="filterSkills()">
<select id="domain-filter" onchange="filterSkills()"><option value="">All domains</option></select>
<select id="grade-filter" onchange="filterSkills()">
<option value="">All grades</option>
<option value="A">A (80-100)</option>
<option value="B">B (60-79)</option>
<option value="C">C (40-59)</option>
<option value="D">D (0-39)</option>
</select>
<span id="skill-count" style="color:#64748b;font-size:0.875rem;align-self:center"></span>
</div>
<table><thead><tr>
<th>Skill</th><th>Domain</th><th>Source</th><th>Score</th><th>Description</th>
</tr></thead><tbody id="skills-tbody"></tbody></table>
</div>
</div>

<script>
const SKILLS = """ + skills_json + r""";
const GRADES = {"old":{"A":0,"B":0,"C":0,"D":0},"new":{"A":0,"B":0,"C":0,"D":0}};
const BENCH_TOP = [];
const BENCH_SUMMARY = {};
const GAPS = [];
const PATTERNS = [];
const PIPELINE = {};
const SCHEDULER = {};

function init(data) {
    Object.assign(GRADES, data.grades);
    BENCH_TOP.push(...data.benchTop);
    Object.assign(BENCH_SUMMARY, data.benchSummary);
    GAPS.push(...data.gaps);
    PATTERNS.push(...data.patterns);
    Object.assign(PIPELINE, data.pipeline);
    Object.assign(SCHEDULER, data.scheduler);
    render();
}

function render() {
    renderStats();
    renderGrades();
    renderBenchmark();
    renderGaps();
    renderPatterns();
    renderPipeline();
    renderSkills();
}

function renderStats() {
    const oldG = GRADES.old || {};
    const newG = GRADES.new || {};
    const total = SKILLS.length;
    const domains = new Set();
    const sources = new Set();
    SKILLS.forEach(s => { if(s.domains) s.domains.forEach(d => domains.add(d)); if(s.source) sources.add(s.source); });
    const p = PIPELINE;
    const bench = BENCH_SUMMARY;
    const oldScore = bench.avg_old_score || '?';
    const newScore = bench.avg_new_score || '?';
    const delta = bench.avg_score_delta || '?';

    document.getElementById('stats').innerHTML = `
        <div class="stat-card"><div class="value">${total}</div><div class="label">Skills Discovered</div></div>
        <div class="stat-card"><div class="value">${domains.size}</div><div class="label">Domains</div><div class="sub">from ${sources.size} sources</div></div>
        <div class="stat-card"><div class="value">${oldScore}<span style="font-size:1rem;color:#64748b"> → </span>${newScore}</div><div class="label">Avg Quality Score</div><div class="sub" style="color:${delta > 0 ? '#22c55e' : '#ef4444'}">${delta > 0 ? '+' : ''}${delta} point improvement</div></div>
        <div class="stat-card"><div class="value">${newG.B + newG.C + newG.A}</div><div class="label">Skills at B or Better</div><div class="sub">up from ${oldG.B + oldG.C + oldG.A}</div></div>
        <div class="stat-card"><div class="value">${p.evolved_count || 0}</div><div class="label">Skills Evolved</div><div class="sub">${p.skills_count || 0} total in registry</div></div>
        <div class="stat-card"><div class="value">${SCHEDULER.total_runs || 0}</div><div class="label">Scheduled Runs</div><div class="sub">Last: ${SCHEDULER.last_run ? (SCHEDULER.last_run.success ? 'OK' : 'FAIL') : 'Never'}</div></div>
    `;
}

function renderGrades() {
    const buildBar = (grades, label) => {
        const total = Object.values(grades).reduce((a,b) => a + b, 0) || 1;
        const grades_order = ['A','B','C','D'];
        return grades_order.map(g => {
            const count = grades[g] || 0;
            const pct = (count / total * 100).toFixed(0);
            return count ? `<div class="grade-${g}" style="width:${pct}%">${count}</div>` : '';
        }).join('');
    };
    const oldG = GRADES.old || {A:0,B:0,C:0,D:0};
    const newG = GRADES.new || {A:0,B:0,C:0,D:0};
    const oldTotal = Object.values(oldG).reduce((a,b) => a + b, 0);
    const newTotal = Object.values(newG).reduce((a,b) => a + b, 0);
    document.getElementById('grade-bars').innerHTML = `
        <div style="margin-bottom:0.5rem;font-size:0.8rem;color:#64748b">Before Evolution (${oldTotal} skills)</div>
        <div class="grade-bar">${buildBar(oldG)}</div>
        <div style="margin-top:0.75rem;margin-bottom:0.5rem;font-size:0.8rem;color:#64748b">After Evolution (${newTotal} skills)</div>
        <div class="grade-bar">${buildBar(newG)}</div>
        <div style="display:flex;gap:1rem;margin-top:0.75rem;font-size:0.75rem">
        <span><span class="badge badge-A">A</span> 80-100</span>
        <span><span class="badge badge-B">B</span> 60-79</span>
        <span><span class="badge badge-C">C</span> 40-59</span>
        <span><span class="badge badge-D">D</span> 0-39</span>
        </div>
    `;
}

function renderBenchmark() {
    const top = BENCH_TOP;
    const summary = BENCH_SUMMARY;
    if (!top.length && !summary.avg_old_score) return;
    document.getElementById('benchmark-comparison').style.display = 'block';

    let html = '<div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(160px,1fr));gap:0.75rem;margin-bottom:1rem">';
    const metrics = [
        {l:'Old Avg Score', v:summary.avg_old_score},
        {l:'New Avg Score', v:summary.avg_new_score},
        {l:'Score Delta', v:'+' + summary.avg_score_delta, c:'#22c55e'},
        {l:'Sections Added', v:summary.total_sections_added},
        {l:'Description +', v:'+' + Math.round(summary.avg_desc_change) + ' chars'},
        {l:'Token +', v:'+' + Math.round(summary.avg_token_change) + ' tokens'},
        {l:'Tags Coverage', v:summary.new_tags_pct + '%', sub:'was ' + summary.old_tags_pct + '%'},
        {l:'Version Coverage', v:summary.new_version_pct + '%', sub:'was ' + summary.old_version_pct + '%'},
    ];
    metrics.forEach(m => {
        html += '<div class="metric"><div class="val" style="color:' + (m.c || '#f8fafc') + '">' + m.v + '</div><div class="lbl">' + m.l + '</div>';
        if (m.sub) html += '<div style="font-size:0.65rem;color:#475569">' + m.sub + '</div>';
        html += '</div>';
    });
    html += '</div>';

    if (top.length) {
        html += '<table><thead><tr><th>Skill</th><th>Before</th><th>After</th><th>+/-</th></tr></thead><tbody>';
        top.forEach(r => {
            html += '<tr><td>' + r.name + '</td><td>' + r.old_score + '</td><td>' + r.new_score + '</td><td style="color:#22c55e">+' + r.delta + '</td></tr>';
        });
        html += '</tbody></table>';
    }
    document.getElementById('top-improvements').innerHTML = html;
    document.getElementById('top-improvements-section').style.display = 'block';
}

function renderGaps() {
    if (!GAPS.length) {
        document.getElementById('gaps').innerHTML = '<div style="color:#64748b">No domain gaps detected.</div>';
        return;
    }
    document.getElementById('gaps').innerHTML = GAPS.map(g =>
        `<div class="progress-row">
            <div class="name">${g.domain}</div>
            <div class="track"><div class="fill" style="width:${Math.min(100, (1 - g.deficit / Math.max(g.deficit + g.current, 1)) * 100)}%;background:#eab308"></div></div>
            <span style="font-size:0.8rem;color:#94a3b8">${g.current}/${g.expected} (${g.deficit} missing)</span>
        </div>`
    ).join('');
}

function renderPatterns() {
    if (!PATTERNS.length) {
        document.getElementById('patterns').innerHTML = '<div style="color:#64748b">No pattern data.</div>';
        return;
    }
    const maxCount = Math.max(...PATTERNS.map(p => p.count));
    document.getElementById('patterns').innerHTML = PATTERNS.map(p =>
        `<div class="progress-row">
            <div class="name">${p.section}</div>
            <div class="track"><div class="fill" style="width:${(p.count / maxCount * 100).toFixed(0)}%;background:#3b82f6"></div></div>
            <span style="font-size:0.8rem;color:#94a3b8">${p.count}/${SKILLS.length} skills</span>
        </div>`
    ).join('');
}

function renderPipeline() {
    const p = PIPELINE;
    let html = '<div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(140px,1fr));gap:0.5rem">';
    const stages = ['discovery','analyzer','research','evolution','store'];
    const stageStatus = p.stages || {};
    stages.forEach(s => {
        const info = stageStatus[s] || {};
        const ok = info.status === 'passed';
        html += '<div class="metric"><div class="val" style="font-size:1.5rem;color:' + (ok ? '#22c55e' : '#64748b') + '">' + (ok ? '+' : '-') + '</div><div class="lbl">' + s + '</div>';
        if (info.last_run) html += '<div style="font-size:0.65rem;color:#475569">' + (info.last_run||'').slice(0,10) + '</div>';
        html += '</div>';
    });
    html += '</div>';
    html += '<div style="margin-top:0.75rem;font-size:0.8rem;color:#64748b">';
    html += 'Total runs: ' + (p.total_runs || 0) + ' &middot; ';
    html += 'Skills: ' + (p.skills_count || 0) + ' &middot; ';
    html += 'Evolved: ' + (p.evolved_count || 0);
    html += '</div>';
    document.getElementById('pipeline-status').innerHTML = html;
}

function getScore(skill) {
    return skill.quality_score || skill.frontmatter_quality || 0;
}

function getDomain(skill) {
    const d = skill.domains;
    if (Array.isArray(d)) return d.join(', ');
    return d || 'uncategorized';
}

function getGrade(score) {
    if (score >= 80) return 'A';
    if (score >= 60) return 'B';
    if (score >= 40) return 'C';
    return 'D';
}

function renderSkills(filtered) {
    const list = filtered || SKILLS;
    const tbody = document.getElementById('skills-tbody');
    document.getElementById('skill-count').textContent = list.length + ' skills';
    tbody.innerHTML = list.map(s => {
        const name = s.name || s.id || '?';
        const desc = (s.description || '').slice(0, 100);
        const score = getScore(s);
        const grade = getGrade(score);
        const domain = getDomain(s);
        const source = s.source || s.label || '?';
        const pct = Math.min(score, 100);
        return '<tr><td><strong>' + esc(name) + '</strong></td>'
            + '<td>' + esc(domain) + '</td>'
            + '<td><span style="color:#64748b">' + esc(source) + '</span></td>'
            + '<td><span class="score-bar" style="width:' + pct + '%;background:' + color_for_score(score) + '"></span>' + score + ' <span class="badge badge-' + grade + '">' + grade + '</span></td>'
            + '<td style="color:#94a3b8;font-size:0.8rem">' + esc(desc) + '</td></tr>';
    }).join('');
}

function color_for_score(score) {
    if (score >= 80) return '#22c55e';
    if (score >= 60) return '#3b82f6';
    if (score >= 40) return '#eab308';
    return '#ef4444';
}

function filterSkills() {
    const q = document.getElementById('skill-search').value.toLowerCase();
    const domainFilter = document.getElementById('domain-filter').value;
    const gradeFilter = document.getElementById('grade-filter').value;
    const filtered = SKILLS.filter(s => {
        const name = (s.name || '').toLowerCase();
        const desc = (s.description || '').toLowerCase();
        const domain = getDomain(s).toLowerCase();
        const score = getScore(s);
        const grade = getGrade(score);
        if (q && !name.includes(q) && !desc.includes(q) && !domain.includes(q)) return false;
        if (domainFilter && !domain.includes(domainFilter)) return false;
        if (gradeFilter && grade !== gradeFilter) return false;
        return true;
    });
    renderSkills(filtered);
}

function populateDomainFilter() {
    const domains = new Set();
    SKILLS.forEach(s => {
        const d = s.domains;
        if (Array.isArray(d)) d.forEach(dd => domains.add(dd));
        else if (d) domains.add(d);
    });
    const sel = document.getElementById('domain-filter');
    [...domains].sort().forEach(d => {
        const opt = document.createElement('option');
        opt.value = d;
        opt.textContent = d;
        sel.appendChild(opt);
    });
}

document.addEventListener('DOMContentLoaded', () => {
    const data = {
        grades: """ + json.dumps({
            "old": {"A": sum(1 for s in registry if isinstance(s, dict) and 80 <= (s.get("quality_score") or s.get("frontmatter_quality", 0))),
                    "B": sum(1 for s in registry if isinstance(s, dict) and 60 <= (s.get("quality_score") or s.get("frontmatter_quality", 0)) < 80),
                    "C": sum(1 for s in registry if isinstance(s, dict) and 40 <= (s.get("quality_score") or s.get("frontmatter_quality", 0)) < 60),
                    "D": sum(1 for s in registry if isinstance(s, dict) and (s.get("quality_score") or s.get("frontmatter_quality", 0)) < 40)},
            "new": {"A": grades.get("A", 0), "B": grades.get("B", 0), "C": grades.get("C", 0), "D": grades.get("D", 0)}
        }, ensure_ascii=True) + r""",
        benchTop: """ + json.dumps(bench_top, ensure_ascii=True) + r""",
        benchSummary: """ + json.dumps(bench_summary, ensure_ascii=True) + r""",
        gaps: """ + json.dumps(gaps_list, ensure_ascii=True) + r""",
        patterns: """ + json.dumps(top_patterns, ensure_ascii=True) + r""",
        pipeline: """ + json.dumps({
            "stages": pipeline_state.get("stages", {}) if isinstance(pipeline_state, dict) else {},
            "total_runs": pipeline_state.get("total_runs", 0) if isinstance(pipeline_state, dict) else 0,
            "skills_count": last_pipeline.get("skills_count", skill_count),
            "evolved_count": evolved_count,
        }, ensure_ascii=True) + r""",
        scheduler: """ + json.dumps({
            "total_runs": scheduler_state.get("total_runs", 0) if isinstance(scheduler_state, dict) else 0,
            "last_run": scheduler_last.get("success", None) if isinstance(scheduler_last, dict) else None,
        }, ensure_ascii=True) + r""",
    };
    init(data);
    populateDomainFilter();
});
</script>
</body>
</html>"""
    OUTPUT_FILE.write_text(html, encoding="utf-8")
    print(f"Dashboard generated: {OUTPUT_FILE}")


if __name__ == "__main__":
    build_html()
