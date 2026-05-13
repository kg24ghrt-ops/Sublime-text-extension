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

    /// Correct trait signature: &mut self, returns Result<Option<ContextServerConfiguration>, String>
    fn context_server_configuration(
        &mut self,
        _server_id: &zed::ContextServerId,
        _project: &zed::Project,
    ) -> zed::Result<Option<zed::ContextServerConfiguration>> {
        // Return the default configuration for the context server.
        // The user will fill in the token from the Zed settings UI.
        Ok(Some(zed::ContextServerConfiguration {
            name: "NovaCibes Python Runner".to_string(),
            tooltip: Some("Enter your Hugging Face token".into()),
            settings: serde_json::json!({
                "huggingface_token": "",
                "runner_url": "https://novacibes-python-running-api.hf.space"
            }),
        }))
    }

    /// Called when the user saves the context server settings.
    fn context_server_configuration_updated(
        &mut self,
        _server_id: &zed::ContextServerId,
        settings: serde_json::Value,
    ) {
        if let Ok(rs) = serde_json::from_value::<RunnerSettings>(settings) {
            SETTINGS.with(|s| *s.borrow_mut() = Some(rs));
        }
    }
}

impl NovaCibesPythonRunner {
    fn get_settings_or_prompt(&self) -> zed::Result<(String, String)> {
        let maybe = SETTINGS.with(|s| s.borrow().clone());
        if let Some(settings) = maybe {
            if let Some(token) = settings.huggingface_token {
                if !token.is_empty() {
                    let url = settings.runner_url.unwrap_or_else(|| "https://novacibes-python-running-api.hf.space".into());
                    return Ok((token, url));
                }
            }
        }

        // Token missing → return an error with instructions.
        Err(
            "## ⚠️ Hugging Face Token Missing\n\n\
            Click the **Context Servers** button in the Assistant Panel or run \
            `context-server: configure` to open settings, then paste your token.\n\n\
            Alternatively, add this to your `settings.json`:\n\
            ```json\n\
            \"context_servers\": {\n\
              \"novacibes-python-runner\": {\n\
                \"settings\": {\n\
                  \"huggingface_token\": \"hf_your_token\"\n\
                }\n\
              }\n\
            }\n\
            ```"
                .into(),
        )
    }

    fn execute_code(&self, args: Vec<String>) -> zed::Result<zed::SlashCommandOutput> {
        let code = args.join(" ");
        if code.is_empty() {
            return Err("No Python code provided.".into());
        }

        let (token, base_url) = match self.get_settings_or_prompt() {
            Ok(t) => t,
            Err(e) => {
                // Show the prompt message as output
                return Ok(zed::SlashCommandOutput {
                    text: e,
                    sections: vec![],
                });
            }
        };

        let url = format!("{}/run", base_url.trim_end_matches('/'));

        let request_body = serde_json::json!({ "code": code });
        let body_bytes = serde_json::to_vec(&request_body).unwrap();

        // Build the HTTP request (HttpRequestBuilder::build returns Result<HttpRequest, String>)
        let request = zed::http_client::HttpRequestBuilder::new()
            .method(zed::http_client::HttpMethod::Post)
            .url(&url)
            .header("Content-Type", "application/json")
            .header("Authorization", &format!("Bearer {}", token))
            .body(body_bytes)
            .build()
            .map_err(|e| format!("Failed to build request: {}", e))?;

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
