use std::io::{self, BufRead, Write};
use crate::{PantheonPlugin, PluginContext, TaskAction};
use pantheon_kernel::error::Result;
use pantheon_scheduler::task::ScheduledTask;

pub struct LocalHumanPlugin {
    name: &'static str,
    ctx: Option<PluginContext>,
    prompt_prefix: String,
}

impl LocalHumanPlugin {
    pub fn new() -> Self {
        Self {
            name: "local-human",
            ctx: None,
            prompt_prefix: "  >>> ".to_string(),
        }
    }

    pub fn with_prompt_prefix(mut self, prefix: &str) -> Self {
        self.prompt_prefix = prefix.to_string();
        self
    }

    fn prompt(&self, question: &str, default: &str) -> io::Result<String> {
        let stdin = io::stdin();
        let mut stdout = io::stdout();

        writeln!(stdout)?;
        writeln!(stdout, "  ┌─────────────────────────────────────────────┐")?;
        writeln!(stdout, "  │  Human Decision Required                    │")?;
        writeln!(stdout, "  └─────────────────────────────────────────────┘")?;
        writeln!(stdout)?;
        writeln!(stdout, "  {}", question)?;
        write!(stdout, "  {}{} ", self.prompt_prefix, default)?;
        stdout.flush()?;

        let mut input = String::new();
        stdin.lock().read_line(&mut input)?;
        let input = input.trim().to_string();

        if input.is_empty() { Ok(default.to_string()) } else { Ok(input) }
    }
}

impl PantheonPlugin for LocalHumanPlugin {
    fn name(&self) -> &'static str {
        self.name
    }

    fn description(&self) -> &'static str {
        "Presents decisions to a human operator and relays responses back into the cell pipeline"
    }

    fn init(&mut self, ctx: PluginContext) -> Result<()> {
        self.ctx = Some(ctx);
        Ok(())
    }

    fn on_start(&self) -> Result<()> {
        println!();
        println!("  ╔══════════════════════════════════════════╗");
        println!("  ║     Local Human Plugin Active            ║");
        println!("  ║     Human-as-a-Cell mode engaged         ║");
        println!("  ╚══════════════════════════════════════════╝");
        println!();
        Ok(())
    }

    fn on_task(&self, task: &ScheduledTask) -> Result<TaskAction> {
        let payload = task.payload();
        let data = payload.as_bytes();

        if data.is_empty() {
            return Ok(TaskAction::Pass);
        }

        let text = String::from_utf8_lossy(data);

        if text.contains("\"requires_human\":true") || text.contains("human_decision") {
            let answer = self
                .prompt(
                    &format!(
                        "Task from {}: {}\nApprove?",
                        task.source(),
                        &text[..text.len().min(120)]
                    ),
                    "Y",
                )
                .unwrap_or_else(|_| "Y".to_string());

            let approved = answer.to_uppercase().starts_with('Y');

            if let Some(ctx) = &self.ctx {
                let metrics = ctx.metrics.lock().unwrap();
                if approved {
                    metrics.increment("human.decisions.approved");
                } else {
                    metrics.increment("human.decisions.rejected");
                }
            }

            if approved {
                println!("  ✓ Human approved task {}", task.id());
                Ok(TaskAction::Pass)
            } else {
                println!("  ✗ Human rejected task {}", task.id());
                Ok(TaskAction::Cancel("Rejected by human operator".into()))
            }
        } else {
            Ok(TaskAction::Pass)
        }
    }

    fn on_shutdown(&self) -> Result<()> {
        println!("  Local Human Plugin shutting down");
        Ok(())
    }
}
