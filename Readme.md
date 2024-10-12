## Gitlab Duo integration for ZED with LSP

### Setup

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
