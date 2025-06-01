use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT, HeaderName};
use std::collections::HashMap;

pub(crate) async fn fetch_with_query(
    url: &str,
    params: Option<&HashMap<&str, &str>>,
    extra_headers: Option<HashMap<String, String>>,
) -> Result<String, reqwest::Error> {
    let client = reqwest::Client::new();

    // Build headers with default User-Agent
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("MyRustClient/0.1"));

    if let Some(hdrs) = extra_headers {
        for (key, val) in hdrs {
            headers.insert(
                key.parse::<HeaderName>().unwrap(),
                HeaderValue::from_str(&val).unwrap(),
            );
        }
    }

    let req = client.get(url).headers(headers);

    // Add query params if provided
    let req = if let Some(p) = params {
        req.query(p)
    } else {
        req
    };

    let res = req.send().await?;

    let body = res.text().await?;
    Ok(body)
}

