## Gitlab Duo integration for ZED with LSP

### Setup

Install rust

```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Install npx

```
npm install -g npx
git clone <repo>
```

Open Zed -> Extensions -> Install Dev extension -> Pick Downloaded path

Then in add configuration in settings

```json
{
 "lsp": {
  "gitlab-duo": {
    "settings": {
      "baseUrl": "https://gitlab.com",
      
      // Generate from https://gitlab.com/-/user_settings/personal_access_tokens
      // (or the equivalent URL of your Gitlab instance)
      // with scopes: api, ai_features 
      "token": "glpat-<pat>"
    }
  }
 }
}

```

### Use

> [!IMPORTANT]  
> Gitlab Duo support is implemented as a 
> [LSP](https://zed.dev/docs/extensions/languages#language-servers), 
> and experimental  [ACP](https://zed.dev/docs/extensions/agent-servers).
 
<img width="363" height="379" alt="Screenshot 2026-02-01 at 11 26 52 PM" src="https://github.com/user-attachments/assets/a7998037-4789-4e0d-a5b4-67374226b87c" />

For GitLab duo agent, the token need to be set as the env var `GITLAB_AUTH_TOKEN` or in the config file

```
~/.config/gitlab-duo-acp/config.yml
```
```yaml
gitlab_auth_token: glpat-xxxx
```


Open some file and start coding. Duo will start providing suggestions automatically. 
You can ask Duo to display suggestions by running the `editor: show completions` 
action from the command pallette (usually bound to <kbd>Ctrl</kbd>+<kbd>Space</kbd>).
