use std::{env, process::Command};

pub fn run(command: &str) -> Result<String, String> {
    if approve_exec_command(command) {
        tracing::debug!(command, "execute snippet  content");
        match Command::new("sh").arg("-c").arg(command).output() {
            Ok(output) => Ok(String::from_utf8_lossy(&output.stdout).to_string()),
            Err(e) => Err(e.to_string()),
        }
    } else {
        Err("command not approved".to_string())
    }
}
fn approve_exec_command(command: &str) -> bool {
    if env::var("SNIPDOC_SKIP_EXEC_COMMANDS").map_or(false, |val| val == "true") {
        true
    } else {
        let question = requestty::Question::confirm("confirm")
            .message(format!(
                "Security Warning: Snipdoc is about to execute the following command: \
                 `{command}`. Do you approve?"
            ))
            .build();

        match requestty::prompt_one(question) {
            Ok(answer) => answer.as_bool().is_some_and(|a| a),
            Err(err) => {
                tracing::debug!(err = %err, "prompt error");
                false
            }
        }
    }
}
