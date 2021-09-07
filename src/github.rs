/// TODO: Doc everything
use std::error::Error;

use serde::{Deserialize, Serialize};
use ureq::Response;

const USER_AGENT_HEADER: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
const AUTHORIZATION_HEADER: &str = include_str!("../.auth");
const ACCEPT_HEADER: &str = "application/vnd.github.v3+json";

/// Retrieve all open user notifications.
pub fn notifications() -> Vec<Notification> {
    let url = "https://api.github.com/notifications";
    json_request("GET", url).unwrap_or_default()
}

/// GitHub notification.
#[derive(Serialize, Deserialize, Debug)]
pub struct Notification {
    id: String,
    unread: bool,
    reason: Reason,
    updated_at: String,
    last_read_at: Option<String>,
    subject: Subject,
    repository: Repository,
    url: String,
    subscription_url: String,
}

impl Notification {
    /// Mark a notification as read.
    pub fn read(self) -> Self {
        json_request("PATCH", &self.url).unwrap_or(self)
    }

    /// Remove this notification's subscription.
    pub fn unsubscribe(&self) {
        let _ = request("DELETE", &self.subscription_url);
    }
}

/// Notification subject.
#[derive(Serialize, Deserialize, Debug)]
pub struct Subject {
    title: String,
    url: String,
    latest_comment_url: String,
    r#type: Type,
}

/// Notification reasons.
#[derive(Serialize, Deserialize, Debug)]
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
#[derive(Serialize, Deserialize, Debug)]
pub enum Type {
    PullRequest,
    Commit,
    Issue,
}

/// Subscription for GitHub notifications.
#[derive(Serialize, Deserialize, Debug)]
pub struct Subscription {
    subscribed: bool,
    ignored: bool,
    reason: Option<Reason>,
    created_at: String,
    url: String,
    thread_url: String,
}

/// GitHub repository.
#[derive(Serialize, Deserialize, Debug)]
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

/// Send a request with a JSON response.
fn json_request<T>(method: &str, url: &str) -> Result<T, Box<dyn Error>>
where
    T: for<'de> Deserialize<'de>,
{
    Ok(request(method, url)?.into_json()?)
}

/// Send a request.
fn request(method: &str, url: &str) -> Result<Response, Box<dyn Error>> {
    Ok(ureq::request(method, url)
        .set("authorization", AUTHORIZATION_HEADER)
        .set("user-agent", USER_AGENT_HEADER)
        .set("accept", ACCEPT_HEADER)
        .call()?)
}
