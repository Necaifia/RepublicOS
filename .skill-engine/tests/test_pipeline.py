"""
Integration tests for RepublicOS Skill Engine.
Tests that all stages run correctly and produce valid output.
"""

import json
import os
import subprocess
import sys
import time
import unittest
from pathlib import Path

SCRIPT_DIR = Path(__file__).parent.parent.resolve()
REGISTRY_DIR = SCRIPT_DIR / "registry"


def run_script(name: str, timeout: int = 120) -> dict:
    script = SCRIPT_DIR / f"{name}.py"
    start = time.time()
    try:
        result = subprocess.run(
            [sys.executable, str(script)],
            capture_output=True, text=True, timeout=timeout,
        )
        return {
            "success": result.returncode == 0,
            "returncode": result.returncode,
            "stdout": result.stdout,
            "stderr": result.stderr,
            "elapsed": time.time() - start,
        }
    except subprocess.TimeoutExpired:
        return {"success": False, "returncode": -1, "stdout": "", "stderr": "Timeout", "elapsed": timeout}
    except Exception as e:
        return {"success": False, "returncode": -1, "stdout": "", "stderr": str(e), "elapsed": time.time() - start}


def load_json(path: Path):
    if path.exists():
        with open(path, encoding="utf-8") as f:
            return json.load(f)
    return {}


class TestDiscovery(unittest.TestCase):
    def test_registry_exists(self):
        registry = load_json(REGISTRY_DIR / "skills-registry.json")
        self.assertIsInstance(registry, list)
        self.assertGreater(len(registry), 0, "No skills discovered")

    def test_registry_structure(self):
        registry = load_json(REGISTRY_DIR / "skills-registry.json")
        for skill in registry[:5]:
            self.assertIn("name", skill, "Skill missing 'name'")
            self.assertIn("description", skill, "Skill missing 'description'")
            self.assertIn("source", skill, "Skill missing 'source'")
            self.assertIn("domains", skill, "Skill missing 'domains'")
            self.assertIn("id", skill, "Skill missing 'id'")

    def test_discovery_config(self):
        config = load_json(SCRIPT_DIR / "discovery-config.json")
        self.assertIn("known_repos", config)
        repos = config["known_repos"]
        self.assertIsInstance(repos, list)
        self.assertGreater(len(repos), 0, "No repos configured")
        for repo in repos:
            self.assertIn("name", repo)
            self.assertIn("path", repo)

    def test_sources_file(self):
        sources = load_json(REGISTRY_DIR / "sources.json")
        self.assertIsInstance(sources, dict)
        self.assertGreater(len(sources), 0, "No sources tracked")


class TestAnalyzer(unittest.TestCase):
    def test_clusters_exist(self):
        clusters = load_json(REGISTRY_DIR / "clusters.json")
        self.assertIsInstance(clusters, dict)
        clusters_data = clusters.get("clusters", {}) if isinstance(clusters, dict) else {}
        self.assertGreater(len(clusters_data), 0, "No domain clusters found")

    def test_quality_scores(self):
        quality = load_json(REGISTRY_DIR / "quality.json")
        self.assertIsInstance(quality, dict)
        self.assertTrue("average_score" in quality or "avg_score" in quality,
                        "quality.json missing average_score/avg_score")
        self.assertIn("grade_distribution", quality,
                      "quality.json missing grade_distribution")
        self.assertTrue("all_scores" in quality or "scores" in quality,
                        "quality.json missing all_scores/scores")

    def test_patterns(self):
        patterns = load_json(REGISTRY_DIR / "patterns.json")
        self.assertIsInstance(patterns, dict)
        self.assertIn("top_sections", patterns)

    def test_gaps(self):
        gaps = load_json(REGISTRY_DIR / "gaps.json")
        self.assertIsInstance(gaps, dict)
        self.assertIn("gaps", gaps)


class TestResearch(unittest.TestCase):
    def test_research_files_exist(self):
        research_dir = REGISTRY_DIR / "research"
        if research_dir.exists():
            files = list(research_dir.glob("*.md"))
            self.assertGreater(len(files), 0, "No research files found")
            for f in files:
                content = f.read_text(encoding="utf-8")
                self.assertGreater(len(content), 10, f"Research file {f.name} is too short")


class TestEvolution(unittest.TestCase):
    def test_evolved_directory(self):
        evolved_dir = REGISTRY_DIR / "evolved"
        self.assertTrue(evolved_dir.exists(), "Evolved directory missing")
        md_files = list(evolved_dir.glob("*.md"))
        self.assertGreater(len(md_files), 0, "No evolved skill files found")

    def test_evolution_log(self):
        log_data = load_json(REGISTRY_DIR / "evolved" / "evolution-log.json")
        self.assertIsInstance(log_data, dict)
        self.assertIn("total_evolved", log_data)
        self.assertGreater(log_data.get("total_evolved", 0), 0)

    def test_evolved_content_has_improvements(self):
        evolved_dir = REGISTRY_DIR / "evolved"
        md_files = list(evolved_dir.glob("*.md"))
        if not md_files:
            self.skipTest("No evolved files")
        sample = md_files[0]
        content = sample.read_text(encoding="utf-8")
        new_sections = ["Prerequisites", "Error Handling", "Security",
                        "Rollback", "Verification", "Configuration"]
        found = [s for s in new_sections if s.lower() in content.lower()]
        self.assertGreater(len(found), 0, f"No new sections found in {sample.name}")


class TestStore(unittest.TestCase):
    def test_store_directory(self):
        store_dir = SCRIPT_DIR / "store" / "agent-skills"
        self.assertTrue(store_dir.exists(), "Store directory missing")
        agents = [d.name for d in store_dir.iterdir() if d.is_dir()]
        expected = {"cline", "claude", "opencode", "generic"}
        self.assertTrue(expected.issubset(set(agents)),
                        f"Missing agents. Have: {agents}, Need: {expected}")

    def test_store_has_files(self):
        store_dir = SCRIPT_DIR / "store" / "agent-skills"
        for agent_dir in store_dir.iterdir():
            if agent_dir.is_dir():
                files = list(agent_dir.rglob("*.md"))
                self.assertGreater(len(files), 0, f"No SKILL.md files in {agent_dir.name}")

    def test_store_catalog(self):
        catalog = load_json(SCRIPT_DIR / "store" / "agent-skills" / "catalog.json")
        self.assertIsInstance(catalog, dict)
        self.assertTrue("total_exports" in catalog or "generated_at" in catalog,
                        "catalog.json missing expected keys")


class TestPipeline(unittest.TestCase):
    def test_pipeline_log(self):
        log_data = load_json(REGISTRY_DIR / "pipeline-log.json")
        self.assertIsInstance(log_data, list)
        self.assertGreater(len(log_data), 0, "No pipeline runs recorded")

    def test_pipeline_state(self):
        state = load_json(REGISTRY_DIR / "pipeline-state.json")
        self.assertIsInstance(state, dict)
        self.assertIn("total_runs", state)
        self.assertIn("stages", state)

    def test_all_stages_passed_recently(self):
        log_data = load_json(REGISTRY_DIR / "pipeline-log.json")
        if isinstance(log_data, list) and log_data:
            last = log_data[-1]
            stages = last.get("stages", {})
            failed = [k for k, v in stages.items() if v == "failed"]
            self.assertEqual(len(failed), 0, f"Failed stages: {failed}")


class TestEndToEnd(unittest.TestCase):
    def test_full_pipeline_can_run(self):
        result = run_script("pipeline", timeout=180)
        self.assertTrue(result["success"], f"Pipeline failed: {result['stderr'][:500]}")

    def test_benchmark_produces_report(self):
        result = run_script("benchmark", timeout=120)
        self.assertTrue(result["success"], f"Benchmark failed: {result['stderr'][:500]}")
        report = REGISTRY_DIR / "benchmark" / "REPORT.md"
        self.assertTrue(report.exists(), "Benchmark report not generated")

    def test_dashboard_generates_html(self):
        result = run_script("dashboard", timeout=60)
        self.assertTrue(result["success"], f"Dashboard failed: {result['stderr'][:500]}")
        html = SCRIPT_DIR / "dashboard.html"
        self.assertTrue(html.exists(), "dashboard.html not generated")
        self.assertGreater(html.stat().st_size, 1000, "dashboard.html too small")


class TestBenchmark(unittest.TestCase):
    def test_benchmark_data(self):
        data = load_json(REGISTRY_DIR / "benchmark" / "benchmark.json")
        if not data:
            self.skipTest("No benchmark data")
        summary = data.get("summary", {}) if isinstance(data, dict) else {}
        self.assertIn("avg_old_score", summary)
        self.assertIn("avg_new_score", summary)
        self.assertIn("avg_score_delta", summary)
        if summary.get("avg_score_delta"):
            self.assertGreater(summary["avg_score_delta"], 0, "No quality improvement")

    def test_benchmark_grades_improved(self):
        data = load_json(REGISTRY_DIR / "benchmark" / "benchmark.json")
        if not data:
            self.skipTest("No benchmark data")
        grades = data.get("grade_distribution", {}) if isinstance(data, dict) else {}
        old_d = grades.get("old", {}).get("D", 0)
        new_d = grades.get("new", {}).get("D", 0)
        self.assertLess(new_d, old_d, "Grade D should decrease after evolution")


if __name__ == "__main__":
    unittest.main(verbosity=2)
