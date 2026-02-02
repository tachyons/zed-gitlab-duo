use zed::lsp::{Completion, Symbol};
use zed::serde_json::{self, json};
use zed::settings::LspSettings;
use zed::{CodeLabel, CodeLabelSpan, ContextServerId, LanguageServerId};
use zed_extension_api::{self as zed, settings::ContextServerSettings, Result};

// Constants for versioning and configuration
const EXTENSION_NAME: &str = "Zed Duo extension";
const EXTENSION_VERSION: &str = "0.0.1";
const ZED_NAME: &str = "Zed";
const ZED_VERSION: &str = "0.156.1";
const ZED_VENDOR: &str = "Zed";
const GITLAB_REGISTRY: &str = "https://gitlab.com/api/v4/packages/npm/";
const GITLAB_LSP_PACKAGE: &str = "@gitlab-org/gitlab-lsp";
const DEFAULT_GITLAB_URL: &str = "https://gitlab.com";

struct GitLabDuoExtension {
    cached_binary_path: Option<String>,
}

impl GitLabDuoExtension {
    fn language_server_binary_path(
        &mut self,
        _language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<String> {
        // Use cached path if available
        if let Some(cached) = &self.cached_binary_path {
            return Ok(cached.clone());
        }

        // Find npx in PATH
        if let Some(path) = worktree.which("npx") {
            self.cached_binary_path = Some(path.clone());
            Ok(path)
        } else {
            Err(
                "npx not found in PATH. Please install Node.js and npm to use GitLab Duo."
                    .to_string(),
            )?
        }
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
                format!("--registry={}", GITLAB_REGISTRY),
                GITLAB_LSP_PACKAGE.to_string(),
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
            .map_err(|e| format!("Failed to load GitLab Duo settings: {}", e))?
            .settings
            .unwrap_or_else(|| {
                // Provide default settings with helpful comment
                json!({
                    "codeCompletion": {
                        "enabled": true
                    }
                })
            });

        Ok(Some(settings))
    }

    fn language_server_initialization_options(
        &mut self,
        _language_server_id: &LanguageServerId,
        _worktree: &zed::Worktree,
    ) -> Result<Option<serde_json::Value>> {
        // Use json! macro for cleaner, compile-time validated JSON
        Ok(Some(json!({
            "extension": {
                "name": EXTENSION_NAME,
                "version": EXTENSION_VERSION
            },
            "ide": {
                "name": ZED_NAME,
                "version": ZED_VERSION,
                "vendor": ZED_VENDOR
            }
        })))
    }

    fn label_for_completion(
        &self,
        _language_server_id: &LanguageServerId,
        completion: Completion,
    ) -> Option<CodeLabel> {
        // Build the label text with optional detail
        let label_text = if let Some(detail) = &completion.detail {
            if detail.is_empty() {
                completion.label.clone()
            } else {
                format!("{} {}", completion.label, detail)
            }
        } else {
            completion.label.clone()
        };

        let filter_range = 0..completion.label.len();

        // Create highlighting spans based on completion kind
        let mut spans = vec![CodeLabelSpan::literal(&completion.label, None)];

        if let Some(detail) = &completion.detail {
            if !detail.is_empty() {
                spans.push(CodeLabelSpan::literal(" ", None));
                spans.push(CodeLabelSpan::literal(detail, Some("comment".to_string())));
            }
        }

        Some(CodeLabel {
            code: label_text,
            spans,
            filter_range: filter_range.into(),
        })
    }

    fn label_for_symbol(
        &self,
        _language_server_id: &LanguageServerId,
        symbol: Symbol,
    ) -> Option<CodeLabel> {
        let label_text = symbol.name.clone();
        let filter_range = 0..symbol.name.len();

        // Apply syntax highlighting based on symbol kind
        let highlight = match symbol.kind {
            zed::lsp::SymbolKind::Function | zed::lsp::SymbolKind::Method => "function",
            zed::lsp::SymbolKind::Class | zed::lsp::SymbolKind::Interface => "type",
            zed::lsp::SymbolKind::Variable | zed::lsp::SymbolKind::Constant => "variable",
            zed::lsp::SymbolKind::Module | zed::lsp::SymbolKind::Namespace => "keyword",
            _ => "identifier",
        };

        Some(CodeLabel {
            code: label_text,
            spans: vec![CodeLabelSpan::literal(
                &symbol.name,
                Some(highlight.to_string()),
            )],
            filter_range: filter_range.into(),
        })
    }


    fn context_server_command(
        &mut self,
        _context_server_id: &ContextServerId,
        project: &zed::Project,
    ) -> Result<zed::Command> {
        // Load and validate settings
        let settings = ContextServerSettings::for_project("gitlab-mcp", project)
            .map_err(|e| format!("Failed to load GitLab MCP settings: {}", e))?;

        let settings_json = settings.settings.unwrap_or_default();

        // Extract and validate base URL
        let url = settings_json
            .get("baseUrl")
            .and_then(|v| v.as_str())
            .unwrap_or(DEFAULT_GITLAB_URL);

        // Basic URL validation
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(format!(
                "Invalid baseUrl '{}': must start with http:// or https://",
                url
            ))?;
        }

        // Note: npx will fail with a clear error if not found, so we don't need to validate here
        Ok(zed::Command {
            command: "npx".to_string(),
            args: vec![
                "-y".to_string(),
                "mcp-remote@latest".to_string(),
                format!("{}/api/v4/mcp", url.trim_end_matches('/')),
            ],
            env: Default::default(),
        })
    }
}

zed::register_extension!(GitLabDuoExtension);
