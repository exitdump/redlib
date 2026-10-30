// JSON API: serves Reddit's raw JSON for any page path, without the HTML UI.
//
//   /api/r/rust/hot          -> /r/rust/hot.json
//   /api/comments/abc123     -> /comments/abc123.json
//   /api/user/spez/submitted -> /user/spez/submitted.json
//   /api/search?q=docker     -> /search.json?q=docker
//
// Kept in its own file so upstream merges stay conflict free.

use crate::backends;
use crate::body::Body;
use hyper::{Request, Response};
use serde_json::json;

/// Maps `/api/<path>?<query>` to the Reddit JSON path the backends expect.
fn reddit_path(req: &Request<Body>) -> String {
	let path = req.uri().path().strip_prefix("/api").unwrap_or_default().trim_end_matches('/');
	let path = path.strip_suffix(".json").unwrap_or(path);
	let path = if path.is_empty() { "/hot" } else { path };
	let query = req.uri().query().unwrap_or_default();
	format!("{path}.json?{query}&raw_json=1")
}

fn respond(status: u16, body: String, backend: &str) -> Response<Body> {
	Response::builder()
		.status(status)
		.header("content-type", "application/json")
		.header("Access-Control-Allow-Origin", "*")
		.header("Cache-Control", "no-store")
		.header("X-Redlib-Backend", backend)
		.body(body.into())
		.unwrap_or_default()
}

pub async fn handler(req: Request<Body>) -> Result<Response<Body>, String> {
	let path = reddit_path(&req);
	Ok(match backends::fetch(path.clone(), false).await {
		Ok((json, backend)) => respond(200, json.to_string(), backend),
		Err(msg) => {
			let status = if backends::is_blocked_error(&msg) { 503 } else { 502 };
			respond(status, json!({ "error": msg, "path": path }).to_string(), "none")
		}
	})
}
