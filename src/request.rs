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

impl From<&AuthDraft> for Auth {
    fn from(d: &AuthDraft) -> Self {
        match d.kind {
            AuthKind::None => Auth::None,
            AuthKind::Bearer => Auth::Bearer(d.bearer_token.clone()),
            AuthKind::Basic => Auth::Basic {
                username: d.basic_username.clone(),
                password: d.basic_password.clone(),
            },
            AuthKind::ApiKey => Auth::ApiKey {
                header: d.api_key_header.clone(),
                value: d.api_key_value.clone(),
            },
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub enum AuthKind {
    #[default]
    None,
    Bearer,
    Basic,
    ApiKey,
}

#[derive(Debug, Default, Clone)]
pub struct AuthDraft {
    pub kind: AuthKind,
    pub bearer_token: String,
    pub basic_username: String,
    pub basic_password: String,
    pub api_key_header: String,
    pub api_key_value: String,
}

impl From<&Auth> for AuthDraft {
    fn from(a: &Auth) -> Self {
        let mut d = AuthDraft::default();
        match a {
            Auth::None => {}
            Auth::Bearer(t) => {
                d.kind = AuthKind::Bearer;
                d.bearer_token = t.clone();
            }
            Auth::Basic { username, password } => {
                d.kind = AuthKind::Basic;
                d.basic_username = username.clone();
                d.basic_password = password.clone();
            }
            Auth::ApiKey { header, value } => {
                d.kind = AuthKind::ApiKey;
                d.api_key_header = header.clone();
                d.api_key_value = value.clone();
            }
        }
        d
    }
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

    pub fn populate_placeholders(&mut self, variables: &HashMap<String, String>) -> Result<(), String> {
        let url = self.url.as_str();
        let variables = variables.clone();
        let url = subst::substitute(&url, &variables);
        match url {
            Ok(u) => {
                self.url = u;
                Ok(())
            }
            _ => Err("Failed to populate placeholders".to_string()),
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
        // write!(f, "{} {} ({})", method, self.url, self.name)
        write!(f, "{} - {}", method, self.name)
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
