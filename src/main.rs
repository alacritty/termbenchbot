use std::time::Duration;
use std::{env, thread};

use diesel::prelude::*;
use dotenv::dotenv;
use tracing::debug;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

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
    // Setup logging.
    let directives = env::var("RUST_LOG").unwrap_or("warn,perfbot=debug,rocket=info".into());
    let env_filter = EnvFilter::builder().parse_lossy(directives);
    FmtSubscriber::builder().with_env_filter(env_filter).with_line_number(true).init();

    trace_error!(dotenv());

    thread::spawn(watch_notifications);
    thread::spawn(watch_alacritty_master);

    webserver::launch().await.expect("webserver crash");
}

/// Watch for new GitHub notifications.
fn watch_notifications() -> ! {
    let mut connection = model::db_connection();

    loop {
        // Handle all new benchmark requests.
        for notification in Notification::all() {
            process_notification(&mut connection, notification);
        }

        // Retry outdated jobs.
        Job::update_stale(&mut connection);

        thread::sleep(NOTIFICATION_FREQUENCY);
    }
}

/// Watch for new commits on master.
fn watch_alacritty_master() -> ! {
    let repository = Repository::get("alacritty", "alacritty").expect("repo not found");

    let mut connection = model::db_connection();

    loop {
        process_alacritty_master(&mut connection, &repository);
        thread::sleep(NOTIFICATION_FREQUENCY);
    }
}

/// Check the alacritty master for new updates.
fn process_alacritty_master(connection: &mut SqliteConnection, repository: &Repository) {
    let last_build = MasterBuild::latest(connection);
    let commits = repository.commits();

    // Skip if this commit has already been benchmarked.
    if commits.is_empty() || last_build.map_or(false, |build| commits[0].sha == build.hash) {
        debug!("Found no new commits on master");
        return;
    }
    debug!("Found {} new commits on master", commits.len());

    // Stage job.
    let repository = repository.full_name.clone();
    NewJob::new(repository, None, None).insert(connection);

    // Update latest master build.
    NewMasterBuild::new(commits[0].sha.clone()).insert(connection);
}

/// Process a single GitHub notification.
///
/// This will check if a benchmark request is valid and then insert it into the
/// database.
fn process_notification(connection: &mut SqliteConnection, notification: Notification) {
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

/// Log an error, ignoring success.
///
/// This is a macro to preserve log message line numbers.
#[macro_export]
macro_rules! trace_error {
    ($result:expr) => {{
        if let Err(err) = &$result {
            tracing::error!("{err}");
        }
    }};
}
