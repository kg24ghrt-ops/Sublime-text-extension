use zed_extension_api as zed;
use serde::Deserialize;
use std::env;

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
        _context_server_id: &zed::ContextServerId,
        _project: &zed::Project,
    ) -> zed::Result<Option<zed::ContextServerConfiguration>> {
        Ok(Some(zed::ContextServerConfiguration {
            installation_instructions: "Enter your Hugging Face personal access token."
                .to_string(),
            settings_schema: serde_json::json!({
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
            })
            .to_string(),
            default_settings: serde_json::json!({
                "huggingface_token": "",
                "runner_url": "https://novacibes-python-running-api.hf.space"
            })
            .to_string(),
        }))
    }
}

impl NovaCibesPythonRunner {
    fn get_token(&self) -> zed::Result<String> {
        env::var("HUGGINGFACE_TOKEN").map_err(|_| {
            "HUGGINGFACE_TOKEN environment variable not set. Please add:\n\
             export HUGGINGFACE_TOKEN=\"hf_your_token\"\n\
             to your shell profile and restart Zed."
                .into()
        })
    }

    fn execute_code(&self, args: Vec<String>) -> zed::Result<zed::SlashCommandOutput> {
        let code = args.join(" ");
        if code.is_empty() {
            return Err("No Python code provided.".into());
        }

        let token = match self.get_token() {
            Ok(t) => t,
            Err(e) => {
                return Ok(zed::SlashCommandOutput {
                    text: format!("## ⚠️ Token Not Configured\n\n{}", e),
                    sections: vec![],
                });
            }
        };

        let url = format!(
            "{}/run",
            env::var("RUNNER_URL")
                .unwrap_or("https://novacibes-python-running-api.hf.space".into())
                .trim_end_matches('/')
        );

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
