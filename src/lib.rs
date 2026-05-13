use zed_extension_api as zed;
use serde::Deserialize;

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

    fn context_server_configuration(
        &mut self,
        _server_id: &zed::ContextServerId,
        _project: &zed::Project,
    ) -> zed::Result<Option<zed::ContextServerConfiguration>> {
        Ok(Some(zed::ContextServerConfiguration {
            installation_instructions: Some(
                "Enter your Hugging Face personal access token.".into(),
            ),
            settings_schema: Some(serde_json::json!({
                "type": "object",
                "properties": {
                    "huggingface_token": {
                        "type": "string",
                        "description": "Your Hugging Face token"
                    },
                    "runner_url": {
                        "type": "string",
                        "description": "Base URL of the Python runner",
                        "default": "https://novacibes-python-running-api.hf.space"
                    }
                }
            })),
            default_settings: Some(serde_json::json!({
                "huggingface_token": "",
                "runner_url": "https://novacibes-python-running-api.hf.space"
            })),
        }))
    }
}

impl NovaCibesPythonRunner {
    /// Retrieve the token and URL from the context server settings.
    fn get_settings(&self) -> zed::Result<(String, String)> {
        let settings = zed::context_server_settings("novacibes-python-runner")
            .ok_or("Context server settings not found. Please configure the NovaCibes Python Runner in the settings (use 'context-server: configure').")?;
        let token = settings["huggingface_token"]
            .as_str()
            .filter(|t| !t.is_empty())
            .ok_or("Hugging Face token is empty. Please enter it in the context server settings.")?;
        let url = settings["runner_url"]
            .as_str()
            .unwrap_or("https://novacibes-python-running-api.hf.space");
        Ok((token.to_string(), url.to_string()))
    }

    fn execute_code(&self, args: Vec<String>) -> zed::Result<zed::SlashCommandOutput> {
        let code = args.join(" ");
        if code.is_empty() {
            return Err("No Python code provided.".into());
        }

        let (token, base_url) = match self.get_settings() {
            Ok(t) => t,
            Err(e) => {
                return Ok(zed::SlashCommandOutput {
                    text: format!("## ⚠️ Token Not Configured\n\n{}\n\nOpen the **Context Servers** panel (or run `context-server: configure`) and select **NovaCibes Python Runner** to set your token.", e),
                    sections: vec![],
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
            .build()
            .map_err(|e| format!("Failed to build HTTP request: {}", e))?;

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
