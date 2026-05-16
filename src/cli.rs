use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use serde_json::{Value, json};

use crate::config::{self, Stored};
use crate::forum::ForumClient;
use crate::gh::GhClient;
use crate::output::{Format, emit, print_message};

#[derive(Parser, Debug)]
#[command(name = "geek", version, about = "Yangtze University Geek Class CLI for agents and humans", long_about = None)]
pub struct Cli {
    /// Output format
    #[arg(global = true, short, long, value_enum, default_value_t = Format::Json)]
    pub format: Format,

    #[command(subcommand)]
    pub command: Cmd,
}

#[derive(Subcommand, Debug)]
pub enum Cmd {
    /// Sign in via GitHub OAuth Device flow
    Login,
    /// Forget the stored token
    Logout,
    /// Show the currently signed-in account
    Whoami,
    /// Organization commands
    #[command(subcommand)]
    Org(OrgCmd),
    /// Member commands
    #[command(subcommand)]
    Member(MemberCmd),
    /// Pending invite commands
    #[command(subcommand)]
    Invite(InviteCmd),
    /// Repo commands
    #[command(subcommand)]
    Repo(RepoCmd),
    /// Audit / activity
    #[command(subcommand)]
    Activity(ActivityCmd),
    /// Forum (YUGC AI Native discussion) commands
    #[command(subcommand)]
    Forum(ForumCmd),
    /// Open the YUGC admin dashboard in browser
    Dashboard {
        /// Optional org
        org: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum ForumCmd {
    /// Show current forum identity (resolved from GitHub PAT via github_id)
    Me,
    /// Forum-wide stats
    Stats,
    /// List active (non-legacy) categories
    Categories,
    /// List threads. --archive shows the legacy mbbs archive instead.
    Threads {
        #[arg(long)]
        category: Option<String>,
        #[arg(long)]
        q: Option<String>,
        #[arg(long, default_value = "latest", value_parser = ["latest", "hot"])]
        sort: String,
        #[arg(long, default_value_t = 1)]
        page: u32,
        #[arg(long)]
        archive: bool,
    },
    /// Show a thread + replies
    Thread {
        id: u64,
        #[arg(long, default_value_t = 1)]
        page: u32,
    },
    /// Create a new thread
    New {
        /// Category slug (use `forum categories` to list)
        #[arg(long)]
        category: String,
        #[arg(long)]
        title: String,
        /// Body text; mutually exclusive with --body-file
        #[arg(long)]
        body: Option<String>,
        /// Read body from file (path or `-` for stdin)
        #[arg(long)]
        body_file: Option<String>,
    },
    /// Reply to a thread
    Reply {
        thread_id: u64,
        #[arg(long)]
        body: Option<String>,
        #[arg(long)]
        body_file: Option<String>,
        /// Optional parent post id to reply to a specific floor
        #[arg(long)]
        reply_to: Option<u64>,
    },
    /// Like / unlike a post
    Like { post_id: u64 },
    /// Delete a thread you own (or any thread if admin)
    DeleteThread { id: u64 },
    /// Delete a post you own (or any post if admin)
    DeletePost { id: u64 },
    /// Admin: pin/unpin / mark essence / lock thread
    PinThread {
        id: u64,
        #[arg(long, value_parser = ["sticky", "essence", "lock"])]
        what: String,
        #[arg(long, default_value_t = true)]
        on: bool,
    },
    /// Show a user's public profile
    User { username: String },
    /// List groups
    Groups,
    /// Admin: list forum users (paginated, filterable)
    Users {
        #[arg(long)]
        q: Option<String>,
        #[arg(long, value_parser = ["admin", "mod", "member", "banned"])]
        role: Option<String>,
        #[arg(long, default_value_t = 1)]
        page: u32,
    },
    /// Admin: change a user's role
    SetRole {
        user_id: u64,
        #[arg(value_parser = ["admin", "mod", "member", "banned"])]
        role: String,
    },
    /// List unread notifications
    Notifications,
}

#[derive(Subcommand, Debug)]
pub enum OrgCmd {
    /// List orgs the signed-in user belongs to
    List,
    /// Show org details
    Show { org: String },
    /// Members of an org (alias for `member list`)
    Members { org: String },
}

#[derive(Subcommand, Debug)]
pub enum MemberCmd {
    /// List org members with role
    List { org: String },
    /// Show one member's GitHub profile + role in the org
    Show { org: String, login: String },
    /// Remove a member (org admin only)
    Remove { org: String, login: String },
    /// Set a member's role
    SetRole {
        org: String,
        login: String,
        #[arg(value_parser = ["admin", "member"])]
        role: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum InviteCmd {
    /// List pending invitations
    List { org: String },
    /// Invite by GitHub username
    User {
        org: String,
        login: String,
        #[arg(long, default_value = "direct_member", value_parser = ["admin", "direct_member", "billing_manager"])]
        role: String,
        #[arg(long)]
        team: Vec<String>,
    },
    /// Invite by email
    Email {
        org: String,
        email: String,
        #[arg(long, default_value = "direct_member", value_parser = ["admin", "direct_member", "billing_manager"])]
        role: String,
    },
    /// Cancel a pending invitation by id
    Cancel { org: String, id: u64 },
}

#[derive(Subcommand, Debug)]
pub enum RepoCmd {
    /// List repos in an org
    List { org: String },
    /// Show repo info
    Show { full: String },
    /// Create a new repo: <org> <name>
    Create {
        org: String,
        name: String,
        #[arg(long, default_value = "private", value_parser = ["public", "private"])]
        visibility: String,
        #[arg(long)]
        description: Option<String>,
        #[arg(long)]
        no_init: bool,
        #[arg(long)]
        gitignore: Option<String>,
        #[arg(long)]
        license: Option<String>,
    },
    /// Delete a repo (DANGEROUS)
    Delete {
        full: String,
        #[arg(long)]
        yes: bool,
    },
    /// List branches
    Branches { full: String },
    /// List directory entries at path
    Tree {
        full: String,
        #[arg(long, default_value = "")]
        path: String,
        #[arg(long)]
        r#ref: Option<String>,
    },
    /// Print file content
    Cat {
        full: String,
        path: String,
        #[arg(long)]
        r#ref: Option<String>,
    },
    /// List commits
    Commits {
        full: String,
        #[arg(long)]
        r#ref: Option<String>,
        #[arg(long, default_value_t = 30)]
        limit: u32,
    },
    /// List issues
    Issues {
        full: String,
        #[arg(long, default_value = "open", value_parser = ["open", "closed", "all"])]
        state: String,
    },
    /// List PRs
    Prs {
        full: String,
        #[arg(long, default_value = "open", value_parser = ["open", "closed", "all"])]
        state: String,
    },
    /// Add a collaborator
    Collab {
        full: String,
        login: String,
        #[arg(long, default_value = "push", value_parser = ["pull", "triage", "push", "maintain", "admin"])]
        permission: String,
    },
    /// Remove a collaborator
    Uncollab { full: String, login: String },
}

#[derive(Subcommand, Debug)]
pub enum ActivityCmd {
    /// Org-level public events
    Events { org: String, #[arg(long, default_value_t = 30)] limit: u32 },
}

pub async fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Cmd::Login => cmd_login().await,
        Cmd::Logout => cmd_logout(),
        Cmd::Whoami => cmd_whoami(cli.format).await,
        Cmd::Org(c) => cmd_org(c, cli.format).await,
        Cmd::Member(c) => cmd_member(c, cli.format).await,
        Cmd::Invite(c) => cmd_invite(c, cli.format).await,
        Cmd::Repo(c) => cmd_repo(c, cli.format).await,
        Cmd::Activity(c) => cmd_activity(c, cli.format).await,
        Cmd::Forum(c) => cmd_forum(c, cli.format).await,
        Cmd::Dashboard { org } => cmd_dashboard(org),
    }
}

fn read_body(body: Option<String>, body_file: Option<String>) -> Result<String> {
    if let Some(b) = body {
        return Ok(b);
    }
    if let Some(p) = body_file {
        if p == "-" {
            use std::io::Read;
            let mut s = String::new();
            std::io::stdin().read_to_string(&mut s)?;
            return Ok(s);
        }
        return std::fs::read_to_string(&p).with_context(|| format!("read {p}"));
    }
    bail!("provide --body or --body-file")
}

async fn cmd_forum(c: ForumCmd, fmt: Format) -> Result<()> {
    let token = config::require_token()?;
    let f = ForumClient::new(token)?;
    match c {
        ForumCmd::Me => {
            let v: Value = f.get("/api/forum/me").await?;
            emit(&v, fmt)
        }
        ForumCmd::Stats => {
            let v: Value = f.get("/api/forum/stats").await?;
            emit(&v, fmt)
        }
        ForumCmd::Categories => {
            let v: Value = f.get("/api/forum/categories").await?;
            emit(&v, fmt)
        }
        ForumCmd::Threads { category, q, sort, page, archive } => {
            let mut qs: Vec<String> = vec![format!("sort={sort}"), format!("page={page}")];
            if let Some(c) = category { qs.push(format!("category={}", urlencoding::encode(&c))); }
            if let Some(s) = q { qs.push(format!("q={}", urlencoding::encode(&s))); }
            if archive { qs.push("archive=1".into()); }
            let v: Value = f.get(&format!("/api/forum/threads?{}", qs.join("&"))).await?;
            emit(&v, fmt)
        }
        ForumCmd::Thread { id, page } => {
            let v: Value = f.get(&format!("/api/forum/threads/{id}?page={page}")).await?;
            emit(&v, fmt)
        }
        ForumCmd::New { category, title, body, body_file } => {
            let body = read_body(body, body_file)?;
            // resolve slug -> category_id
            let cats: Value = f.get("/api/forum/categories").await?;
            let id = cats["categories"].as_array().and_then(|a| a.iter().find(|c| c["slug"] == Value::String(category.clone()))).and_then(|c| c["id"].as_i64())
                .ok_or_else(|| anyhow::anyhow!("unknown category slug; use `geek forum categories` to list"))?;
            let v: Value = f.post("/api/forum/threads", &json!({
                "category_id": id, "title": title, "content": body, "content_format": "markdown"
            })).await?;
            emit(&v, fmt)
        }
        ForumCmd::Reply { thread_id, body, body_file, reply_to } => {
            let body = read_body(body, body_file)?;
            let mut payload = json!({ "thread_id": thread_id, "content": body, "content_format": "markdown" });
            if let Some(r) = reply_to { payload["reply_post_id"] = json!(r); }
            let v: Value = f.post("/api/forum/posts", &payload).await?;
            emit(&v, fmt)
        }
        ForumCmd::Like { post_id } => {
            let v: Value = f.post(&format!("/api/forum/posts/{post_id}/like"), &json!({})).await?;
            emit(&v, fmt)
        }
        ForumCmd::DeleteThread { id } => {
            f.delete(&format!("/api/forum/threads/{id}")).await?;
            emit(&json!({ "ok": true, "deleted_thread": id }), fmt)
        }
        ForumCmd::DeletePost { id } => {
            f.delete(&format!("/api/forum/posts/{id}")).await?;
            emit(&json!({ "ok": true, "deleted_post": id }), fmt)
        }
        ForumCmd::PinThread { id, what, on } => {
            let field = match what.as_str() {
                "sticky" => "is_sticky",
                "essence" => "is_essence",
                "lock" => "is_locked",
                _ => bail!("invalid --what"),
            };
            let v = f.patch(&format!("/api/forum/threads/{id}"), &json!({ field: if on { 1 } else { 0 } })).await?;
            emit(&v, fmt)
        }
        ForumCmd::User { username } => {
            let v: Value = f.get(&format!("/api/forum/users/{}", urlencoding::encode(&username))).await?;
            emit(&v, fmt)
        }
        ForumCmd::Groups => {
            let v: Value = f.get("/api/forum/groups").await?;
            emit(&v, fmt)
        }
        ForumCmd::Users { q, role, page } => {
            let mut qs: Vec<String> = vec![format!("page={page}")];
            if let Some(s) = q { qs.push(format!("q={}", urlencoding::encode(&s))); }
            if let Some(r) = role { qs.push(format!("role={r}")); }
            let v: Value = f.get(&format!("/api/forum/admin/users?{}", qs.join("&"))).await?;
            emit(&v, fmt)
        }
        ForumCmd::SetRole { user_id, role } => {
            let v = f.patch(&format!("/api/forum/admin/users/{user_id}/role"), &json!({ "role": role })).await?;
            emit(&v, fmt)
        }
        ForumCmd::Notifications => {
            let v: Value = f.get("/api/forum/me/notifications").await?;
            emit(&v, fmt)
        }
    }
}

async fn cmd_login() -> Result<()> {
    let (token, login) = crate::auth::login().await?;
    config::save(&Stored { token: Some(token), login: Some(login.clone()) })?;
    print_message(&format!("已登录为 @{login}。token 存到 {}", config::token_path()?.display()));
    Ok(())
}

fn cmd_logout() -> Result<()> {
    config::clear()?;
    print_message("已退出，token 已删除");
    Ok(())
}

async fn cmd_whoami(fmt: Format) -> Result<()> {
    let token = config::require_token()?;
    let gh = GhClient::new(token)?;
    let me: Value = gh.get("/user").await?;
    emit(&json!({
        "login": me["login"],
        "id": me["id"],
        "name": me["name"],
        "html_url": me["html_url"],
        "avatar_url": me["avatar_url"],
    }), fmt)
}

fn split_full(full: &str) -> Result<(&str, &str)> {
    full.split_once('/').context("expected <org>/<repo> form")
}

async fn cmd_org(c: OrgCmd, fmt: Format) -> Result<()> {
    let gh = GhClient::new(config::require_token()?)?;
    match c {
        OrgCmd::List => {
            let memberships: Vec<Value> = gh
                .paginate("/user/memberships/orgs?state=active")
                .await?;
            let rows: Vec<Value> = memberships
                .into_iter()
                .map(|m| {
                    json!({
                        "login": m["organization"]["login"],
                        "name":  m["organization"]["name"],
                        "role":  m["role"],
                        "state": m["state"],
                    })
                })
                .collect();
            emit(&rows, fmt)
        }
        OrgCmd::Show { org } => {
            let v: Value = gh.get(&format!("/orgs/{org}")).await?;
            emit(&v, fmt)
        }
        OrgCmd::Members { org } => members_list(&gh, &org, fmt).await,
    }
}

async fn cmd_member(c: MemberCmd, fmt: Format) -> Result<()> {
    let gh = GhClient::new(config::require_token()?)?;
    match c {
        MemberCmd::List { org } => members_list(&gh, &org, fmt).await,
        MemberCmd::Show { org, login } => {
            let ms: Value = gh
                .get(&format!("/orgs/{org}/memberships/{login}"))
                .await?;
            let user: Value = gh.get(&format!("/users/{login}")).await?;
            emit(&json!({ "user": user, "membership": ms }), fmt)
        }
        MemberCmd::Remove { org, login } => {
            gh.delete(&format!("/orgs/{org}/memberships/{login}")).await?;
            emit(&json!({ "ok": true, "removed": login, "org": org }), fmt)
        }
        MemberCmd::SetRole { org, login, role } => {
            let v = gh
                .put(
                    &format!("/orgs/{org}/memberships/{login}"),
                    &json!({ "role": role }),
                )
                .await?;
            emit(&v, fmt)
        }
    }
}

async fn members_list(gh: &GhClient, org: &str, fmt: Format) -> Result<()> {
    let members: Vec<Value> = gh.paginate(&format!("/orgs/{org}/members")).await?;
    let mut rows: Vec<Value> = Vec::with_capacity(members.len());
    for m in members {
        let login = m["login"].as_str().unwrap_or("").to_string();
        let ms: Value = gh
            .get(&format!("/orgs/{org}/memberships/{login}"))
            .await
            .unwrap_or(Value::Null);
        rows.push(json!({
            "login": login,
            "role": ms.get("role").cloned().unwrap_or(Value::Null),
            "state": ms.get("state").cloned().unwrap_or(Value::Null),
            "avatar_url": m["avatar_url"],
            "html_url": m["html_url"],
        }));
    }
    emit(&rows, fmt)
}

async fn cmd_invite(c: InviteCmd, fmt: Format) -> Result<()> {
    let gh = GhClient::new(config::require_token()?)?;
    match c {
        InviteCmd::List { org } => {
            let v: Vec<Value> = gh.paginate(&format!("/orgs/{org}/invitations")).await?;
            let rows: Vec<Value> = v
                .into_iter()
                .map(|i| {
                    json!({
                        "id": i["id"],
                        "login": i["login"],
                        "email": i["email"],
                        "role": i["role"],
                        "created_at": i["created_at"],
                        "inviter": i["inviter"]["login"],
                    })
                })
                .collect();
            emit(&rows, fmt)
        }
        InviteCmd::User { org, login, role, team } => {
            let user: Value = gh.get(&format!("/users/{login}")).await?;
            let id = user["id"].as_u64().context("user id missing")?;
            let team_ids = if team.is_empty() {
                vec![]
            } else {
                let mut ids = Vec::new();
                for slug in team {
                    let t: Value = gh.get(&format!("/orgs/{org}/teams/{slug}")).await?;
                    if let Some(id) = t["id"].as_u64() { ids.push(id); }
                }
                ids
            };
            let body = if team_ids.is_empty() {
                json!({ "invitee_id": id, "role": role })
            } else {
                json!({ "invitee_id": id, "role": role, "team_ids": team_ids })
            };
            let r: Value = gh.post(&format!("/orgs/{org}/invitations"), &body).await?;
            emit(&r, fmt)
        }
        InviteCmd::Email { org, email, role } => {
            let r: Value = gh
                .post(
                    &format!("/orgs/{org}/invitations"),
                    &json!({ "email": email, "role": role }),
                )
                .await?;
            emit(&r, fmt)
        }
        InviteCmd::Cancel { org, id } => {
            gh.delete(&format!("/orgs/{org}/invitations/{id}")).await?;
            emit(&json!({ "ok": true, "cancelled": id, "org": org }), fmt)
        }
    }
}

async fn cmd_repo(c: RepoCmd, fmt: Format) -> Result<()> {
    let gh = GhClient::new(config::require_token()?)?;
    match c {
        RepoCmd::List { org } => {
            let v: Vec<Value> = gh.paginate(&format!("/orgs/{org}/repos?type=all")).await?;
            let rows: Vec<Value> = v
                .into_iter()
                .map(|r| {
                    json!({
                        "name": r["name"],
                        "full_name": r["full_name"],
                        "visibility": r["visibility"],
                        "default_branch": r["default_branch"],
                        "size_kb": r["size"],
                        "language": r["language"],
                        "pushed_at": r["pushed_at"],
                        "open_issues": r["open_issues_count"],
                        "html_url": r["html_url"],
                    })
                })
                .collect();
            emit(&rows, fmt)
        }
        RepoCmd::Show { full } => {
            let (o, r) = split_full(&full)?;
            let v: Value = gh.get(&format!("/repos/{o}/{r}")).await?;
            emit(&v, fmt)
        }
        RepoCmd::Create { org, name, visibility, description, no_init, gitignore, license } => {
            let mut body = json!({
                "name": name,
                "visibility": visibility,
                "auto_init": !no_init,
            });
            if let Some(d) = description { body["description"] = json!(d); }
            if let Some(g) = gitignore { body["gitignore_template"] = json!(g); }
            if let Some(l) = license { body["license_template"] = json!(l); }
            let r: Value = gh.post(&format!("/orgs/{org}/repos"), &body).await?;
            emit(&r, fmt)
        }
        RepoCmd::Delete { full, yes } => {
            if !yes {
                bail!("拒绝删除：删除仓库不可恢复。确认后加 --yes");
            }
            let (o, r) = split_full(&full)?;
            gh.delete(&format!("/repos/{o}/{r}")).await?;
            emit(&json!({ "ok": true, "deleted": full }), fmt)
        }
        RepoCmd::Branches { full } => {
            let (o, r) = split_full(&full)?;
            let v: Vec<Value> = gh.paginate(&format!("/repos/{o}/{r}/branches")).await?;
            let rows: Vec<Value> = v
                .into_iter()
                .map(|b| json!({ "name": b["name"], "protected": b["protected"] }))
                .collect();
            emit(&rows, fmt)
        }
        RepoCmd::Tree { full, path, r#ref } => {
            let (o, r) = split_full(&full)?;
            let mut url = format!("/repos/{o}/{r}/contents/{path}");
            if let Some(rf) = r#ref { url.push_str(&format!("?ref={rf}")); }
            let v: Value = gh.get(&url).await?;
            let arr: Vec<Value> = match v {
                Value::Array(a) => a,
                other => vec![other],
            };
            let rows: Vec<Value> = arr
                .into_iter()
                .map(|e| json!({
                    "name": e["name"],
                    "type": e["type"],
                    "size": e["size"],
                    "path": e["path"],
                }))
                .collect();
            emit(&rows, fmt)
        }
        RepoCmd::Cat { full, path, r#ref } => {
            let (o, r) = split_full(&full)?;
            let mut url = format!("/repos/{o}/{r}/contents/{path}");
            if let Some(rf) = r#ref { url.push_str(&format!("?ref={rf}")); }
            let v: Value = gh.get(&url).await?;
            if v.is_array() { bail!("path 是目录"); }
            if v["encoding"] == "base64" {
                let raw = v["content"].as_str().unwrap_or("").replace('\n', "");
                use base64::Engine;
                let bytes = base64::engine::general_purpose::STANDARD
                    .decode(&raw)
                    .context("base64 decode")?;
                let text = String::from_utf8_lossy(&bytes);
                print!("{text}");
                Ok(())
            } else {
                let text = v["content"].as_str().unwrap_or("");
                print!("{text}");
                Ok(())
            }
        }
        RepoCmd::Commits { full, r#ref, limit } => {
            let (o, r) = split_full(&full)?;
            let mut url = format!("/repos/{o}/{r}/commits?per_page={limit}");
            if let Some(rf) = r#ref { url.push_str(&format!("&sha={rf}")); }
            let v: Vec<Value> = gh.get(&url).await?;
            let rows: Vec<Value> = v
                .into_iter()
                .map(|c| {
                    let sha = c["sha"].as_str().unwrap_or("").to_string();
                    json!({
                        "sha": sha[..7.min(sha.len())].to_string(),
                        "message": c["commit"]["message"].as_str().unwrap_or("").lines().next().unwrap_or("").to_string(),
                        "author": c["commit"]["author"]["name"],
                        "date": c["commit"]["author"]["date"],
                        "html_url": c["html_url"],
                    })
                })
                .collect();
            emit(&rows, fmt)
        }
        RepoCmd::Issues { full, state } => {
            let (o, r) = split_full(&full)?;
            let v: Vec<Value> = gh
                .paginate(&format!("/repos/{o}/{r}/issues?state={state}"))
                .await?;
            let rows: Vec<Value> = v
                .into_iter()
                .filter(|i| i.get("pull_request").is_none() || i["pull_request"].is_null())
                .map(|i| {
                    json!({
                        "number": i["number"],
                        "state": i["state"],
                        "title": i["title"],
                        "user": i["user"]["login"],
                        "comments": i["comments"],
                        "created_at": i["created_at"],
                        "html_url": i["html_url"],
                    })
                })
                .collect();
            emit(&rows, fmt)
        }
        RepoCmd::Prs { full, state } => {
            let (o, r) = split_full(&full)?;
            let v: Vec<Value> = gh
                .paginate(&format!("/repos/{o}/{r}/pulls?state={state}&sort=updated&direction=desc"))
                .await?;
            let rows: Vec<Value> = v
                .into_iter()
                .map(|p| {
                    json!({
                        "number": p["number"],
                        "state": p["state"],
                        "draft": p["draft"],
                        "merged": p["merged_at"].is_string(),
                        "title": p["title"],
                        "user": p["user"]["login"],
                        "head": p["head"]["ref"],
                        "base": p["base"]["ref"],
                        "html_url": p["html_url"],
                    })
                })
                .collect();
            emit(&rows, fmt)
        }
        RepoCmd::Collab { full, login, permission } => {
            let (o, r) = split_full(&full)?;
            gh.put(
                &format!("/repos/{o}/{r}/collaborators/{login}"),
                &json!({ "permission": permission }),
            )
            .await?;
            emit(&json!({ "ok": true, "added": login, "permission": permission }), fmt)
        }
        RepoCmd::Uncollab { full, login } => {
            let (o, r) = split_full(&full)?;
            gh.delete(&format!("/repos/{o}/{r}/collaborators/{login}")).await?;
            emit(&json!({ "ok": true, "removed": login }), fmt)
        }
    }
}

async fn cmd_activity(c: ActivityCmd, fmt: Format) -> Result<()> {
    let gh = GhClient::new(config::require_token()?)?;
    match c {
        ActivityCmd::Events { org, limit } => {
            let v: Vec<Value> = gh
                .get(&format!("/orgs/{org}/events?per_page={limit}"))
                .await?;
            let rows: Vec<Value> = v
                .into_iter()
                .map(|e| {
                    json!({
                        "id": e["id"],
                        "type": e["type"],
                        "actor": e["actor"]["login"],
                        "repo": e["repo"]["name"],
                        "created_at": e["created_at"],
                    })
                })
                .collect();
            emit(&rows, fmt)
        }
    }
}

fn cmd_dashboard(org: Option<String>) -> Result<()> {
    let url = match org {
        Some(o) => format!("https://github.yangtzeu.work/admin/{o}"),
        None => "https://github.yangtzeu.work/admin".to_string(),
    };
    print_message(&format!("打开 {url}"));
    #[cfg(target_os = "macos")]
    { std::process::Command::new("open").arg(&url).spawn().ok(); }
    #[cfg(target_os = "linux")]
    { std::process::Command::new("xdg-open").arg(&url).spawn().ok(); }
    Ok(())
}
