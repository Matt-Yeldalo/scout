use crate::request::{Request, Response};
use anyhow::Result;

pub async fn send(request: Request) -> Result<Response> {
    let req_builder = build_request(&request)?;
    let start_time = std::time::Instant::now();
    let resp = req_builder.send().await?;
    let duration = start_time.elapsed();

    let status = resp.status();
    let status_text = status.canonical_reason().unwrap_or("").to_string();
    let headers = resp
        .headers()
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
        .collect();
    let body = resp.text().await?;

    Ok(Response {
        status: status.as_u16(),
        status_text,
        headers,
        body,
        duration_ms: duration.as_millis(),
    })
}

fn build_request(request: &Request) -> Result<reqwest::RequestBuilder> {
    let client = reqwest::Client::new();
    let mut req_builder = match request.method {
        crate::request::HttpMethod::Get => client.get(&request.url),
        crate::request::HttpMethod::Post => client.post(&request.url),
    };

    for (key, value) in &request.headers {
        req_builder = req_builder.header(key, value);
    }

    if let Some(body) = &request.body {
        req_builder = req_builder.body(body.clone());
    }

    match &request.auth {
        crate::request::Auth::Bearer(token) => {
            req_builder = req_builder.bearer_auth(token);
        }
        crate::request::Auth::Basic { username, password } => {
            req_builder = req_builder.basic_auth(username, Some(password));
        }
        crate::request::Auth::ApiKey { header, value } => {
            req_builder = req_builder.header(header, value);
        }
        _ => {}
    }

    Ok(req_builder)
}
