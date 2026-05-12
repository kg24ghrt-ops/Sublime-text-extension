use zed_extension_api as zed;
use serde_json::Value;

struct NovaCibesExtension;

impl zed::Extension for NovaCibesExtension {
    fn new() -> Self {
        Self
    }

    fn language_server_command(
        &mut self,
        _config: &zed::LanguageServerConfig,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command, String> {
        // Fix for E0425: Use the correct settings retrieval method for Zed 1.1[span_2](start_span)[span_2](end_span)
        // Fix for E0282: Explicitly type the error to help the compiler[span_3](start_span)[span_3](end_span)
        let settings = zed::settings::get_extension_settings("novacibes-rust-pro", worktree)
            .map_err(|e: String| e)?;

        let token = settings.get("hf_token")
            .and_then(|v: &Value| v.as_str())
            .ok_or_else(|| "Missing 'hf_token' in settings.json".to_string())?;

        // Professional execution using the system shell[span_4](start_span)[span_4](end_span)
        Ok(zed::Command {
            command: "sh".to_string(),
            args: vec![
                "-c".to_string(),
                format!(
                    "curl -s -X POST https://novacibes-python-running-api.hf.space/run \
                    -H 'Authorization: Bearer {}' \
                    --data-binary @$ZED_FILE", 
                    token
                ),
            ],
            env: vec![],
        })
    }
}

zed::register_extension!(NovaCibesExtension);
