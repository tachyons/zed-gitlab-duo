use zed::lsp::{Completion, Symbol};
use zed::settings::LspSettings;
use zed::{serde_json, CodeLabel, LanguageServerId};
use zed_extension_api::{self as zed, Result};

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
        // Ok(zed::Command {
        //     command: "/opt/homebrew/bin/npm".to_string(),
        //     args: vec![
        //         "run".to_string(),
        //         "start:stdio".to_string(),
        //         "--prefix=/Users/apple/code/gitlab-lsp".to_string(),
        //     ],
        //     env: Default::default(),
        // })
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
        let data = r#"
                {
                "extension": {
                  "name": "Zed Duo extension,
                  "version": "0.0.1",
                },
                "ide": {
                  "name: 'Zed',
                  "version": "0.156.1",
                  "vendor": "Zed",
                 },
                }"#;

        Ok(Some(serde_json::json!(data)))
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
}

zed::register_extension!(GitLabDuoExtension);
