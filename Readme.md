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
> not [ACP](https://zed.dev/docs/extensions/agent-servers).
> Therefore, you will not find find it in Zed's typical AI locations:
>
> ![Gitlab Duo within Zed](https://github.com/user-attachments/assets/ba42113f-9fcb-49dc-be8a-2f60fc6e18b6)

Open some file and start coding. Duo will start providing suggestions automatically. 
You can ask Duo to display suggestions by running the `editor: show completions` 
action from the command pallette (usually bound to <kbd>Ctrl</kbd>+<kbd>Space</kbd>).
