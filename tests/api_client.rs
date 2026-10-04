//! 本地契约测试：进程内 mock server 验证传输层（无外部网络）。
//! 覆盖：请求头（Cookie / User-Agent / 不发送 Content-Type）、JSON 解析、统一错误体解析与退出码映射。

use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::mpsc::{channel, Receiver};

use geek_cli::api::{ApiError, Client};

fn http_response(status: &str, body: &str) -> String {
    format!(
        "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
}

/// 起一个只服务一次请求的 mock server，返回 (base, 收到的请求文本)。
fn spawn_server(response: String) -> (String, Receiver<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let addr = listener.local_addr().expect("addr");
    let (tx, rx) = channel();
    std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept");
        let mut buf = [0u8; 4096];
        let n = stream.read(&mut buf).expect("read");
        let request = String::from_utf8_lossy(&buf[..n]).to_string();
        stream.write_all(response.as_bytes()).expect("write");
        let _ = stream.flush();
        let _ = tx.send(request);
    });
    (format!("http://{addr}"), rx)
}

#[tokio::test]
async fn sends_session_cookie_and_parses_json() {
    let (base, rx) = spawn_server(http_response("200 OK", r#"{"ok":true,"ts":1}"#));
    let client = Client::new(&base, Some("test-sid".to_string())).expect("client");
    let value: serde_json::Value = client.get("/healthz").await.expect("get");
    assert_eq!(value["ok"], true);

    let request = rx.recv().expect("request captured");
    assert!(request.starts_with("GET /healthz "), "path: {request}");
    let lower = request.to_lowercase();
    assert!(lower.contains("cookie: sid=test-sid"), "cookie: {request}");
    assert!(lower.contains("user-agent: geek-cli/"), "ua: {request}");
    assert!(
        lower.contains("accept: application/json"),
        "accept: {request}"
    );
}

#[tokio::test]
async fn maps_unified_error_body_to_exit_code() {
    let (base, _rx) = spawn_server(http_response(
        "403 Forbidden",
        r#"{"error":"missing_capability","message":"需要「查看投递」权限","request_id":"r1"}"#,
    ));
    let client = Client::new(&base, None).expect("client");
    let error = client
        .get::<serde_json::Value>("/api/console/me")
        .await
        .expect_err("403");
    let api = error.downcast_ref::<ApiError>().expect("api error");
    assert_eq!(api.error, "missing_capability");
    assert_eq!(api.message, "需要「查看投递」权限");
    assert_eq!(api.request_id.as_deref(), Some("r1"));
    assert_eq!(api.exit_code(), 4);
}

#[tokio::test]
async fn empty_post_sends_no_content_type() {
    let (base, rx) = spawn_server(http_response("200 OK", r#"{"ok":true}"#));
    let client = Client::new(&base, None).expect("client");
    let _: serde_json::Value = client.post_empty("/auth/signout").await.expect("post");
    let request = rx.recv().expect("request captured");
    assert!(
        !request.to_lowercase().contains("content-type"),
        "should not send content-type: {request}"
    );
}

#[tokio::test]
async fn patch_sends_json_body_and_content_type() {
    let (base, rx) = spawn_server(http_response("200 OK", r#"{"ok":true}"#));
    let client = Client::new(&base, Some("s".into())).expect("client");
    let body = serde_json::json!({"status": "interview"});
    let _: serde_json::Value = client
        .patch_json("/api/console/applications/x", &body)
        .await
        .expect("patch");
    let request = rx.recv().expect("request captured");
    assert!(
        request.starts_with("PATCH /api/console/applications/x "),
        "{request}"
    );
    let lower = request.to_lowercase();
    assert!(
        lower.contains("content-type: application/json"),
        "{request}"
    );
    assert!(request.contains(r#"{"status":"interview"}"#), "{request}");
}

#[tokio::test]
async fn delete_sends_no_content_type() {
    let (base, rx) = spawn_server(http_response("200 OK", r#"{"ok":true}"#));
    let client = Client::new(&base, None).expect("client");
    let _: serde_json::Value = client
        .delete_json("/api/console/feedback/1")
        .await
        .expect("delete");
    let request = rx.recv().expect("request captured");
    assert!(
        request.starts_with("DELETE /api/console/feedback/1 "),
        "{request}"
    );
    assert!(
        !request.to_lowercase().contains("content-type"),
        "{request}"
    );
}
