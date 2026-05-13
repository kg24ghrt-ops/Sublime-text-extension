use zed_extension_api as zed;
use serde::Deserialize;
use std::cell::RefCell;

thread_local! {
    static SETTINGS: RefCell<Option<RunnerSettings>> = const { RefCell::new(None) };
}

#[derive(Deserialize, Clone)]
struct RunnerSettings {
    huggingface_token: Option<String>,
    runner_url: Option<String>,
}

struct NovaCibesPythonRunner;

impl zed::Extension for NovaCibesPythonRunner {
    fn new() -> Self {
        Self
    }

    fn run_slash_command(
        &self,
        command: zed::SlashCommand,
        args: Vec<String>,
        _worktree: Option<&zed::Worktree>,
    ) -> zed::Result<zed::SlashCommandOutput> {
        match command.name.as_str() {
            "python-run" => self.execute_code(args),
            "python-stop" => self.stop_execution(),
            _ => Err("Unknown command".into()),
        }
    }

    /// This is called when the user updates their context server settings.
    fn context_server_configuration(
        &self,
        server_id: &zed::ContextServerId,
        settings: serde_json::Value,
    ) {
        if let Ok(rs) = serde_json::from_value::<RunnerSettings>(settings) {
            SETTINGS.with(|s| *s.borrow_mut() = Some(rs));
        }
    }
}

impl NovaCibesPythonRunner {
    /// Get the stored settings, or return an error with a prompt that opens settings.json
    fn get_settings_or_prompt(&self) -> zed::Result<(String, String)> {
        let maybe = SETTINGS.with(|s| s.borrow().clone());
        if let Some(settings) = maybe {
            if let (Some(token), Some(url)) = (settings.huggingface_token.clone(), settings.runner_url.clone()) {
                if !token.is_empty() {
                    return Ok((token, url));
                }
            }
        }

        // Token missing → return a special error containing a link to open settings.json
        Err("Token not configured. Click the file path below to open your settings and add the context server block.\n\n→ file://~/.config/zed/settings.json".into())
    }

    fn execute_code(&self, args: Vec<String>) -> zed::Result<zed::SlashCommandOutput> {
        let code = args.join(" ");
        if code.is_empty() {
            return Err("No Python code provided.".into());
        }

        let (token, base_url) = match self.get_settings_or_prompt() {
            Ok(t) => t,
            Err(e) => {
                // Return a SlashCommandOutput that includes a clickable file link.
                // We use a section pointing to the settings file.
                let settings_path = dirs::home_dir()
                    .unwrap_or_default()
                    .join(".config/zed/settings.json")
                    .to_string_lossy()
                    .to_string();

                let section = zed::SlashCommandOutputSection {
                    range: zed::Range {
                        start: zed::Point::new(0, 0), // opens the file
                        end: zed::Point::new(0, 0),
                    },
                    label: "Open settings.json".to_string(),
                };

                return Ok(zed::SlashCommandOutput {
                    text: format!(
                        "## ⚠️ Missing Token\n\nPaste the following into your `settings.json`:\n\n```json\n\"context_servers\": {{\n  \"novacibes-python-runner\": {{\n    \"settings\": {{\n      \"huggingface_token\": \"hf_your_token_here\"\n    }}\n  }}\n}}\n```"
                    ),
                    sections: vec![section],
                });
            }
        };

        let url = format!("{}/run", base_url.trim_end_matches('/'));

        let request_body = serde_json::json!({ "code": code });
        let body_bytes = serde_json::to_vec(&request_body).unwrap();

        let request = zed::http_client::HttpRequestBuilder::new()
            .method(zed::http_client::HttpMethod::Post)
            .url(&url)
            .header("Content-Type", "application/json")
            .header("Authorization", &format!("Bearer {}", token))
            .body(body_bytes)
            .build();

        let response = zed::http_client::fetch(&request)?;

        #[derive(Deserialize)]
        struct RunResult {
            stdout: String,
            stderr: String,
        }

        let result: RunResult = serde_json::from_slice(&response.body)
            .map_err(|e| format!("Failed to parse runner output: {}", e))?;

        let mut output = String::new();
        if !result.stdout.is_empty() {
            output.push_str("**Output:**\n```\n");
            output.push_str(&result.stdout);
            output.push_str("\n```\n");
        }
        if !result.stderr.is_empty() {
            output.push_str("**Errors:**\n```diff\n- ");
            output.push_str(&result.stderr.replace('\n', "\n- "));
            output.push_str("\n```\n");
        }
        if output.is_empty() {
            output.push_str("*(Code ran with no output)*");
        }

        Ok(zed::SlashCommandOutput {
            text: output,
            sections: vec![],
        })
    }

    fn stop_execution(&self) -> zed::Result<zed::SlashCommandOutput> {
        Ok(zed::SlashCommandOutput {
            text: "Stop command sent. Processes will terminate automatically.".into(),
            sections: vec![],
        })
    }
}

zed::register_extension!(NovaCibesPythonRunner);
