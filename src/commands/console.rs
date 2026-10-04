//! 控制台命令（P1）：只读面 + 写操作（投递审核 / 意见处理）。

use anyhow::Result;
use serde_json::Value;

use crate::api::console as console_api;
use crate::api::console::{ApplicationReviewBody, FeedbackUpdateBody, ReviewLetter};
use crate::api::UsageError;
use crate::commands::{confirm, plan, session_client, WriteOpts};
use crate::output::{emit, Format};

/// `geek console me`：控制台身份 / 称号 / 能力（GET /api/console/me）。
pub async fn me(base: &str, fmt: Format) -> Result<()> {
    let client = session_client(base)?;
    emit(&console_api::me(&client).await?, fmt)
}

/// `geek console summary`：概览统计（按本人能力返回子集）。
pub async fn summary(base: &str, fmt: Format) -> Result<()> {
    let client = session_client(base)?;
    emit(&console_api::summary(&client).await?, fmt)
}

/// `geek console catalogue`：配置类只读数据。
pub async fn catalogue(base: &str, fmt: Format) -> Result<()> {
    let client = session_client(base)?;
    emit(&console_api::catalogue(&client).await?, fmt)
}

/// `geek console people`：成员全名单（含称号）。
pub async fn people(base: &str, fmt: Format) -> Result<()> {
    let client = session_client(base)?;
    emit(&console_api::people(&client).await?, fmt)
}

/// `geek console department list`：部门列表。
pub async fn departments(base: &str, fmt: Format) -> Result<()> {
    let client = session_client(base)?;
    emit(&console_api::departments(&client).await?, fmt)
}

/// `geek console assignment list [--department --role]`：指派列表。
pub async fn assignments(
    base: &str,
    department_id: Option<&str>,
    role: Option<&str>,
    fmt: Format,
) -> Result<()> {
    let client = session_client(base)?;
    emit(
        &console_api::assignments(&client, department_id, role).await?,
        fmt,
    )
}

/// `geek console application list [--status --q --limit --offset]`：投递列表。
pub async fn applications(
    base: &str,
    status: Option<&str>,
    q: Option<&str>,
    limit: Option<u32>,
    offset: Option<u32>,
    fmt: Format,
) -> Result<()> {
    let client = session_client(base)?;
    emit(
        &console_api::applications(&client, status, q, limit, offset).await?,
        fmt,
    )
}

/// `geek console application show <uuid>`：投递详情（含审核历史）。
pub async fn application(base: &str, application_id: &str, fmt: Format) -> Result<()> {
    let client = session_client(base)?;
    emit(
        &console_api::application(&client, application_id).await?,
        fmt,
    )
}

/// `geek console feedback list [--status --limit]`：意见箱列表。
pub async fn feedback(
    base: &str,
    status: Option<&str>,
    limit: Option<u32>,
    fmt: Format,
) -> Result<()> {
    let client = session_client(base)?;
    emit(&console_api::feedback(&client, status, limit).await?, fmt)
}

/// `geek console audit [--limit --offset --action]`：审计日志（脱敏）。
pub async fn audit(
    base: &str,
    limit: Option<u32>,
    offset: Option<u32>,
    action: Option<&str>,
    fmt: Format,
) -> Result<()> {
    let client = session_client(base)?;
    emit(
        &console_api::audit(&client, limit, offset, action).await?,
        fmt,
    )
}

/// `geek console application review` 的参数。
#[derive(Debug, Clone)]
pub struct ReviewArgs {
    pub status: String,
    pub expected_status: Option<String>,
    pub expected_review_id: Option<u64>,
    pub note: Option<String>,
    pub notify: bool,
    pub time: Option<String>,
    pub place: Option<String>,
    pub notes: Option<String>,
    pub message: Option<String>,
}

/// 从投递详情推导 `(expected_status, expected_review_id)`；无审核记录时 review_id = 0（服务端约定）。
fn derive_expected(detail: &Value) -> (Option<String>, u64) {
    let status = detail["application"]["status"].as_str().map(str::to_string);
    let review_id = detail["reviews"]
        .as_array()
        .map(|reviews| {
            reviews
                .iter()
                .filter_map(|r| r["id"].as_u64())
                .max()
                .unwrap_or(0)
        })
        .unwrap_or(0);
    (status, review_id)
}

/// `geek console application review <uuid>`：改状态 / 写备注 / 可选发信（需 `--yes`；`--dry-run` 只打印请求）。
pub async fn application_review(
    base: &str,
    application_id: &str,
    args: ReviewArgs,
    opts: &WriteOpts,
    fmt: Format,
) -> Result<()> {
    let client = session_client(base)?;

    // 未显式给出的 expected_* 用「先读一次」补齐，防止陈旧页面覆盖他人审核。
    let (expected_status, expected_review_id) =
        if args.expected_status.is_none() || args.expected_review_id.is_none() {
            let detail = console_api::application(&client, application_id).await?;
            let (status, review_id) = derive_expected(&detail);
            (
                args.expected_status.clone().or(status),
                Some(args.expected_review_id.unwrap_or(review_id)),
            )
        } else {
            (args.expected_status.clone(), args.expected_review_id)
        };

    let letter = ReviewLetter {
        time: args.time,
        place: args.place,
        notes: args.notes,
        message: args.message,
    };
    let body = ApplicationReviewBody {
        status: args.status.clone(),
        expected_status,
        expected_review_id,
        note: args.note,
        notify: if args.notify { None } else { Some(false) },
        letter: if letter.is_empty() {
            None
        } else {
            Some(letter)
        },
    };

    let path = format!(
        "/api/console/applications/{}",
        urlencoding::encode(application_id)
    );
    let value = serde_json::to_value(&body)?;
    if plan(opts, "PATCH", &path, Some(&value))? {
        return Ok(());
    }
    confirm(
        &format!(
            "将投递 {application_id} 的状态改为 {}（可能给投递人发通知信）",
            body.status
        ),
        opts,
    )?;
    emit(
        &console_api::application_review(&client, application_id, &body).await?,
        fmt,
    )
}

/// `geek console feedback reply <id> [--status --reply]`：改意见状态 / 回复（至少一项；需 `--yes`）。
pub async fn feedback_reply(
    base: &str,
    id: u64,
    status: Option<&str>,
    reply: Option<&str>,
    opts: &WriteOpts,
    fmt: Format,
) -> Result<()> {
    if status.is_none() && reply.is_none() {
        return Err(UsageError("至少提供 --status 或 --reply 之一".into()).into());
    }
    let client = session_client(base)?;
    let body = FeedbackUpdateBody {
        status: status.map(str::to_string),
        reply: reply.map(str::to_string),
    };
    let path = format!("/api/console/feedback/{id}");
    let value = serde_json::to_value(&body)?;
    if plan(opts, "PATCH", &path, Some(&value))? {
        return Ok(());
    }
    confirm(&format!("将更新意见 #{id}"), opts)?;
    emit(
        &console_api::feedback_update(&client, id, &body).await?,
        fmt,
    )
}

/// `geek console feedback delete <id>`：删除意见（不可恢复；需 `--yes`）。
pub async fn feedback_delete(base: &str, id: u64, opts: &WriteOpts, fmt: Format) -> Result<()> {
    let client = session_client(base)?;
    let path = format!("/api/console/feedback/{id}");
    if plan(opts, "DELETE", &path, None)? {
        return Ok(());
    }
    confirm(&format!("将删除意见 #{id}（不可恢复）"), opts)?;
    emit(&console_api::feedback_delete(&client, id).await?, fmt)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derives_expected_from_detail() {
        let detail = serde_json::json!({
            "application": {"status": "received"},
            "reviews": [{"id": 3}, {"id": 5}]
        });
        assert_eq!(derive_expected(&detail), (Some("received".into()), 5));

        let empty = serde_json::json!({"application": {"status": "interview"}, "reviews": []});
        assert_eq!(derive_expected(&empty), (Some("interview".into()), 0));
    }
}
