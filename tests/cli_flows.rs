use std::fs;
use std::process::Command;

use serde_json::Value;
use serde_json::json;
use tempfile::TempDir;
use tempfile::tempdir;

fn isolated_command() -> Option<(TempDir, Command)> {
    let temp = TempDir::new().ok()?;
    let config_home = temp.path().join("config");
    let state_home = temp.path().join("state");
    fs::create_dir_all(&config_home).ok()?;
    fs::create_dir_all(&state_home).ok()?;

    let mut command = Command::new(env!("CARGO_BIN_EXE_gbyctl"));
    let _command = command.env("HOME", temp.path());
    let _command = command.env("XDG_CONFIG_HOME", &config_home);
    let _command = command.env("XDG_STATE_HOME", &state_home);

    Some((temp, command))
}

fn isolated_command_with_config(config: Value) -> Option<(TempDir, Command)> {
    let (temp, command) = isolated_command()?;
    let config_dir = temp.path().join(".config/gbyctl");
    fs::create_dir_all(&config_dir).ok()?;
    let raw = serde_json::to_vec_pretty(&config).ok()?;
    fs::write(config_dir.join("config.json"), raw).ok()?;
    Some((temp, command))
}

#[test]
fn plan_mode_for_doctor_succeeds() {
    let command_result = isolated_command();
    assert!(command_result.is_some());
    let (_temp, mut command) = if let Some(values) = command_result {
        values
    } else {
        return;
    };
    let output_result = command.args(["--plan", "doctor"]).output();
    assert!(output_result.is_ok());
    let output = if let Ok(output) = output_result {
        output
    } else {
        return;
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if stderr.contains("must not be run as root") {
        assert!(!output.status.success());
        return;
    }

    assert!(output.status.success());
    assert!(stdout.contains("PLAN"));
}

#[test]
fn no_input_returns_guided_help() {
    let temp_home_result = tempdir();
    assert!(temp_home_result.is_ok());
    let temp_home = if let Ok(temp_home) = temp_home_result {
        temp_home
    } else {
        return;
    };

    let output_result = Command::new(env!("CARGO_BIN_EXE_gbyctl"))
        .env("HOME", temp_home.path())
        .current_dir(temp_home.path())
        .output();
    assert!(output_result.is_ok());
    let output = if let Ok(output) = output_result {
        output
    } else {
        return;
    };

    assert!(output.status.success());
    let stdout_result = String::from_utf8(output.stdout);
    assert!(stdout_result.is_ok());
    let stdout = if let Ok(stdout) = stdout_result {
        stdout
    } else {
        return;
    };
    assert!(stdout.contains("Usage: gbyctl"));
    assert!(stdout.contains("gbyctl \"disk is full\""));
    assert!(stdout.contains("--status"));
}

#[test]
fn no_input_matches_help_surface() {
    let command_result = isolated_command();
    assert!(command_result.is_some());
    let (_temp, mut no_args_cmd) = if let Some(values) = command_result {
        values
    } else {
        return;
    };
    let no_args_output_result = no_args_cmd.output();
    assert!(no_args_output_result.is_ok());
    let no_args_output = if let Ok(output) = no_args_output_result {
        output
    } else {
        return;
    };
    assert!(no_args_output.status.success());

    let command_result = isolated_command();
    assert!(command_result.is_some());
    let (_temp, mut help_cmd) = if let Some(values) = command_result {
        values
    } else {
        return;
    };
    let help_output_result = help_cmd.arg("--help").output();
    assert!(help_output_result.is_ok());
    let help_output = if let Ok(output) = help_output_result {
        output
    } else {
        return;
    };
    assert!(help_output.status.success());

    let no_args_stdout_result = String::from_utf8(no_args_output.stdout);
    assert!(no_args_stdout_result.is_ok());
    let no_args_stdout = if let Ok(stdout) = no_args_stdout_result {
        stdout
    } else {
        return;
    };
    let help_stdout_result = String::from_utf8(help_output.stdout);
    assert!(help_stdout_result.is_ok());
    let help_stdout = if let Ok(stdout) = help_stdout_result {
        stdout
    } else {
        return;
    };
    assert_eq!(no_args_stdout.trim_end(), help_stdout.trim_end());
}

#[test]
fn unquoted_multiword_request_routes_normally() {
    let command_result = isolated_command();
    assert!(command_result.is_some());
    let (_temp, mut command) = if let Some(values) = command_result {
        values
    } else {
        return;
    };

    let output_result = command
        .args(["show", "me", "disk", "usage", "--json", "--plan"])
        .output();
    assert!(output_result.is_ok());
    let output = if let Ok(output) = output_result {
        output
    } else {
        return;
    };

    let stderr = String::from_utf8_lossy(&output.stderr);
    if stderr.contains("must not be run as root") {
        assert!(!output.status.success());
        return;
    }

    assert!(output.status.success());
    let json = parse_json_output(&output);
    assert!(json.is_some());
    let json = if let Some(json) = json {
        json
    } else {
        return;
    };
    assert_eq!(json.get("mode").and_then(Value::as_str), Some("plan-only"));
    assert_eq!(
        json.get("intent").and_then(Value::as_str),
        Some("disk_full_triage")
    );
}

#[test]
fn natural_language_routes_to_disk_triage() {
    let command_result = isolated_command();
    assert!(command_result.is_some());
    let (_temp, mut command) = if let Some(values) = command_result {
        values
    } else {
        return;
    };
    let output_result = command.args(["my disk is full", "--plan"]).output();
    assert!(output_result.is_ok());
    let output = if let Ok(output) = output_result {
        output
    } else {
        return;
    };

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("disk_full_triage"));
}

#[test]
fn forbidden_shell_syntax_is_rejected() {
    let command_result = isolated_command();
    assert!(command_result.is_some());
    let (_temp, mut command) = if let Some(values) = command_result {
        values
    } else {
        return;
    };
    let output_result = command
        .args(["sudo apt-get update; rm -rf /", "--plan"])
        .output();
    assert!(output_result.is_ok());
    let output = if let Ok(output) = output_result {
        output
    } else {
        return;
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if stderr.contains("must not be run as root") {
        assert!(!output.status.success());
        return;
    }

    assert!(output.status.success());
    assert!(
        stdout.contains("Request did not match supported Ubuntu operations skills.")
            || stdout.contains("unsafe shell syntax")
            || stdout.contains("Forbidden")
    );
}

fn parse_json_output(output: &std::process::Output) -> Option<Value> {
    serde_json::from_slice::<Value>(&output.stdout).ok()
}

#[test]
fn json_plan_response_has_stable_shape() {
    let command_result = isolated_command();
    assert!(command_result.is_some());
    let (_temp, mut command) = if let Some(values) = command_result {
        values
    } else {
        return;
    };
    let output_result = command.args(["--json", "--plan", "doctor"]).output();
    assert!(output_result.is_ok());
    let output = if let Ok(output) = output_result {
        output
    } else {
        return;
    };

    let stderr = String::from_utf8_lossy(&output.stderr);
    if stderr.contains("must not be run as root") {
        assert!(!output.status.success());
        return;
    }

    assert!(output.status.success());
    let json = parse_json_output(&output);
    assert!(json.is_some());
    let json = if let Some(json) = json {
        json
    } else {
        return;
    };

    assert_eq!(json.get("mode").and_then(Value::as_str), Some("plan-only"));
    assert_eq!(json.get("intent").and_then(Value::as_str), Some("doctor"));
    assert_eq!(
        json.get("message").and_then(Value::as_str),
        Some("Plan rendered")
    );
}

#[test]
fn json_out_of_scope_response_has_stable_shape() {
    let command_result = isolated_command();
    assert!(command_result.is_some());
    let (_temp, mut command) = if let Some(values) = command_result {
        values
    } else {
        return;
    };
    let output_result = command
        .args(["write a python script", "--plan", "--json"])
        .output();
    assert!(output_result.is_ok());
    let output = if let Ok(output) = output_result {
        output
    } else {
        return;
    };

    let stderr = String::from_utf8_lossy(&output.stderr);
    if stderr.contains("must not be run as root") {
        assert!(!output.status.success());
        return;
    }

    assert!(output.status.success());
    let json = parse_json_output(&output);
    assert!(json.is_some());
    let json = if let Some(json) = json {
        json
    } else {
        return;
    };

    assert_eq!(
        json.get("mode").and_then(Value::as_str),
        Some("out_of_scope")
    );
    assert_eq!(json.get("intent").and_then(Value::as_str), None);
    assert_eq!(
        json.get("message").and_then(Value::as_str),
        Some("Request did not match supported Ubuntu operations skills.")
    );
}

#[test]
fn json_clarification_response_has_stable_shape() {
    let command_result = isolated_command();
    assert!(command_result.is_some());
    let (_temp, mut command) = if let Some(values) = command_result {
        values
    } else {
        return;
    };
    let output_result = command.args(["open port", "--plan", "--json"]).output();
    assert!(output_result.is_ok());
    let output = if let Ok(output) = output_result {
        output
    } else {
        return;
    };

    let stderr = String::from_utf8_lossy(&output.stderr);
    if stderr.contains("must not be run as root") {
        assert!(!output.status.success());
        return;
    }

    assert!(output.status.success());
    let json = parse_json_output(&output);
    assert!(json.is_some());
    let json = if let Some(json) = json {
        json
    } else {
        return;
    };

    assert_eq!(
        json.get("mode").and_then(Value::as_str),
        Some("clarification")
    );
    assert_eq!(
        json.get("intent").and_then(Value::as_str),
        Some("troubleshoot_firewall")
    );
    assert_eq!(
        json.get("message").and_then(Value::as_str),
        Some("Which port should be checked?")
    );
}

#[test]
fn json_execute_response_has_stable_shape() {
    let command_result = isolated_command();
    assert!(command_result.is_some());
    let (_temp, mut command) = if let Some(values) = command_result {
        values
    } else {
        return;
    };
    let output_result = command.args(["--json", "--yes", "doctor"]).output();
    assert!(output_result.is_ok());
    let output = if let Ok(output) = output_result {
        output
    } else {
        return;
    };

    let stderr = String::from_utf8_lossy(&output.stderr);
    if stderr.contains("must not be run as root") {
        assert!(!output.status.success());
        return;
    }

    assert!(output.status.success());
    let json = parse_json_output(&output);
    assert!(json.is_some());
    let json = if let Some(json) = json {
        json
    } else {
        return;
    };

    assert_eq!(json.get("mode").and_then(Value::as_str), Some("execute"));
    assert_eq!(json.get("intent").and_then(Value::as_str), Some("doctor"));
    assert_eq!(
        json.get("message").and_then(Value::as_str),
        Some("Skill doctor completed with 6 executed steps")
    );
}

#[test]
fn json_manual_only_response_has_stable_shape() {
    let command_result = isolated_command();
    assert!(command_result.is_some());
    let (_temp, mut command) = if let Some(values) = command_result {
        values
    } else {
        return;
    };
    let output_result = command.args(["--json", "--yes", "resize-root"]).output();
    assert!(output_result.is_ok());
    let output = if let Ok(output) = output_result {
        output
    } else {
        return;
    };

    let stderr = String::from_utf8_lossy(&output.stderr);
    if stderr.contains("must not be run as root") {
        assert!(!output.status.success());
        return;
    }

    assert!(output.status.success());
    let json = parse_json_output(&output);
    assert!(json.is_some());
    let json = if let Some(json) = json {
        json
    } else {
        return;
    };

    assert_eq!(
        json.get("mode").and_then(Value::as_str),
        Some("manual-only")
    );
    assert_eq!(
        json.get("intent").and_then(Value::as_str),
        Some("resize_root_plan")
    );
    assert_eq!(
        json.get("message").and_then(Value::as_str),
        Some("Skill resize_root_plan completed with 1 executed steps")
    );
}

#[test]
fn deterministic_request_bypasses_model_setup_and_key_lookup() {
    let config = json!({
        "provider": "OpenAiCompatible",
        "base_url": "https://api.openai.com/v1",
        "model": "gpt-4.1-mini",
        "api_key_id": "missing-key"
    });
    let command_result = isolated_command_with_config(config);
    assert!(command_result.is_some());
    let (_temp, mut command) = if let Some(values) = command_result {
        values
    } else {
        return;
    };
    let output_result = command
        .args(["my disk is full", "--plan", "--json"])
        .output();
    assert!(output_result.is_ok());
    let output = if let Ok(output) = output_result {
        output
    } else {
        return;
    };

    let stderr = String::from_utf8_lossy(&output.stderr);
    if stderr.contains("must not be run as root") {
        assert!(!output.status.success());
        return;
    }

    assert!(output.status.success());
    let json = parse_json_output(&output);
    assert!(json.is_some());
    let json = if let Some(json) = json {
        json
    } else {
        return;
    };
    assert_eq!(json.get("mode").and_then(Value::as_str), Some("plan-only"));
    assert_eq!(
        json.get("intent").and_then(Value::as_str),
        Some("disk_full_triage")
    );
}

#[test]
fn status_reports_not_configured_without_setup() {
    let command_result = isolated_command();
    assert!(command_result.is_some());
    let (_temp, mut command) = if let Some(values) = command_result {
        values
    } else {
        return;
    };
    let output_result = command.args(["--status"]).output();
    assert!(output_result.is_ok());
    let output = if let Ok(output) = output_result {
        output
    } else {
        return;
    };

    let stderr = String::from_utf8_lossy(&output.stderr);
    if stderr.contains("must not be run as root") {
        assert!(!output.status.success());
        return;
    }

    assert!(output.status.success());
    let stdout_result = String::from_utf8(output.stdout);
    assert!(stdout_result.is_ok());
    let stdout = if let Ok(stdout) = stdout_result {
        stdout
    } else {
        return;
    };
    assert!(stdout.contains("LLM status: not configured"));
    assert!(stdout.contains("gbyctl setup"));
}
