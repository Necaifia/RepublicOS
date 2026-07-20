use std::time::{Duration, Instant};
use std::io::{self, Write};
use std::thread;
use pantheon_kernel::cell_id::CellId;
use pantheon_kernel::event::EventPayload;
use pantheon_scheduler::task::{ScheduledTask, TaskPriority};
use pantheon_plugin::{PantheonPlugin, TaskAction};

const CLEAR: &str = "\x1B[2J\x1B[H";
const HIDE: &str = "\x1B[?25l";
const SHOW: &str = "\x1B[?25h";
const RST: &str = "\x1B[0m";
const GRN: &str = "\x1B[32m";
const YLW: &str = "\x1B[33m";
const RED: &str = "\x1B[31m";
const CYN: &str = "\x1B[36m";
const BLD: &str = "\x1B[1m";
const DIM: &str = "\x1B[2m";

#[derive(Clone, Copy, PartialEq)]
enum Status {
    Idle,
    Working,
    Done,
    #[allow(dead_code)]
    Failed,
}

struct Stage {
    name: &'static str,
    status: Status,
    progress: f32,
    task: String,
}

pub struct Pipeline {
    stages: Vec<Stage>,
    timeline: Vec<String>,
    start: Instant,
}

impl Pipeline {
    pub fn todo() -> Self {
        Self {
            stages: vec![
                Stage { name: "Planner", status: Status::Idle, progress: 0.0, task: String::new() },
                Stage { name: "Backend", status: Status::Idle, progress: 0.0, task: String::new() },
                Stage { name: "Test",    status: Status::Idle, progress: 0.0, task: String::new() },
                Stage { name: "Review",  status: Status::Idle, progress: 0.0, task: String::new() },
                Stage { name: "Release", status: Status::Idle, progress: 0.0, task: String::new() },
            ],
            timeline: Vec::new(),
            start: Instant::now(),
        }
    }

    pub fn run(mut self) {
        let mut stdout = io::stdout();
        print!("{}", HIDE);
        let _ = stdout.flush();

        // Intro splash
        self.render_splash(&mut stdout);

        // Run each stage
        self.run_stage(&mut stdout, 0, "Creating implementation plan", &[
            ("Analyzing requirements...", Duration::from_millis(400)),
            ("Defining 5 features...",    Duration::from_millis(600)),
            ("Creating plan structure...", Duration::from_millis(500)),
            ("Plan complete",             Duration::from_millis(300)),
        ]);

        self.run_stage(&mut stdout, 1, "Generating code from plan", &[
            ("Scaffolding project...",    Duration::from_millis(500)),
            ("Implementing models...",    Duration::from_millis(700)),
            ("Writing API handlers...",   Duration::from_millis(600)),
            ("Adding database layer...",  Duration::from_millis(400)),
            ("Code generation complete",  Duration::from_millis(300)),
        ]);

        self.run_stage(&mut stdout, 2, "Running test suite", &[
            ("Running unit tests...",     Duration::from_millis(400)),
            ("Running integration tests...", Duration::from_millis(600)),
            ("Checking coverage...",      Duration::from_millis(300)),
            ("All 47 tests passed",       Duration::from_millis(200)),
        ]);

        self.run_stage(&mut stdout, 3, "Reviewing implementation", &[
            ("Checking code quality...",  Duration::from_millis(400)),
            ("Reviewing security...",     Duration::from_millis(500)),
            ("Verifying best practices...", Duration::from_millis(300)),
            ("Review approved",           Duration::from_millis(200)),
        ]);

        self.run_stage(&mut stdout, 4, "Releasing build", &[
            ("Building artifacts...",     Duration::from_millis(500)),
            ("Tagging v0.1.0...",         Duration::from_millis(300)),
            ("Publishing package...",     Duration::from_millis(400)),
            ("Release complete!",         Duration::from_millis(200)),
        ]);

        // Final summary
        self.render_summary(&mut stdout);
        print!("{}", SHOW);
        let _ = stdout.flush();
    }

    pub fn run_with_plugin(mut self, plugin: &dyn PantheonPlugin) {
        let mut stdout = io::stdout();
        print!("{}", HIDE);
        let _ = stdout.flush();

        let _ = plugin.on_start();

        self.render_splash(&mut stdout);

        self.run_stage(&mut stdout, 0, "Creating implementation plan", &[
            ("Analyzing requirements...", Duration::from_millis(400)),
            ("Defining 5 features...",    Duration::from_millis(600)),
            ("Creating plan structure...", Duration::from_millis(500)),
            ("Plan complete",             Duration::from_millis(300)),
        ]);

        self.run_stage(&mut stdout, 1, "Generating code from plan", &[
            ("Scaffolding project...",    Duration::from_millis(500)),
            ("Implementing models...",    Duration::from_millis(700)),
            ("Writing API handlers...",   Duration::from_millis(600)),
            ("Adding database layer...",  Duration::from_millis(400)),
            ("Code generation complete",  Duration::from_millis(300)),
        ]);

        self.run_stage(&mut stdout, 2, "Running test suite", &[
            ("Running unit tests...",     Duration::from_millis(400)),
            ("Running integration tests...", Duration::from_millis(600)),
            ("Checking coverage...",      Duration::from_millis(300)),
            ("All 47 tests passed",       Duration::from_millis(200)),
        ]);

        self.run_stage_with_plugin(&mut stdout, 3, plugin);

        if self.stages[3].status == Status::Done {
            self.run_stage(&mut stdout, 4, "Releasing build", &[
                ("Building artifacts...",     Duration::from_millis(500)),
                ("Tagging v0.1.0...",         Duration::from_millis(300)),
                ("Publishing package...",     Duration::from_millis(400)),
                ("Release complete!",         Duration::from_millis(200)),
            ]);

            self.render_summary(&mut stdout);
        } else {
            self.render_failed_summary(&mut stdout);
        }

        let _ = plugin.on_shutdown();
        print!("{}", SHOW);
        let _ = stdout.flush();
    }

    fn run_stage_with_plugin(&mut self, stdout: &mut io::Stdout, idx: usize, plugin: &dyn PantheonPlugin) {
        self.stages[idx].status = Status::Working;
        self.stages[idx].task = "Awaiting human approval".to_string();
        self.render(stdout);
        self.log(&format!("{}  {} {}", self.stages[idx].name, "▶", "Awaiting human review"));

        let task = ScheduledTask::new(
            CellId::random(),
            EventPayload::new(
                br#"{"task":"release_approval","requires_human":true,"version":"0.1.0","tests_passed":47}"#.to_vec(),
                "application/json",
            ),
            TaskPriority::Normal,
        );

        self.stages[idx].progress = 50.0;
        self.render(stdout);

        match plugin.on_task(&task).unwrap_or(TaskAction::Pass) {
            TaskAction::Pass | TaskAction::Handled => {
                self.stages[idx].progress = 100.0;
                self.stages[idx].status = Status::Done;
                self.stages[idx].task = "Review approved".to_string();
                self.log(&format!("{}  {} Human approved release", self.stages[idx].name, "✓"));
                self.render(stdout);
                thread::sleep(Duration::from_millis(400));
            }
            TaskAction::Cancel(reason) => {
                self.stages[idx].status = Status::Failed;
                let short = if reason.len() > 14 { &reason[..14] } else { &reason };
                self.stages[idx].task = short.to_string();
                self.log(&format!("{}  ✗ {}", self.stages[idx].name, reason));
                self.render(stdout);
                thread::sleep(Duration::from_secs(2));
            }
        }
    }

    fn run_stage(&mut self, stdout: &mut io::Stdout, idx: usize, task: &str, steps: &[(&str, Duration)]) {
        self.stages[idx].status = Status::Working;
        self.stages[idx].task = task.to_string();
        self.log(&format!("{}  {} {}", self.stages[idx].name, "▶", task));

        let total_dur: Duration = steps.iter().map(|(_, d)| *d).sum();
        let mut elapsed = Duration::ZERO;

        for (step, dur) in steps {
            self.stages[idx].task = step.to_string();
            let step_start = Instant::now();

            while step_start.elapsed() < *dur {
                let step_progress = step_start.elapsed().as_secs_f32() / dur.as_secs_f32();
                self.stages[idx].progress = (elapsed.as_secs_f32() + step_progress * dur.as_secs_f32()) / total_dur.as_secs_f32() * 100.0;
                self.render(stdout);
                thread::sleep(Duration::from_millis(30));
            }

            elapsed += *dur;
            self.stages[idx].progress = elapsed.as_secs_f32() / total_dur.as_secs_f32() * 100.0;
            self.render(stdout);
        }

        self.stages[idx].status = Status::Done;
        self.stages[idx].progress = 100.0;
        self.log(&format!("{}  {} {}", self.stages[idx].name, "✓", "Complete"));
        self.render(stdout);
        thread::sleep(Duration::from_millis(200));
    }

    fn render(&self, stdout: &mut io::Stdout) {
        let mut out = String::new();
        out.push_str(CLEAR);
        out.push_str(&format!("{}{:^70}{}\n\n", BLD, "Pantheon Pipeline — Todo App", RST));

        // Cell boxes
        for i in 0..self.stages.len() {
            out.push_str(&format!("┌{:─^14}┐", ""));
            if i < self.stages.len() - 1 { out.push_str("  ──→  "); }
        }
        out.push('\n');

        for i in 0..self.stages.len() {
            out.push_str(&format!("│{:^14}│", self.stages[i].name));
            if i < self.stages.len() - 1 { out.push_str("       "); }
        }
        out.push('\n');

        for i in 0..self.stages.len() {
            let bar = self.bar(self.stages[i].progress, 12);
            let (s, c) = match self.stages[i].status {
                Status::Idle => (" IDLE      ", DIM),
                Status::Working => ("▶ WORKING  ", YLW),
                Status::Done => ("✓ DONE     ", GRN),
                Status::Failed => ("✗ FAILED   ", RED),
            };
            out.push_str(&format!("│{}   {}{}{}│", bar, c, s, RST));
            if i < self.stages.len() - 1 { out.push_str("       "); }
        }
        out.push('\n');

        for i in 0..self.stages.len() {
            let task = &self.stages[i].task;
            let task_display = if task.is_empty() { "waiting..." } else { task };
            if task_display.len() > 14 {
                out.push_str(&format!("│{:<14}│", &task_display[..14]));
            } else {
                out.push_str(&format!("│{:^14}│", task_display));
            }
            if i < self.stages.len() - 1 { out.push_str("       "); }
        }
        out.push('\n');

        for i in 0..self.stages.len() {
            out.push_str(&format!("└{:─^14}┘", ""));
            if i < self.stages.len() - 1 { out.push_str("  ──→  "); }
        }
        out.push_str("\n\n");

        // Timeline
        out.push_str(&format!("{}{}{}\n", CYN, "═".repeat(60), RST));
        out.push_str(&format!("{}  Event Timeline{}\n", BLD, RST));
        out.push_str(&format!("{}{}{}\n", CYN, "═".repeat(60), RST));

        let start = if self.timeline.len() > 12 { self.timeline.len() - 12 } else { 0 };
        for entry in &self.timeline[start..] {
            out.push_str(&format!("  {}\n", entry));
        }

        print!("{}", out);
        let _ = stdout.flush();
    }

    fn render_splash(&self, stdout: &mut io::Stdout) {
        let mut out = String::new();
        out.push_str(CLEAR);
        out.push_str(&format!("{}{:^70}{}\n\n", BLD, "Pantheon Pipeline Engine", RST));
        out.push_str(&format!("  {}Initializing cells...{}\n\n", BLD, RST));
        for stage in &self.stages {
            out.push_str(&format!("  {}  {} · {}\n", "○", stage.name, "waiting"));
        }
        out.push_str("\n\n  Press Ctrl+C to stop\n");
        print!("{}", out);
        let _ = stdout.flush();
        thread::sleep(Duration::from_millis(800));
    }

    fn render_summary(&self, stdout: &mut io::Stdout) {
        let mut out = String::new();
        out.push_str(CLEAR);
        out.push_str(&format!("{}{:^70}{}\n\n", BLD, "Pipeline Complete", RST));
        out.push_str(&format!("  {}✓{} All 5 stages completed successfully\n\n", GRN, RST));

        for stage in &self.stages {
            out.push_str(&format!("  {} {} — Complete\n", GRN, stage.name));
        }

        out.push_str("\n");
        out.push_str(&format!("  {}═══ Summary {}{}\n", CYN, "═".repeat(40), RST));
        out.push_str(&format!("  Total time:  {:?}\n", self.start.elapsed()));
        out.push_str(&format!("  Events:      {}\n", self.timeline.len()));
        out.push_str(&format!("  Stages:      {}\n\n", self.stages.len()));
        out.push_str(&format!("  {}Next: pantheon init my-project{}  to create your own pipeline\n", DIM, RST));
        out.push_str(&format!("  {}      pantheon dashboard 8080{} to monitor live\n", DIM, RST));

        print!("{}", out);
        let _ = stdout.flush();
    }

    fn render_failed_summary(&self, stdout: &mut io::Stdout) {
        let mut out = String::new();
        out.push_str(CLEAR);
        out.push_str(&format!("{}{:^70}{}\n\n", BLD, "Pipeline Blocked", RST));
        out.push_str(&format!("  {}✗{} Pipeline paused — human decision required\n\n", RED, RST));

        for stage in &self.stages {
            match stage.status {
                Status::Done => out.push_str(&format!("  {} {} — Complete\n", GRN, stage.name)),
                Status::Failed => out.push_str(&format!("  {} {} — {}\n", RED, stage.name, stage.task)),
                Status::Working => out.push_str(&format!("  {} {} — {}\n", YLW, stage.name, stage.task)),
                Status::Idle => out.push_str(&format!("  {} {} — pending\n", DIM, stage.name)),
            }
        }

        out.push_str("\n");
        out.push_str(&format!("  {}═══ Human decision required to continue{}\n\n", CYN, RST));
        out.push_str("  Try: pantheon run --with-human  to approve interactively\n\n");

        print!("{}", out);
        let _ = stdout.flush();
    }

    fn bar(&self, pct: f32, width: usize) -> String {
        let filled = ((pct / 100.0) * width as f32).round() as usize;
        let filled = filled.min(width);
        let empty = width.saturating_sub(filled);
        let mut s = String::new();
        s.push_str("[");
        if filled > 0 {
            s.push_str(&format!("{}{}", GRN, "█".repeat(filled.saturating_sub(1))));
            if filled == width { s.push_str("█"); } else { s.push_str("▓"); }
        }
        s.push_str(&format!("{}{}", DIM, "░".repeat(empty)));
        s.push_str(&format!("{}]", RST));
        s
    }

    fn log(&mut self, msg: &str) {
        let elapsed = self.start.elapsed();
        let ts = format!("{:02}:{:02}:{:02}.{:03}",
            (elapsed.as_secs() / 3600) % 24,
            (elapsed.as_secs() / 60) % 60,
            elapsed.as_secs() % 60,
            elapsed.as_millis() % 1000,
        );
        self.timeline.push(format!("{}{}{}  {}", DIM, ts, RST, msg));
    }
}
