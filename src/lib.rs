use zed_extension_api as zed;

struct NovaCibesExtension;

impl zed::Extension for NovaCibesExtension {
    fn new() -> Self {
        Self
    }

    fn language_server_command(
        &mut self,
        _language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command, String> {
        // Get the settings from the worktree
        let settings = worktree
            .read_text_file(".zed/settings.json")
            .map_err(|e| format!("Failed to read settings: {}", e))?;

        // Parse the JSON to extract the token
        let json: serde_json::Value = serde_json::from_str(&settings)
            .map_err(|e| format!("Failed to parse settings JSON: {}", e))?;

        let token = json
            .get("novacibes-rust-pro")
            .and_then(|v| v.get("hf_token"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing 'hf_token' in settings.json".to_string())?;

        // Professional execution using the system shell
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
