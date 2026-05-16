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

## Forum (AI Native discussion) — yangtzeu.work/forum

The CLI uses the same stored GitHub PAT as auth. The forum backend resolves the PAT via GitHub `/user` once, caches the github_id → forum_user mapping for 5 minutes, and treats subsequent calls as that forum user. You don't need a separate forum login.

```bash
# identity + stats
geek forum me                              # current forum user, role, groups, permissions
geek forum stats                           # member count, thread/post totals, recent threads

# read
geek forum categories                      # 6 AI Native sections
geek forum threads                         # latest across all
geek forum threads --category models       # in one section
geek forum threads --q "agent"             # title search
geek forum threads --sort hot              # by view + reply
geek forum threads --archive               # legacy mbbs read-only archive
geek forum thread 42                       # thread + replies (page 1)
geek forum user crosery                    # public profile

# write (requires member or above)
geek forum new --category agent-mcp --title "MCP server discovery" --body "正文..."
geek forum new --category models --title "..." --body-file ./draft.md
geek forum new --category models --title "..." --body-file -        # read from stdin
geek forum reply 42 --body "回复内容"
geek forum reply 42 --reply-to 88 --body "回复 #88 楼"
geek forum like 99                         # toggles like on post 99
geek forum delete-thread 42                # own thread, or any if admin
geek forum delete-post 99                  # own post, or any if admin

# admin (requires role=admin or mod)
geek forum users --role admin              # list role=admin
geek forum users --q crosery               # search by username/display/github
geek forum set-role 165 admin              # promote user 165 to 负责人
geek forum set-role 165 member             # demote
geek forum set-role 165 banned             # ban
geek forum pin-thread 42 --what sticky     # toggle 置顶 (or essence / lock)
geek forum pin-thread 42 --what sticky --on=false   # un-stick

# groups
geek forum groups                          # list 游客 / 成员 / 负责人 with member + perm counts
geek forum notifications                   # your unread + recent
```

Permissions are enforced on the backend (`hasPermission(...)` per route). The CLI will surface 403 verbatim — non-admin users calling admin endpoints will fail at the API layer, not just the CLI.

By default the forum API lives at `https://yangtzeu.work`. Override with `GEEK_FORUM_BASE=https://forum.yangtzeu.work` once the dedicated subdomain has its cert.

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
