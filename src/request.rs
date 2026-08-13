use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub enum HttpMethod {
    #[default]
    Get,
    Post,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub enum Auth {
    #[default]
    None,
    Bearer(String),
    Basic {
        username: String,
        password: String,
    },
    ApiKey {
        header: String,
        value: String,
    },
}

// TODO: add ability to lock to prevent save_all from overwriting
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Request {
    pub name: String,
    pub method: HttpMethod,
    pub url: String,
    pub headers: Vec<(String, String)>,
    #[serde(default)]
    pub body: Option<String>,
    pub auth: Auth,
    pub id: String,
}

impl Request {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            method: HttpMethod::Get,
            url: String::new(),
            headers: Vec::new(),
            body: None,
            auth: Auth::None,
            id: uuid::Uuid::new_v4().to_string(),
        }
    }
}

impl std::fmt::Display for Request {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let method = match self.method {
            HttpMethod::Get => "GET",
            HttpMethod::Post => "POST",
        };

        // write!(
        //     f,
        //     "id: {}\nn: {}\nm: {}\nu: {}\nh: {:?}\nb: {:?}\na: {:?}",
        //     self.id, self.name, method, self.url, self.headers, self.body, self.auth
        // )
        write!(f, "{} {} ({})", method, self.url, self.name)
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Response {
    pub status: u16,
    pub status_text: String,
    pub headers: HashMap<String, String>,
    pub body: String,
    pub duration_ms: u128,
}
