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
      "token": "glpat-<pat>"
    }
  }
 }
}

```
