use zed_extension_api as zed;
use serde::Deserialize;

struct NovaCibesPythonRunner;

#[derive(Deserialize)]
struct ContextSettings {
    huggingface_token: Option<String>,
    runner_url: Option<String>,
}

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

    // This returns the default settings shown to users in the context server config UI
    fn context_server_settings(
        &self,
        _server_id: &zed::ContextServerId,
    ) -> zed::Result<serde_json::Value> {
        Ok(serde_json::json!({
            "huggingface_token": "",
            "runner_url": "https://novacibes-python-running-api.hf.space"
        }))
    }
}

impl NovaCibesPythonRunner {
    /// Read the actual user settings from Zed’s settings store
    fn get_settings(&self) -> zed::Result<(String, String)> {
        let config = zed::settings::get_context_server_settings("novacibes-python-runner")
            .ok_or("Extension settings not found. Please add 'novacibes-python-runner' to your context_servers in settings.json.")?;
        let settings: ContextSettings = serde_json::from_value(config)
            .map_err(|e| format!("Invalid settings: {}", e))?;
        let token = settings.huggingface_token
            .filter(|t| !t.is_empty())
            .ok_or("Hugging Face token missing. Set it in Settings → Context Servers → NovaCibes Python Runner.")?;
        let url = settings.runner_url.unwrap_or_else(|| "https://novacibes-python-running-api.hf.space".to_string());
        Ok((token, url))
    }

    fn execute_code(&self, args: Vec<String>) -> zed::Result<zed::SlashCommandOutput> {
        let code = args.join(" ");
        if code.is_empty() {
            return Err("No Python code provided.".into());
        }

        let (token, base_url) = self.get_settings()?;
        let url = format!("{}/run", base_url.trim_end_matches('/'));

        let request_body = serde_json::json!({ "code": code });
        let body = serde_json::to_string(&request_body).unwrap();

        let response = zed::http_client::fetch(
            &url,
            zed::HttpMethod::Post,
            Some(&body),
            &[("Content-Type", "application/json"), ("Authorization", &format!("Bearer {}", token))],
        )?;

        if response.status != 200 {
            let err_text = String::from_utf8_lossy(&response.body);
            return Err(format!("Runner HTTP {}: {}", response.status, err_text));
        }

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
            tooltip: None,
        })
    }

    fn stop_execution(&self) -> zed::Result<zed::SlashCommandOutput> {
        Ok(zed::SlashCommandOutput {
            text: "Stop command sent. Running processes will be terminated automatically.",
            tooltip: None,
        })
    }
}

zed::register_extension!(NovaCibesPythonRunner);