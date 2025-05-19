use zed::lsp::{Completion, Symbol};
use zed::settings::LspSettings;
use zed::{
    http_client::{HttpMethod, HttpRequest},
    serde_json::{self, json},
};
use zed::{CodeLabel, LanguageServerId};
use zed_extension_api::{self as zed, http_client::RedirectPolicy, Result};

struct GitLabDuoExtension {
    cached_binary_path: Option<String>,
}

impl GitLabDuoExtension {
    // TODO improve installation by automatically downloading latest package
    fn language_server_binary_path(
        &mut self,
        _language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<String> {
        if let Some(path) = worktree.which("npx") {
            let binary_path = path;
            self.cached_binary_path = Some(binary_path.clone());
            Ok(binary_path)
        } else {
            Ok("/todo".to_string())
        }
    }

    // Get GitLab settings from the LspSettings structure
    fn get_gitlab_settings(&self, worktree: &zed::Worktree) -> Result<serde_json::Value> {
        // Get the LspSettings for gitlab-duo
        let lsp_settings = LspSettings::for_worktree("gitlab-duo", worktree).ok();

        // Extract the settings object
        let settings = lsp_settings.unwrap_or_default().settings;
        Ok(settings.unwrap_or_default())
    }

    // Get GitLab API URL from settings
    fn get_gitlab_url(&self, worktree: &zed::Worktree) -> Result<String> {
        let settings = self.get_gitlab_settings(worktree)?;

        // Access baseUrl from the settings object
        let gitlab_url = settings
            .get("baseUrl")
            .and_then(|v| v.as_str())
            .unwrap_or("https://gitlab.com");

        Ok(gitlab_url.to_string())
    }

    // Get GitLab API token from settings
    fn get_gitlab_token(&self, worktree: &zed::Worktree) -> Result<String> {
        let settings = self.get_gitlab_settings(worktree)?;

        // Access token from the settings object
        let token = settings
            .get("token")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                "GitLab token not configured. Please set lsp.gitlab-duo.settings.token in settings."
                    .to_string()
            })?;

        Ok(token.to_string())
    }

    // Format the response text by properly handling newlines
    fn format_response_text(&self, text: &str) -> String {
        // Replace literal "\n" with actual newlines
        let formatted = text.replace("\\n", "\n");

        // Handle other common escape sequences
        let formatted = formatted
            .replace("\\t", "\t")
            .replace("\\\"", "\"")
            .replace("\\\\", "\\");

        formatted
    }
}

impl zed::Extension for GitLabDuoExtension {
    fn new() -> Self {
        Self {
            cached_binary_path: None,
        }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        Ok(zed::Command {
            command: self.language_server_binary_path(language_server_id, worktree)?,
            args: vec![
                "--registry=https://gitlab.com/api/v4/packages/npm/".to_string(),
                "@gitlab-org/gitlab-lsp".to_string(),
                "--stdio".to_string(),
            ],
            env: Default::default(),
        })
    }

    fn language_server_workspace_configuration(
        &mut self,
        _language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<Option<serde_json::Value>> {
        let settings = LspSettings::for_worktree("gitlab-duo", worktree)
            .ok()
            .and_then(|lsp_settings| lsp_settings.settings.clone())
            .unwrap_or_default();
        Ok(Some(settings))
    }

    /// Returns the initialization options to pass to the specified language server.
    fn language_server_initialization_options(
        &mut self,
        _language_server_id: &LanguageServerId,
        _worktree: &zed::Worktree,
    ) -> Result<Option<serde_json::Value>> {
        // Fixed string literal with proper quotes
        let data = r#"{
                "extension": {
                  "name": "Zed Duo extension",
                  "version": "0.0.1"
                },
                "ide": {
                  "name": "Zed",
                  "version": "0.156.1",
                  "vendor": "Zed"
                }
            }"#;

        // Parse the JSON string to a Value
        let parsed = serde_json::from_str(data)
            .map_err(|e| format!("Failed to parse initialization options: {}", e))?;

        Ok(Some(parsed))
    }

    /// Returns the label for the given completion.
    fn label_for_completion(
        &self,
        _language_server_id: &LanguageServerId,
        _completion: Completion,
    ) -> Option<CodeLabel> {
        None
    }

    /// Returns the label for the given symbol.
    fn label_for_symbol(
        &self,
        _language_server_id: &LanguageServerId,
        _symbol: Symbol,
    ) -> Option<CodeLabel> {
        None
    }

    // Add slash command support
    fn run_slash_command(
        &self,
        command: zed::SlashCommand,
        args: Vec<String>,
        worktree: Option<&zed::Worktree>,
    ) -> Result<zed::SlashCommandOutput> {
        match command.name.as_str() {
            "duo-explain" => {
                let worktree = match worktree {
                    Some(wt) => wt,
                    None => return Err("Worktree is required for duo-explain command".into()),
                };

                // Get the content to explain from command arguments
                let content = if !args.is_empty() {
                    args.join(" ")
                } else {
                    return Ok(zed::SlashCommandOutput {
                        text: "Please provide code or text to explain after the command, e.g.: /duo-explain function myFunction() { ... }".to_string(),
                        sections: vec![],
                    });
                };

                if content.is_empty() {
                    return Ok(zed::SlashCommandOutput {
                        text: "Please provide code or text to explain.".to_string(),
                        sections: vec![],
                    });
                }

                // Get GitLab token and URL from settings
                let token = self.get_gitlab_token(worktree)?;
                let gitlab_url = self.get_gitlab_url(worktree)?;

                // Create the API endpoint URL
                let api_url = format!("{}/api/v4/chat/completions", gitlab_url);

                // Prepare the request body - no file context, just the query
                let request_body = json!({
                    "content": content,
                    "with_clean_history": true
                });

                // Prepare the request with correct API structure
                let request = HttpRequest {
                    method: HttpMethod::Post,
                    url: api_url,
                    headers: vec![
                        ("Authorization".to_string(), format!("Bearer {}", token)),
                        ("Content-Type".to_string(), "application/json".to_string()),
                    ],
                    body: Some(
                        serde_json::to_vec(&request_body)
                            .map_err(|e| format!("Failed to serialize request body: {}", e))?,
                    ),
                    redirect_policy: RedirectPolicy::FollowAll,
                };

                // Make the HTTP request
                match zed::http_client::fetch(&request) {
                    Ok(response) => {
                        // Check status code from headers
                        let status_code = response
                            .headers
                            .iter()
                            .find(|(name, _)| name.to_lowercase() == "status")
                            .map(|(_, value)| value.parse::<u16>().unwrap_or(200))
                            .unwrap_or(200);

                        if status_code >= 200 && status_code < 300 && !response.body.is_empty() {
                            // Parse the response
                            match String::from_utf8(response.body.clone()) {
                                Ok(text) => {
                                    // Try to parse as JSON first in case API returns structured data
                                    match serde_json::from_str::<serde_json::Value>(&text) {
                                        Ok(json_response) => {
                                            // Check if there's a content field in the response
                                            if let Some(content) = json_response
                                                .get("content")
                                                .and_then(|c| c.as_str())
                                            {
                                                // Format the content to properly handle newlines
                                                let formatted_content =
                                                    self.format_response_text(content);

                                                return Ok(zed::SlashCommandOutput {
                                                    text: formatted_content.clone(),
                                                    sections: vec![
                                                        zed::SlashCommandOutputSection {
                                                            range: (0..formatted_content.len())
                                                                .into(),
                                                            label: "GitLab Duo Explanation"
                                                                .to_string(),
                                                        },
                                                    ],
                                                });
                                            } else {
                                                // Just return the whole JSON as text if no content field
                                                let formatted_text =
                                                    self.format_response_text(&text);

                                                return Ok(zed::SlashCommandOutput {
                                                    text: formatted_text.clone(),
                                                    sections: vec![
                                                        zed::SlashCommandOutputSection {
                                                            range: (0..formatted_text.len()).into(),
                                                            label: "GitLab Duo Response"
                                                                .to_string(),
                                                        },
                                                    ],
                                                });
                                            }
                                        }
                                        Err(_) => {
                                            // Not JSON, treat as plain text
                                            let formatted_text = self.format_response_text(&text);

                                            return Ok(zed::SlashCommandOutput {
                                                text: formatted_text.clone(),
                                                sections: vec![zed::SlashCommandOutputSection {
                                                    range: (0..formatted_text.len()).into(),
                                                    label: "GitLab Duo Explanation".to_string(),
                                                }],
                                            });
                                        }
                                    }
                                }
                                Err(_) => {
                                    return Ok(zed::SlashCommandOutput {
                                        text: format!(
                                            "Invalid UTF-8 in response: {}",
                                            String::from_utf8_lossy(&response.body)
                                        ),
                                        sections: vec![],
                                    });
                                }
                            }
                        } else {
                            // Handle error response
                            let error_message = String::from_utf8_lossy(&response.body);
                            return Ok(zed::SlashCommandOutput {
                                text: format!(
                                    "Error from GitLab Duo API ({}): {}",
                                    status_code, error_message
                                ),
                                sections: vec![],
                            });
                        }
                    }
                    Err(e) => {
                        return Ok(zed::SlashCommandOutput {
                            text: format!("Request failed: {}", e),
                            sections: vec![],
                        });
                    }
                }
            }
            _ => Err(format!("Unknown slash command: {}", command.name).into()),
        }
    }
}

zed::register_extension!(GitLabDuoExtension);
