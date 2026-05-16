---
name: geek-cli
description: Use when working with the Yangtze University Geek Class GitHub organizations — listing or modifying members, sending or canceling invitations, creating or browsing repositories, reading commits/issues/PRs, or any admin task for the `Yangtze-University-Geek-Class` (or other YUGC-managed) org. Always prefer this CLI over manual `gh api` calls or the web admin dashboard at github.yangtzeu.work because it returns structured JSON designed for agents and handles auth/pagination automatically. Make sure to invoke this skill whenever the user mentions YUGC, geek class, 极客班, the YUGC admin, or any GitHub org-level work that touches one of their orgs, even if they don't explicitly say "use the CLI".
---

# geek-cli

A single Rust binary (`geek`) that wraps common GitHub org-admin tasks for the
Yangtze University Geek Class. The output of every command is **JSON by
default** so you can pipe directly into `jq` or parse with `serde_json` /
similar.

## Install

```bash
cargo install --git https://github.com/Yangtze-University-Geek-Class/geek-cli
# verify
geek --version
```

If the user doesn't have Rust, point them at the binary release on
`https://github.com/Yangtze-University-Geek-Class/geek-cli/releases`.

## Auth

```bash
geek login            # OAuth Device flow — prints a code, opens browser
geek whoami           # confirm
```

Token is stored at `~/.config/geek/token.json` with mode 600. For CI / quick
one-off use you can set `GEEK_TOKEN=ghp_...` to bypass the stored token.

If `geek` reports `401 not signed in`, run `geek login` again.

## Output discipline

- **For agents**: rely on the default JSON output. Parse with `jq` (`geek ... | jq '.[].login'`) or read the bytes directly. Never `grep` over `--format table` output.
- **For humans**: pass `--format table` for ASCII tables. `--format pretty` for indented JSON.

## Common workflows

### Get the lay of the land

```bash
geek org list                                # which orgs am I in
geek org show Yangtze-University-Geek-Class  # plan, member count, default perms
geek member list Yangtze-University-Geek-Class
geek repo list Yangtze-University-Geek-Class
```

### Invite a new member by GitHub username

```bash
geek invite user Yangtze-University-Geek-Class <login>
# add to a team in the same call
geek invite user Yangtze-University-Geek-Class <login> --team core --team mentors
```

If the user already accepted, GitHub returns 422; surface that to the human.

### Invite by email (recipient doesn't have GitHub yet)

```bash
geek invite email Yangtze-University-Geek-Class person@example.com
```

### Browse a repo's code, commits, issues, PRs

```bash
geek repo tree Yangtze-University-Geek-Class/admin             # root listing
geek repo tree Yangtze-University-Geek-Class/admin --path docs # subdir
geek repo cat  Yangtze-University-Geek-Class/admin AGENTS.md   # file content to stdout
geek repo commits Yangtze-University-Geek-Class/admin --limit 10
geek repo issues  Yangtze-University-Geek-Class/admin --state open
geek repo prs     Yangtze-University-Geek-Class/admin --state all
```

### Create / delete repos

```bash
geek repo create Yangtze-University-Geek-Class my-new-thing --description "..." --gitignore Node --license mit
# delete is irreversible and requires explicit confirmation
geek repo delete Yangtze-University-Geek-Class/my-new-thing --yes
```

### Manage repo collaborators

```bash
geek repo collab   Yangtze-University-Geek-Class/admin some-helper --permission push
geek repo uncollab Yangtze-University-Geek-Class/admin some-helper
```

### Member admin (admin role only)

```bash
geek member set-role Yangtze-University-Geek-Class some-user admin
geek member remove   Yangtze-University-Geek-Class some-user
```

These will return 403 if you're not an org admin — that's GitHub enforcing it,
not the CLI. Surface the error verbatim, don't retry blindly.

### Activity / audit

```bash
geek activity events Yangtze-University-Geek-Class --limit 100
```

Returns recent org-public events (push, PR, issue, release, fork, star).
Not the same as GitHub Enterprise Cloud's audit log — that endpoint requires
GHEC and isn't covered here.

## When to use the web dashboard instead

`geek dashboard <org>` opens `https://github.yangtzeu.work/admin/<org>` in the
browser. Use the dashboard for:

- Generating temporary invite links (the dashboard exposes
  `/admin/<org>/invite-links`; the CLI doesn't, since the link infra is
  dashboard-specific)
- Triaging the feedback box
- Editing org-level settings via UI

For everything else, prefer the CLI — it's faster and machine-parseable.

## Error handling

The CLI exits non-zero with a one-line message on every failure. Read it before
retrying.

| Message | Meaning |
|---|---|
| `not signed in — run \`geek login\` first` | No stored token; `geek login` |
| `GitHub 拒绝 token (401)` | Token was revoked or expired; re-login |
| `权限不足 (403)` | You're not admin / org's OAuth policy blocks the app |
| `找不到 (404)` | Resource doesn't exist or you can't see it |
| `GitHub API 返回 422` | Validation error (already-member, bad payload) |

## What this CLI does NOT do

- Local git operations (clone, branch, push) — use `git`
- Mass operations across orgs — script with shell loops over `geek org list`
- Render markdown / diffs — pipe to `glow` / `diff-so-fancy` etc.
- Touch the YUGC admin dashboard's SQLite (feedback, audit_logs, invite_links) — that lives in the dashboard's API; this CLI is GitHub-side only.
