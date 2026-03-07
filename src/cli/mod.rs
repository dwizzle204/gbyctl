//! CLI parsing and request dispatch.

pub mod commands;

use std::collections::BTreeMap;
use std::io::{self, BufRead, IsTerminal, Write};
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{CommandFactory, Parser};
use serde::Serialize;

use crate::audit;
use crate::cli::commands::{Commands, DiagnoseCommands, InstallCommands};
use crate::config::{self, ModelConfig, ProviderKind};
use crate::exec::runner::{self, CommandResult};
use crate::inspect;
use crate::intent;
use crate::intent::llm_classifier::{Classification, ClassifyError};
use crate::plan;
use crate::policy::checklist::{self, ChecklistResult};
use crate::skills::types::{IntentMatch, PolicyClass, SkillId};
use crate::state::cache::{self, LocalState};
use crate::verify;

/// Top-level CLI input.
#[derive(Debug, Parser)]
#[command(name = "gbyctl")]
#[command(
    about = "Ubuntu-focused Linux operations assistant",
    long_about = "Accepts free-form Linux operations requests (e.g., \"disk is full\") or explicit subcommands; plan-first UX is the default.",
    after_help = "Examples:\n  gbyctl \"disk is full\"\n  gbyctl \"install tomcat\"\n  gbyctl --status\n\nTips:\n  --plan     Preview only (no execution)\n  --yes      Execute without interactive approval\n  --json     Machine-readable output\n  --version  Show current gbyctl version",
    version = env!("CARGO_PKG_VERSION")
)]
pub struct Cli {
    /// Optional natural-language request.
    #[arg(value_name = "REQUEST")]
    pub request: Option<String>,

    /// Explicit subcommand mode.
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Plan-only mode.
    #[arg(long = "plan")]
    pub plan_only: bool,

    /// Auto-approve `approval_required` commands.
    #[arg(long)]
    pub yes: bool,

    /// Emit machine-readable output.
    #[arg(long)]
    pub json: bool,

    /// Verbose mode.
    #[arg(long)]
    pub verbose: bool,

    /// Disable color output.
    #[arg(long = "no-color")]
    pub no_color: bool,

    /// Show model/config connectivity status.
    #[arg(long)]
    pub status: bool,
}

#[derive(Debug, Serialize)]
struct JsonResponse {
    mode: String,
    intent: Option<String>,
    message: String,
}

enum ResolveResult {
    Intent {
        request: String,
        intent: IntentMatch,
    },
    Immediate {
        mode: &'static str,
        intent: Option<String>,
        message: String,
    },
}

/// Dispatch user input through plan and execution flow.
pub fn dispatch(cli: Cli) -> Result<()> {
    // Process-level safety invariant: the assistant itself must not run as root.
    if is_running_as_root() {
        return output(
            &cli,
            "refusal",
            None,
            "gbyctl must not be run as root. Run as a regular user; commands may invoke sudo when required.",
        );
    }

    if cli.status {
        return render_status(&cli);
    }

    if cli.request.is_none() && cli.command.is_none() {
        return render_no_input_guidance(&cli);
    }

    if matches!(cli.command, Some(Commands::Setup)) {
        run_setup_interactive(None)?;
        return output(
            &cli,
            "setup",
            None,
            "Setup complete. Configuration and key stored.",
        );
    }

    let mut state = load_state()?;

    let os = inspect::os::detect().context("failed Ubuntu detection")?;
    state.ubuntu_version = Some(os.pretty_name.clone());
    store_state(&state)?;

    if !os.is_ubuntu {
        return output(
            &cli,
            "refusal",
            None,
            "This tool supports Ubuntu only in v1.",
        );
    }

    let resolved = resolve_input(&cli).context("failed to resolve request")?;
    let (request, intent) = match resolved {
        ResolveResult::Intent { request, intent } => (request, intent),
        ResolveResult::Immediate {
            mode,
            intent,
            message,
        } => return output(&cli, mode, intent.as_deref(), &message),
    };

    if let Some(question) = intent::clarify::required_question(&intent) {
        return output(
            &cli,
            "clarification",
            Some(intent.skill_id.as_str()),
            &question.prompt,
        );
    }

    let built_plan = plan::planner::build(&request, &intent).context("failed to build plan")?;

    if let Some(reason) = built_plan.refusal_reason.as_deref() {
        return output(&cli, "refusal", Some(intent.skill_id.as_str()), reason);
    }

    if cli.plan_only {
        return render_plan(&cli, &built_plan, "plan-only");
    }

    // Preview-first UX: show plan and require explicit confirmation before execution.
    let execute_approved = if cli.yes {
        true
    } else if io::stdin().is_terminal() {
        render_plan(&cli, &built_plan, "preview")?;
        prompt_yes_no("Execute this plan now? [y/N]", false)?
    } else {
        false
    };

    if !execute_approved {
        return output(
            &cli,
            "plan-only",
            Some(intent.skill_id.as_str()),
            "Execution skipped. Re-run with --yes to execute non-interactively.",
        );
    }

    let mut results: Vec<CommandResult> = Vec::new();
    let mut saw_manual = false;

    for step in &built_plan.steps {
        match checklist::evaluate(step) {
            ChecklistResult::Allow => {}
            ChecklistResult::Block { reason } => {
                return output(
                    &cli,
                    "refusal",
                    Some(intent.skill_id.as_str()),
                    &format!("Security checklist blocked execution: {reason}"),
                );
            }
        }

        match step.policy_class {
            PolicyClass::SafeExecute => {
                if !cli.json {
                    emit_line(&format!("STEP {}: {}", step.id, step.command.summary))?;
                    emit_line(&format!("CMD: {}", step.command.command))?;
                }
                let result = run_step_with_permission_recovery(
                    &cli,
                    step.id.as_str(),
                    &step.command.command,
                    step.command.modifies_state,
                )?;
                if !cli.json {
                    render_curated_step_result(step.id.as_str(), &result)?;
                }
                results.push(result);
            }
            PolicyClass::ApprovalRequired => {
                if !cli.json {
                    emit_line(&format!("STEP {}: {}", step.id, step.command.summary))?;
                    emit_line(&format!("CMD: {}", step.command.command))?;
                }
                let result = run_command_for_cli(&cli, &step.command.command)?;
                if !cli.json {
                    render_curated_step_result(step.id.as_str(), &result)?;
                }
                results.push(result);
            }
            PolicyClass::ManualOnly => {
                saw_manual = true;
                if !cli.json {
                    emit_line(&format!("MANUAL-ONLY: {}", step.command.command))?;
                }
            }
            PolicyClass::Forbidden => {
                return output(
                    &cli,
                    "refusal",
                    Some(intent.skill_id.as_str()),
                    "A generated step was forbidden by policy.",
                );
            }
        }
    }

    let summary = verify::summary::build(&built_plan, &results);
    let mode = if saw_manual { "manual-only" } else { "execute" };
    let highest = highest_policy(&built_plan.steps);

    let audit_dir = default_state_dir()?.join("sessions");
    let _path = audit::session::persist(&audit_dir, &built_plan, &summary.outcome, highest)?;

    output(&cli, mode, Some(intent.skill_id.as_str()), &summary.details)
}

fn maybe_configuration(cli: &Cli) -> Result<Option<ModelConfig>> {
    if let Some(cfg) = config::load()? {
        if cfg.has_api_key()? {
            return Ok(Some(cfg));
        }
        // In non-interactive contexts, skip setup and allow deterministic fallback.
        if !io::stdin().is_terminal() {
            return Ok(None);
        }
        emit_line("API key is missing from secure storage; starting setup.")?;
        run_setup_interactive(Some(cfg.clone()))?;
        let loaded = config::load()?.context("configuration missing after setup")?;
        return Ok(Some(loaded));
    }

    // First-run setup is interactive by design. Non-interactive callers should
    // provide explicit subcommands or rely on deterministic fallback behavior.
    if !io::stdin().is_terminal() {
        return Ok(None);
    }

    emit_line("No model configuration found. Starting first-run setup.")?;
    run_setup_interactive(None)?;
    let loaded = config::load()?.context("configuration missing after setup")?;
    output(
        cli,
        "setup",
        None,
        "Setup complete. Running requested command now.",
    )?;
    Ok(Some(loaded))
}

fn resolve_input(cli: &Cli) -> Result<ResolveResult> {
    if let Some(command) = &cli.command {
        let (request, intent) = intent_from_command(command);
        return Ok(ResolveResult::Intent { request, intent });
    }

    let request = cli
        .request
        .as_ref()
        .cloned()
        .context("request text or subcommand is required")?;

    if let Some(intent) = intent::llm_classifier::fallback_classify(&request)? {
        Ok(ResolveResult::Intent { request, intent })
    } else {
        // Only consult provider-backed classification when local routing has no answer.
        let cfg = maybe_configuration(cli)?;
        if let Some(cfg) = cfg.as_ref() {
            classify_natural_request(cli, cfg, &request)
        } else {
            Ok(ResolveResult::Immediate {
                mode: "out_of_scope",
                intent: None,
                message: "Request did not match supported Ubuntu operations skills.".to_owned(),
            })
        }
    }
}

fn classify_natural_request(cli: &Cli, cfg: &ModelConfig, request: &str) -> Result<ResolveResult> {
    let api_key = cfg
        .read_api_key()
        .context("failed reading provider API key from secure storage")?;

    match intent::llm_classifier::classify(request, cfg, &api_key) {
        Ok(Classification::Intent(intent)) => Ok(ResolveResult::Intent {
            request: request.to_owned(),
            intent,
        }),
        Ok(Classification::Clarification(message)) => Ok(ResolveResult::Immediate {
            mode: "clarification",
            intent: None,
            message,
        }),
        Ok(Classification::Refusal(message)) => Ok(ResolveResult::Immediate {
            mode: "refusal",
            intent: None,
            message,
        }),
        Ok(Classification::OutOfScope(message)) => Ok(ResolveResult::Immediate {
            mode: "out_of_scope",
            intent: None,
            message,
        }),
        Err(ClassifyError::AuthFailed) => {
            // Expired/revoked keys are recoverable; offer in-flow reconfiguration.
            emit_line("Provider authentication failed. API key may be expired or invalid.")?;
            if prompt_yes_no("Reconfigure provider key now? [Y/n]", true)? {
                run_setup_interactive(Some(cfg.clone()))?;
                let refreshed =
                    config::load()?.context("configuration missing after reconfigure")?;
                let refreshed_key = refreshed
                    .read_api_key()
                    .context("failed reading refreshed API key")?;
                return match intent::llm_classifier::classify(request, &refreshed, &refreshed_key) {
                    Ok(Classification::Intent(intent)) => Ok(ResolveResult::Intent {
                        request: request.to_owned(),
                        intent,
                    }),
                    Ok(Classification::Clarification(message)) => Ok(ResolveResult::Immediate {
                        mode: "clarification",
                        intent: None,
                        message,
                    }),
                    Ok(Classification::Refusal(message)) => Ok(ResolveResult::Immediate {
                        mode: "refusal",
                        intent: None,
                        message,
                    }),
                    Ok(Classification::OutOfScope(message)) => Ok(ResolveResult::Immediate {
                        mode: "out_of_scope",
                        intent: None,
                        message,
                    }),
                    Err(err) => Err(anyhow::anyhow!(
                        "classifier failed after reconfigure: {err}"
                    )),
                };
            }

            Ok(ResolveResult::Immediate {
                mode: "refusal",
                intent: None,
                message:
                    "Request cannot be classified until provider key is valid. Run `gbyctl setup`."
                        .to_owned(),
            })
        }
        Err(ClassifyError::Other(err)) => {
            // Provider/runtime/parser failures should not block supported local usage.
            // Fall back to deterministic router for bounded skills.
            if cli.verbose {
                emit_line(&format!(
                    "LLM classifier failed: {err}; falling back to deterministic router."
                ))?;
            }
            if let Some(intent) = intent::llm_classifier::fallback_classify(request)? {
                Ok(ResolveResult::Intent {
                    request: request.to_owned(),
                    intent,
                })
            } else {
                Ok(ResolveResult::Immediate {
                    mode: "out_of_scope",
                    intent: None,
                    message: "Request did not match supported Ubuntu operations skills.".to_owned(),
                })
            }
        }
    }
}

fn run_setup_interactive(existing: Option<ModelConfig>) -> Result<()> {
    let existing_provider = existing.as_ref().map(|cfg| cfg.provider.clone());

    emit_line("Model setup")?;
    emit_line("1) OpenAI-compatible")?;
    emit_line("2) Claude")?;

    let provider_choice = prompt_line("Select provider [1/2]", Some("1"))?;
    let provider = if provider_choice.trim() == "2" {
        ProviderKind::Claude
    } else if let Some(provider) = existing_provider {
        provider
    } else {
        ProviderKind::OpenAiCompatible
    };

    let default_base = match provider {
        ProviderKind::OpenAiCompatible => "https://api.openai.com/v1",
        ProviderKind::Claude => "https://api.anthropic.com/v1",
    };
    let default_model = match provider {
        ProviderKind::OpenAiCompatible => "gpt-4.1-mini",
        ProviderKind::Claude => "claude-3-5-sonnet-latest",
    };

    let base_url = prompt_line("Base URL", Some(default_base))?;
    let model = prompt_line("Model", Some(default_model))?;
    let api_key = prompt_line("API key", None)?;

    let api_key_id = "default".to_owned();

    let cfg = ModelConfig {
        provider,
        base_url,
        model,
        api_key_id,
    };

    config::store(&cfg)?;
    cfg.write_api_key(&api_key)?;

    emit_line("Configuration saved.")
}

fn prompt_line(prompt: &str, default: Option<&str>) -> Result<String> {
    let mut stdout = io::stdout().lock();
    if let Some(default) = default {
        stdout
            .write_all(format!("{prompt} [{default}]: ").as_bytes())
            .context("failed writing prompt")?;
    } else {
        stdout
            .write_all(format!("{prompt}: ").as_bytes())
            .context("failed writing prompt")?;
    }
    stdout.flush().context("failed flushing prompt")?;

    let mut input = String::new();
    let mut stdin = io::stdin().lock();
    let _bytes_read = stdin
        .read_line(&mut input)
        .context("failed reading prompt input")?;
    let trimmed = input.trim().to_owned();

    if trimmed.is_empty()
        && let Some(default) = default
    {
        return Ok(default.to_owned());
    }

    Ok(trimmed)
}

fn prompt_yes_no(prompt: &str, default_yes: bool) -> Result<bool> {
    let default = if default_yes { "y" } else { "n" };
    let answer = prompt_line(prompt, Some(default))?;
    let lowered = answer.to_ascii_lowercase();
    if lowered.is_empty() {
        return Ok(default_yes);
    }
    if lowered == "y" || lowered == "yes" {
        return Ok(true);
    }
    if lowered == "n" || lowered == "no" {
        return Ok(false);
    }
    Ok(default_yes)
}

fn intent_from_command(command: &Commands) -> (String, IntentMatch) {
    let high = crate::skills::types::IntentConfidence::High;
    match command {
        Commands::Setup => (
            "setup".to_owned(),
            build_intent(SkillId::Doctor, BTreeMap::new(), high),
        ),
        Commands::Doctor => (
            "doctor".to_owned(),
            build_intent(SkillId::Doctor, BTreeMap::new(), high),
        ),
        Commands::InspectStorage => (
            "inspect storage".to_owned(),
            build_intent(SkillId::InspectStorage, BTreeMap::new(), high),
        ),
        Commands::ServiceStatus(args) => {
            let mut map = BTreeMap::new();
            let _old = map.insert("service".to_owned(), args.name.clone());
            (
                format!("service status {}", args.name),
                build_intent(SkillId::ServiceStatus, map, high),
            )
        }
        Commands::PackageStatus(args) => {
            let mut map = BTreeMap::new();
            let _old = map.insert("package".to_owned(), args.name.clone());
            (
                format!("package status {}", args.name),
                build_intent(SkillId::PackageStatus, map, high),
            )
        }
        Commands::Install { command } => match command {
            InstallCommands::Package { name } => {
                let mut map = BTreeMap::new();
                let _old = map.insert("package".to_owned(), name.clone());
                (
                    format!("install {name}"),
                    build_intent(SkillId::InstallPackage, map, high),
                )
            }
            InstallCommands::Tomcat => (
                "install tomcat".to_owned(),
                build_intent(
                    SkillId::InstallPackage,
                    BTreeMap::from([("package".to_owned(), "tomcat10".to_owned())]),
                    high,
                ),
            ),
        },
        Commands::TroubleshootFirewall(args) => {
            let mut map = BTreeMap::new();
            if let Some(port) = args.port {
                let _old = map.insert("port".to_owned(), port.to_string());
            }
            (
                "troubleshoot firewall".to_owned(),
                build_intent(SkillId::TroubleshootFirewall, map, high),
            )
        }
        Commands::Diagnose { command } => match command {
            DiagnoseCommands::Reboot => (
                "diagnose reboot".to_owned(),
                build_intent(SkillId::DiagnoseRebootOrKernelIssue, BTreeMap::new(), high),
            ),
        },
        Commands::Logs(args) => {
            let mut map = BTreeMap::new();
            if let Some(service) = &args.service {
                let _old = map.insert("service".to_owned(), service.clone());
            }
            (
                "logs".to_owned(),
                build_intent(SkillId::LogsGuidance, map, high),
            )
        }
        Commands::Maintenance => (
            "maintenance".to_owned(),
            build_intent(SkillId::MaintenanceGuidance, BTreeMap::new(), high),
        ),
        Commands::ResizeRoot(_args) => (
            "resize root".to_owned(),
            build_intent(SkillId::ResizeRootPlan, BTreeMap::new(), high),
        ),
    }
}

fn build_intent(
    skill_id: SkillId,
    args: BTreeMap<String, String>,
    confidence: crate::skills::types::IntentConfidence,
) -> IntentMatch {
    IntentMatch {
        skill_id,
        confidence,
        args,
    }
}

fn render_plan(cli: &Cli, plan: &crate::skills::types::Plan, mode: &str) -> Result<()> {
    if cli.json {
        return output(cli, mode, Some(plan.skill_id.as_str()), "Plan rendered");
    }

    emit_line(&format!("INTENT: {}", plan.skill_id.as_str()))?;
    for step in &plan.steps {
        emit_line(&format!(
            "PLAN {} [{}]: {} -> {}",
            step.id,
            policy_label(step.policy_class),
            step.command.summary,
            step.command.command
        ))?;
    }
    if !plan.manual_guidance.is_empty() {
        for item in &plan.manual_guidance {
            emit_line(&format!("NOTE: {item}"))?;
        }
    }
    output(cli, mode, Some(plan.skill_id.as_str()), "Plan rendered")
}

fn policy_label(policy: PolicyClass) -> &'static str {
    match policy {
        PolicyClass::SafeExecute => "safe_execute",
        PolicyClass::ApprovalRequired => "approval_required",
        PolicyClass::ManualOnly => "manual_only",
        PolicyClass::Forbidden => "forbidden",
    }
}

fn highest_policy(steps: &[crate::skills::types::PlanStep]) -> PolicyClass {
    let mut rank = 0_u8;
    for step in steps {
        let step_rank = match step.policy_class {
            PolicyClass::SafeExecute => 0_u8,
            PolicyClass::ApprovalRequired => 1_u8,
            PolicyClass::ManualOnly => 2_u8,
            PolicyClass::Forbidden => 3_u8,
        };
        if step_rank > rank {
            rank = step_rank;
        }
    }
    match rank {
        3 => PolicyClass::Forbidden,
        2 => PolicyClass::ManualOnly,
        1 => PolicyClass::ApprovalRequired,
        _ => PolicyClass::SafeExecute,
    }
}

fn output(cli: &Cli, mode: &str, intent: Option<&str>, message: &str) -> Result<()> {
    if cli.json {
        let payload = JsonResponse {
            mode: mode.to_owned(),
            intent: intent.map(str::to_owned),
            message: message.to_owned(),
        };
        let raw = serde_json::to_string_pretty(&payload).context("failed to encode json output")?;
        emit_line(&raw)
    } else {
        if let Some(intent) = intent {
            emit_line(&format!("MODE: {mode}"))?;
            emit_line(&format!("INTENT: {intent}"))?;
        }
        emit_line(message)
    }
}

fn render_no_input_guidance(cli: &Cli) -> Result<()> {
    if !cli.json {
        return emit_line(&render_help_text());
    }

    let message = concat!(
        "Give Gibby a Linux operations request in plain language or use an explicit subcommand.\n",
        "Examples:\n",
        "  gbyctl \"disk is full\"\n",
        "  gbyctl \"why is my server slow\"\n",
        "  gbyctl \"install nginx\"\n",
        "  gbyctl doctor\n",
        "Tips:\n",
        "  - add --plan to preview without executing\n",
        "  - add --json for machine-readable output\n",
        "  - run --help to see all commands"
    );
    output(cli, "help", None, message)
}

fn render_status(cli: &Cli) -> Result<()> {
    let cfg_opt = config::load()?;
    let Some(cfg) = cfg_opt else {
        return output(
            cli,
            "status",
            None,
            "LLM status: not configured\nRun `gbyctl setup` to configure provider/model/API key.",
        );
    };

    if !cfg.has_api_key()? {
        return output(
            cli,
            "status",
            None,
            &format!(
                "LLM status: key missing\nProvider: {}\nModel: {}\nAction: run `gbyctl setup` to set or replace the API key.",
                provider_label(&cfg.provider),
                cfg.model
            ),
        );
    }

    let api_key = cfg.read_api_key()?;
    let (status, note): (&str, String) =
        match intent::llm_classifier::connectivity_probe(&cfg, &api_key) {
            Ok(()) => ("connected", "Connectivity probe succeeded.".to_owned()),
            Err(ClassifyError::AuthFailed) => (
                "auth_failed",
                "Provider rejected credentials (401/403). Run `gbyctl setup` to rotate the key."
                    .to_owned(),
            ),
            Err(ClassifyError::Other(err)) => (
                "unreachable",
                // Compact status messaging: include provider error while avoiding credential leakage.
                format!("Probe failed: {err}"),
            ),
        };

    output(
        cli,
        "status",
        None,
        &format!(
            "LLM status: {status}\nProvider: {}\nModel: {}\nBase URL: {}\nNote: {note}",
            provider_label(&cfg.provider),
            cfg.model,
            cfg.base_url
        ),
    )
}

fn provider_label(provider: &ProviderKind) -> &'static str {
    match provider {
        ProviderKind::OpenAiCompatible => "openai-compatible",
        ProviderKind::Claude => "claude",
    }
}

fn render_help_text() -> String {
    let mut command = Cli::command();
    let mut bytes = Vec::new();
    if command.write_long_help(&mut bytes).is_err() {
        return "Run `gbyctl --help` for usage.".to_owned();
    }
    String::from_utf8_lossy(&bytes).into_owned()
}

fn run_command_for_cli(cli: &Cli, command: &str) -> Result<CommandResult> {
    if cli.json {
        runner::run_quiet(command)
    } else {
        runner::run_streaming(command)
    }
}

fn run_safe_command_for_cli(cli: &Cli, command: &str) -> Result<CommandResult> {
    if cli.json {
        runner::run_quiet(command)
    } else {
        // Curated output is rendered after completion for safe/read-only steps.
        runner::run_quiet(command)
    }
}

fn run_step_with_permission_recovery(
    cli: &Cli,
    step_id: &str,
    command: &str,
    modifies_state: bool,
) -> Result<CommandResult> {
    let result = run_safe_command_for_cli(cli, command)?;
    if should_offer_sudo_retry(cli, command, modifies_state, &result) {
        emit_line(&format!(
            "Detected permission error on step `{step_id}`. Retry with sudo?"
        ))?;
        if prompt_yes_no("Retry with sudo now? [y/N]", false)? {
            let sudo_command = format!("sudo {command}");
            if !cli.json {
                emit_line(&format!("CMD (retry): {sudo_command}"))?;
            }
            return run_command_for_cli(cli, &sudo_command);
        }
    }
    Ok(result)
}

fn should_offer_sudo_retry(
    cli: &Cli,
    command: &str,
    modifies_state: bool,
    result: &CommandResult,
) -> bool {
    if result.exit_code == 0 || cli.json || !io::stdin().is_terminal() {
        return false;
    }
    if modifies_state {
        return false;
    }
    if command.trim_start().starts_with("sudo ") {
        return false;
    }
    looks_like_permission_error(&result.output)
}

fn looks_like_permission_error(output: &str) -> bool {
    let text = output.to_ascii_lowercase();
    let patterns = [
        "permission denied",
        "operation not permitted",
        "must be root",
        "need to be root",
        "authentication is required",
    ];
    patterns.iter().any(|pattern| text.contains(pattern))
}

fn render_curated_step_result(step_id: &str, result: &CommandResult) -> Result<()> {
    let status = if result.exit_code == 0 {
        "success"
    } else {
        "failed"
    };
    emit_line(&format!(
        "RESULT {}: {} (exit {})",
        step_id, status, result.exit_code
    ))?;

    let lines = curated_output_lines(&result.output, 6);
    if lines.is_empty() {
        return emit_line("DETAIL: command returned no output.");
    }

    let label = if result.exit_code == 0 {
        "DETAIL"
    } else {
        "ERROR"
    };
    emit_line(&format!("{label}:"))?;
    for line in lines {
        emit_line(&format!("  {line}"))?;
    }
    Ok(())
}

fn curated_output_lines(raw: &str, max_lines: usize) -> Vec<String> {
    raw.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .take(max_lines)
        .map(str::to_owned)
        .collect()
}

fn emit_line(line: &str) -> Result<()> {
    let mut stdout = io::stdout().lock();
    stdout
        .write_all(format!("{line}\n").as_bytes())
        .context("failed writing output")?;
    stdout.flush().context("failed flushing output")?;
    Ok(())
}

fn default_state_dir() -> Result<PathBuf> {
    let home = std::env::var("HOME").context("HOME env is not set")?;
    Ok(PathBuf::from(home).join(".local/state/gbyctl"))
}

fn load_state() -> Result<LocalState> {
    let path = default_state_dir()?.join("state.json");
    cache::load(&path)
}

fn store_state(state: &LocalState) -> Result<()> {
    let path = default_state_dir()?.join("state.json");
    cache::store(&path, state)
}

fn is_running_as_root() -> bool {
    if let Ok(output) = std::process::Command::new("id").arg("-u").output()
        && output.status.success()
        && let Ok(uid) = String::from_utf8(output.stdout)
    {
        return uid.trim() == "0";
    }
    false
}

#[cfg(test)]
mod tests {
    use super::{curated_output_lines, looks_like_permission_error};

    #[test]
    fn detects_permission_denied_signals() {
        assert!(looks_like_permission_error("journalctl: permission denied"));
        assert!(looks_like_permission_error(
            "Error: authentication is required to perform this operation"
        ));
        assert!(looks_like_permission_error("operation not permitted"));
    }

    #[test]
    fn ignores_unrelated_failures() {
        assert!(!looks_like_permission_error("connection timed out"));
        assert!(!looks_like_permission_error("no such file or directory"));
    }

    #[test]
    fn curates_output_to_non_empty_bounded_lines() {
        let lines = curated_output_lines("line1\n\n line2 \nline3\nline4\nline5\nline6\nline7", 6);
        assert_eq!(lines.len(), 6);
        assert_eq!(lines[0], "line1");
        assert_eq!(lines[1], "line2");
        assert_eq!(lines[5], "line6");
    }
}
