use std::env;

use diesel::prelude::*;
use chrono::{NaiveDateTime, Utc, Duration};
use serde::Serialize;

use crate::schema::jobs::{self, dsl};

/// Maximum minutes before a benchmark is considered to be dead.
const MAX_BENCH_MINUTES: i64 = 120;

#[derive(Serialize, Queryable)]
pub struct Job {
    pub id: i32,
    pub repository: String,
    pub hash: String,
    #[serde(skip)]
    pub comments_url: String,
    #[serde(skip)]
    pub started_at: Option<NaiveDateTime>,
}

impl Job {
    /// Get all staged jobs.
    pub fn all(connection: &SqliteConnection) -> Vec<Self> {
        dsl::jobs.load::<Job>(connection).unwrap_or_default()
    }

    /// Load a specific job using its ID.
    pub fn from_id(connection: &SqliteConnection, id: i32) -> Option<Self> {
        dsl::jobs.filter(dsl::id.eq(id)).first::<Job>(connection).ok()
    }

    /// Remove a job.
    pub fn delete(self, connection: &SqliteConnection) {
        let _ = diesel::delete(dsl::jobs.filter(dsl::id.eq(self.id)))
            .execute(connection);
    }

    /// Mark job as pending for execution.
    pub fn mark_pending(&self, connection: &SqliteConnection) {
        let _ = diesel::update(dsl::jobs.filter(dsl::id.eq(self.id)))
            .set(dsl::started_at.eq::<Option<NaiveDateTime>>(None))
            .execute(connection);
    }

    /// Mark job as currently executing.
    pub fn mark_started(connection: &SqliteConnection, id: i32) {
        let _ = diesel::update(dsl::jobs.filter(dsl::id.eq(id)))
            .set(dsl::started_at.eq(Utc::now().naive_utc()))
            .execute(connection);
    }

    /// Remove `started_at` from stale jobs.
    pub fn update_stale(connection: &SqliteConnection) {
        let limit = Utc::now().naive_utc() - Duration::minutes(MAX_BENCH_MINUTES);
        let _ = diesel::update(dsl::jobs.filter(dsl::started_at.lt(limit)))
            .set(dsl::started_at.eq::<Option<NaiveDateTime>>(None))
            .execute(connection);
    }
}

#[derive(Insertable)]
#[table_name = "jobs"]
pub struct NewJob {
    pub comments_url: String,
    pub repository: String,
    pub hash: String,
}

impl NewJob {
    /// Create a new job for insertion.
    pub fn new(comments_url: String, repository: String, hash: String) -> Self {
        Self { comments_url, repository, hash }
    }

    /// Insert the job in the database.
    pub fn insert(&self, connection: &SqliteConnection) {
        let _ = diesel::insert_into(jobs::table).values(self).execute(connection);
    }
}

/// Connect to the database.
pub fn db_connection() -> SqliteConnection {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL environment variable missing");
    SqliteConnection::establish(&database_url)
        .expect(&format!("Unable to find DB: {}", database_url))
}
