//! GitHub API.

use std::cmp::Ordering;
use std::error::Error;

use chrono::DateTime;
use chrono::offset::Utc;
use serde::{Deserialize, Serialize};
use serde_json::json;
use ureq::Response;

/// User agent identifying this application.
///
/// A valid user agent is required by the GitHub API.
const USER_AGENT_HEADER: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

/// User Token for GitHub API authentication.
const AUTHORIZATION_HEADER: &str = include_str!("../.auth");

/// JSON accept header following GitHub API recommendations.
const ACCEPT_HEADER: &str = "application/vnd.github.v3+json";

/// GitHub notification.
#[derive(Serialize, Deserialize, Debug)]
pub struct Notification {
    pub id: String,
    pub unread: bool,
    pub reason: Reason,
    pub updated_at: DateTime<Utc>,
    pub last_read_at: Option<DateTime<Utc>>,
    pub subject: Subject,
    pub repository: Repository,
    pub url: String,
    pub subscription_url: String,
}

impl Notification {
    /// Retrieve all unread user notifications.
    pub fn all() -> Vec<Notification> {
        json_request("GET", "https://api.github.com/notifications", ()).unwrap_or_default()
    }

    /// Mark the notification as read.
    ///
    /// Marking a notification as read will remove it from the list of notifications retrieved by
    /// [`notifications`].
    pub fn read(self) -> Self {
        json_request("PATCH", &self.url, ()).unwrap_or(self)
    }

    /// Remove this notification's subscription.
    pub fn unsubscribe(&self) {
        let _ = request("DELETE", &self.subscription_url, ());
    }

    /// Get the subscription's PR.
    ///
    /// # Errors
    ///
    /// This will return an error if the subscription's subject is not a pull request.
    pub fn pull_request(&self) -> Result<PullRequest, Box<dyn Error>> {
        if self.subject.r#type != SubjectType::PullRequest {
            return Err("subscription is not a pull request")?;
        }

        json_request("GET", &self.subject.url, ())
    }
}

/// Notification subject.
#[derive(Serialize, Deserialize, Debug)]
pub struct Subject {
    pub title: String,
    pub url: String,
    pub latest_comment_url: String,
    pub r#type: SubjectType,
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
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum SubjectType {
    PullRequest,
    Commit,
    Issue,
}

/// Subscription for GitHub notifications.
#[derive(Serialize, Deserialize, Debug)]
pub struct Subscription {
    pub subscribed: bool,
    pub ignored: bool,
    pub reason: Option<Reason>,
    pub created_at: DateTime<Utc>,
    pub url: String,
    pub thread_url: String,
}

/// GitHub repository.
#[derive(Serialize, Deserialize, Debug)]
pub struct Repository {
    pub id: usize,
    pub name: String,
    pub full_name: String,
    pub private: bool,
    pub description: String,
    pub fork: bool,
    // Some unnecessary fields have been omitted.
}

/// GitHub pull request.
#[derive(Serialize, Deserialize, Debug)]
pub struct PullRequest {
    pub url: String,
    pub id: usize,
    pub merge_commit_sha: String,
    pub comments_url: String,
    // Some unnecessary fields have been omitted.
}

impl PullRequest {
    /// Get the PR's comments.
    pub fn comments(&self) -> Vec<Comment> {
        json_request("GET", &self.comments_url, ()).unwrap_or_default()
    }
}

/// GitHub comment.
#[derive(Serialize, Deserialize, Debug)]
pub struct Comment {
    pub url: String,
    pub id: usize,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub author_association: UserAssociation,
    pub body: String,
    // Some unnecessary fields have been omitted.
}

impl Comment {
    /// Create a new comment.
    pub fn new(url: &str, body: impl Into<String>) -> Result<Self, Box<dyn Error>> {
        json_request("POST", url, CommentRequest { body: body.into() })
    }
}

/// Request payload for new comments.
#[derive(Serialize)]
struct CommentRequest {
    body: String,
}

/// User's association with a repository.
#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, Eq)]
pub enum UserAssociation {
    NONE,
    CONTRIBUTOR,
    COLLABORATOR,
    MEMBER,
    OWNER,
}

impl PartialOrd for UserAssociation {
    #[inline]
    fn partial_cmp(&self, other: &UserAssociation) -> Option<Ordering> {
        Some((*self as usize).cmp(&(*other as usize)))
    }
}

/// Send a request with a JSON response.
fn json_request<T, B>(method: &str, url: &str, body: B) -> Result<T, Box<dyn Error>>
where
    T: for<'de> Deserialize<'de>,
    B: Serialize,
{
    Ok(request(method, url, body)?.into_json()?)
}

/// Send a request.
fn request<B>(method: &str, url: &str, body: B) -> Result<Response, Box<dyn Error>>
where
    B: Serialize,
{
    Ok(ureq::request(method, url)
        .set("authorization", AUTHORIZATION_HEADER)
        .set("user-agent", USER_AGENT_HEADER)
        .set("accept", ACCEPT_HEADER)
        .send_json(json!(body))?)
}
