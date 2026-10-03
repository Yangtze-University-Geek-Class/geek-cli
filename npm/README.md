# geek-cli

Yangtze University Geek Class (YUGC) CLI — manage GitHub orgs + the AI Native forum at `yangtzeu.work`.

## Install

```bash
npm install -g @yangtzeu/geek-cli
geek --version
```

The postinstall script downloads the prebuilt native binary (Rust) for your platform from this repo's GitHub release. Supported targets:

- darwin-arm64 / darwin-x64
- linux-x64 / linux-arm64 / linux-x64-musl
- win32-x64

If the download is blocked (e.g. corporate proxy), build from source:

```bash
cargo install --git https://github.com/Yangtze-University-Geek-Class/geek-cli
```

## Auth

```bash
geek login
geek whoami
```

Uses GitHub OAuth Device flow. The forum API accepts the same PAT as Bearer (server resolves `github_id` → forum_user automatically).

## Common

```bash
geek forum me
geek forum stats
geek forum categories
geek forum threads --category models
geek forum new --category agent-mcp --title "..." --body-file draft.md
geek forum reply 42 --body "..."
geek org list
geek member list Yangtze-University-Geek-Class
geek repo tree Yangtze-University-Geek-Class/admin
```

See full reference at https://github.com/Yangtze-University-Geek-Class/geek-cli
