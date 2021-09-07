/// TODO: Doc everything
use std::convert::TryFrom;
use std::error::Error;

use hyper::body::Buf;
use hyper::{body, header, http, Body, Client, Method, Request, Uri};
use hyper_tls::HttpsConnector;
use serde::de::DeserializeOwned;
use serde::Deserialize;

const USER_AGENT_HEADER: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
const AUTHORIZATION_HEADER: &str = include_str!("../.auth");
const ACCEPT_HEADER: &str = "application/vnd.github.v3+json";

/// Retrieve all open user notifications.
pub async fn notifications() -> Vec<Notification> {
    let url = "https://api.github.com/notifications";
    json_request(Method::GET, url).await.unwrap_or_default()
}

/// GitHub notification.
#[derive(Deserialize, Debug)]
pub struct Notification {
    id: String,
    unread: bool,
    reason: Reason,
    updated_at: String,
    last_read_at: String,
    subject: Subject,
    repository: Repository,
    url: String,
    subscription_url: String,
}

impl Notification {
    /// Mark a notification as read.
    pub async fn read(self) -> Self {
        json_request(Method::PATCH, &self.url).await.unwrap_or(self)
    }

    /// Remove this notification's subscription.
    pub async fn unsubscribe(&self) -> Result<(), Box<dyn Error>> {
        send_request(Method::DELETE, &self.subscription_url).await?;
        Ok(())
    }
}

/// Notification subject.
#[derive(Deserialize, Debug)]
pub struct Subject {
    title: String,
    url: String,
    latest_comment_url: String,
    r#type: Type,
}

/// Notification reasons.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Reason {
    Assign,
    Author,
    Comment,
    Invitation,
    Manual,
    Mention,
    ReviewRequested,
    SecurityAlert,
    StateChange,
    Subscribed,
    TeamMention,
}

/// Notification types.
#[derive(Deserialize, Debug)]
pub enum Type {
    PullRequest,
    Commit,
    Issue,
}

/// Subscription for GitHub notifications.
#[derive(Deserialize, Debug)]
pub struct Subscription {
    subscribed: bool,
    ignored: bool,
    reason: Option<Reason>,
    created_at: String,
    url: String,
    thread_url: String,
}

/// GitHub repository.
#[derive(Deserialize, Debug)]
pub struct Repository {
    id: usize,
    node_id: String,
    name: String,
    full_name: String,
    private: bool,
    description: String,
    fork: bool,
    // Some unnecessary fields have been omitted.
}

async fn json_request<U, T>(method: Method, url: U) -> Result<T, Box<dyn Error>>
where
    Uri: TryFrom<U>,
    http::Error: From<<Uri as TryFrom<U>>::Error>,
    T: DeserializeOwned,
{
    // TODO: Log plaintext json for deserialization failure?
    let body = send_request(method, url).await?;
    let parsed = serde_json::from_reader(body.reader())?;
    Ok(parsed)
}

async fn send_request<U>(method: Method, url: U) -> Result<impl Buf, Box<dyn Error>>
where
    Uri: TryFrom<U>,
    http::Error: From<<Uri as TryFrom<U>>::Error>,
{
    let client = Client::builder().build::<_, Body>(HttpsConnector::new());

    let request = Request::builder()
        .header(header::USER_AGENT, USER_AGENT_HEADER)
        .header(header::AUTHORIZATION, AUTHORIZATION_HEADER)
        .header(header::ACCEPT, ACCEPT_HEADER)
        .method(method)
        .uri(url)
        .body(Body::default())?;

    let response = client.request(request).await?;
    let body = body::aggregate(response).await?;

    Ok(body)
}
