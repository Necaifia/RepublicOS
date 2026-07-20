# RepublicOS Skill Engine

Autonomous pipeline that discovers, analyzes, researches, evolves, and exports AI agent skills at scale.

## Quick Start

```bash
# Full pipeline (all 5 stages)
python pipeline.py

# Specific stages
python pipeline.py --stages discovery analyzer

# With notifications
python pipeline.py --notify
```

## Stages

| Stage | Script | Description |
|-------|--------|-------------|
| Discovery | `discovery.py` | Clones 13+ GitHub repos, parses SKILL.md, classifies by domain |
| Analyzer | `analyzer.py` | Clustering, pattern detection, gap analysis, quality scoring |
| Research | `research.py` | Fetches best practices from official docs per domain |
| Evolution | `evolution.py` | Enhances skills with domain research, adds missing sections |
| Store | `store.py` | Exports to 4 agent formats (Cline, Claude, OpenCode, generic) |

## Tools

### Benchmark
```bash
python benchmark.py
# Generates registry/benchmark/REPORT.md + benchmark.json
```

### Scheduler (auto daily runs)
```bash
python scheduler.py run          # Run once
python scheduler.py status       # View run history
python scheduler.py list         # List recent runs

# Windows (Task Scheduler)
powershell -File install-scheduler.ps1 -Time "03:00"

# Linux/macOS (cron)
chmod +x install-scheduler.sh
./install-scheduler.sh --time 03:00
```

### MCP Server (for Claude Code, Cline, OpenCode)
```bash
python mcp-server.py
# 11 tools: discover, analyze, research, evolve, export,
#           search_skills, get_skill, get_stats, get_status,
#           list_stages, run_pipeline
```

### REST API
```bash
python api-server.py
# http://localhost:8742/api/health
# http://localhost:8742/api/stats
# http://localhost:8742/api/skills
# http://localhost:8742/api/domains
# http://localhost:8742/api/pipeline
# POST http://localhost:8742/api/pipeline/run
```

### Web Dashboard
```bash
python dashboard.py
# Generates dashboard.html — open in browser
```

### Skill Diff (before/after evolution)
```bash
python diff.py --list                          # List all
python diff.py xlsx                            # Show diff
python diff.py xlsx --html                     # Generate HTML diff
python diff.py --all                           # Diff all skills
python diff.py --all --html                    # HTML for all
```

### Notifications (Slack / Email)
```bash
python notifier.py setup           # Interactive config
python notifier.py test --type slack   # Test Slack
python notifier.py test --type email   # Test Email
```

## GitHub Actions

The workflow (`.github/workflows/skill-engine.yml`) runs automatically:
- Daily at 3:00 AM
- On push to main/master (only when skill-engine files change)
- Manual trigger via GitHub UI with stage selection

## Docker

```bash
docker compose run --rm skill-engine
docker compose run --rm skill-engine-mcp
docker compose run --rm skill-engine-dashboard
```

## Results

| Metric | Before | After | Improvement |
|--------|--------|-------|-------------|
| Avg Quality Score | 33.9 | 65.8 | **+31.9 (+94%)** |
| Tags in frontmatter | 0.0% | 90.9% | +90.9% |
| Version in frontmatter | 4.5% | 100.0% | +95.5% |
| Grade B or better | 0 | 73 | **+73 skills** |
| Grade D | 73 | 0 | **-73 skills** |

## Data

All data lives in `registry/`:
- `skills-registry.json` — All discovered skills with metadata
- `clusters.json` — Domain clusters
- `patterns.json` — Section patterns across skills
- `gaps.json` — Coverage gaps by domain
- `quality.json` — Quality scores and grades
- `evolved/` — Evolved skill files + evolution log
- `research/` — Domain research docs
- `benchmark/` — Benchmark report + data
- `pipeline-log.json` — Pipeline run history
- `scheduler/` — Scheduler state + logs
- `raw/` — Raw discovered skill files

## Architecture

```
discovery.py  →  skills-registry.json
                    ↓
analyzer.py   →  clusters + patterns + gaps + quality
                    ↓
research.py   →  domain best practices (20+ sources)
                    ↓
evolution.py  →  enhanced skills with new sections
                    ↓
store.py      →  Cline / Claude Code / OpenCode / generic
```

Each stage reads the output of the previous stage and writes its own enriched data. Stages are independently runnable and checkpoint-safe.
