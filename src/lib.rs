use zed_extension_api as zed;

struct NovaCibesExtension;

impl zed::Extension for NovaCibesExtension {
    fn new() -> Self { Self }

    fn language_server_command(
        &mut self,
        _config: &zed::LanguageServerConfig,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command, String> {
        // Retrieve token from Zed's internal settings (configured by user at runtime)
        let settings = zed::settings::get::<serde_json::Value>("novacibes-rust-pro", worktree)
            .map_err(|e| e.to_string())?;

        let token = settings["hf_token"]
            .as_str()
            .ok_or("Error: 'hf_token' not found. Add it to your Zed settings.json.")?;

        // We use the system's native shell to execute the remote curl/websocket 
        // to bypass WASM networking restrictions for a 'Clean' feel[span_4](start_span)[span_4](end_span)
        Ok(zed::Command {
            command: "sh".to_string(),
            args: vec![
                "-c".to_string(),
                format!(
                    "curl -X POST https://novacibes-python-running-api.hf.space/run \
                    -H 'Authorization: Bearer {}' \
                    -d \"$(cat $ZED_FILE)\"", 
                    token
                ),
            ],
            env: vec![],
        })
    }
}

zed::register_extension!(NovaCibesExtension);
