// NOTE: Removal requires diesel 2.0+
#[macro_use]
extern crate diesel;

use std::thread;
use std::time::Duration;

use diesel::prelude::*;
use dotenv::dotenv;

mod github;
mod model;
mod schema;
mod webserver;

use github::Notification;
use model::{Job, NewJob};

/// Text identifying that the bot has been mentioned.
const BOT_MENTION: &str = "@perfbot";

/// Time between notification updates.
const NOTIFICATION_FREQUENCY: Duration = Duration::from_secs(30);

#[tokio::main]
async fn main() {
    let _ = dotenv();

    thread::spawn(watch_notifications);

    webserver::launch().await.expect("webserver crash");
}

/// Watch for new GitHub notifications.
fn watch_notifications() -> ! {
    let connection = model::db_connection();

    loop {
        // Handle all new benchmark requests.
        for notification in Notification::all() {
            process_notification(&connection, notification);
        }

        // Retry outdated jobs.
        Job::update_stale(&connection);

        thread::sleep(NOTIFICATION_FREQUENCY);
    }
}

/// Process a single GitHub notification.
///
/// This will check if a benchmark request is valid and then insert it into the database.
fn process_notification(connection: &SqliteConnection, notification: Notification) {
    // TODO
    // Remove the notification.
    // let notification = notification.read();
    // notification.unsubscribe();

    // Only process mentions on PRs.
    let pull_request = match notification.pull_request() {
        Ok(pull_request) => pull_request,
        _ => return,
    };

    // Skip notifications without valid benchmark requests.
    if !pull_request.comments().iter().rev().any(|comment| {
        // TODO: Authorize bot to read org users
        // comment.author_association >= UserAssociation::COLLABORATOR
        notification
            .last_read_at
            .as_ref()
            .map_or(true, |last_read_at| &comment.created_at > last_read_at)
            && comment.body.contains(BOT_MENTION)
    }) {
        return;
    }

    // Schedule the job.
    let repository = notification.repository.full_name;
    let comments_url = pull_request.comments_url;
    let hash = pull_request.merge_commit_sha;
    NewJob::new(comments_url, repository, hash).insert(connection);
}
