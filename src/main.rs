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

use github::{Notification, Repository, UserAssociation};
use model::{Job, MasterBuild, NewJob, NewMasterBuild};

/// Time between notification updates.
const NOTIFICATION_FREQUENCY: Duration = Duration::from_secs(30);

/// List of repos tracked for benchmark requests.
const REPO_WHITELIST: [&str; 1] = ["alacritty/alacritty"];

/// Text identifying that the bot has been mentioned.
const BOT_MENTION: &str = "@perfbot";

#[tokio::main]
async fn main() {
    let _ = dotenv();

    thread::spawn(watch_notifications);
    thread::spawn(watch_alacritty_master);

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

/// Watch for new commits on master.
fn watch_alacritty_master() -> ! {
    let repository = Repository::get("alacritty", "alacritty").expect("repo not found");

    let connection = model::db_connection();

    loop {
        process_alacritty_master(&connection, &repository);
        thread::sleep(NOTIFICATION_FREQUENCY);
    }
}

/// Check the alacritty master for new updates.
fn process_alacritty_master(connection: &SqliteConnection, repository: &Repository) {
    let last_build = MasterBuild::latest(connection);
    let commits = repository.commits();

    // Skip if this commit has already been benchmarked.
    if commits.is_empty() || last_build.map_or(false, |build| commits[0].sha == build.hash) {
        return;
    }

    // Stage job.
    let repository = repository.full_name.clone();
    NewJob::new(repository, None, None).insert(connection);

    // Update latest master build.
    NewMasterBuild::new(commits[0].sha.clone()).insert(connection);
}

/// Process a single GitHub notification.
///
/// This will check if a benchmark request is valid and then insert it into the database.
fn process_notification(connection: &SqliteConnection, notification: Notification) {
    // Remove the notification.
    let notification = notification.read();
    notification.unsubscribe();

    // Only process mentions on PRs.
    let pull_request = match notification.pull_request() {
        Ok(pull_request) => pull_request,
        _ => return,
    };

    // Skip notifications without valid benchmark requests.
    if !pull_request.comments().iter().rev().any(|comment| {
        comment.author_association >= UserAssociation::COLLABORATOR
            && notification
                .last_read_at
                .as_ref()
                .map_or(true, |last_read_at| &comment.created_at > last_read_at)
            && REPO_WHITELIST.contains(&notification.repository.full_name.as_str())
            && comment.body.contains(BOT_MENTION)
    }) {
        return;
    }

    // Schedule the job.
    let repository = notification.repository.full_name;
    let comments_url = pull_request.comments_url;
    let hash = pull_request.merge_commit_sha;
    NewJob::new(repository, comments_url, hash).insert(connection);
}
