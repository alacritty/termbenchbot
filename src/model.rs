use std::env;

use chrono::{Duration, NaiveDateTime, Utc};
use diesel::prelude::*;
use serde::Serialize;
use tracing::debug;

use crate::schema::{job, master_build};
use crate::trace_error;

/// Maximum minutes before a benchmark is considered to be dead.
const MAX_BENCH_MINUTES: i64 = 120;

#[derive(Serialize, Queryable)]
pub struct Job {
    pub id: i32,
    pub repository: String,
    pub hash: Option<String>,
    #[serde(skip)]
    pub comments_url: Option<String>,
    #[serde(skip)]
    pub started_at: Option<NaiveDateTime>,
}

impl Job {
    /// Get all staged jobs.
    pub fn all(connection: &mut SqliteConnection) -> Vec<Self> {
        job::dsl::job.load(connection).unwrap_or_default()
    }

    /// Load a specific job using its ID.
    pub fn from_id(connection: &mut SqliteConnection, id: i32) -> Option<Self> {
        job::dsl::job.filter(job::dsl::id.eq(id)).first(connection).ok()
    }

    /// Remove a job.
    pub fn delete(self, connection: &mut SqliteConnection) {
        debug!("Deleting job {}", self.id);
        trace_error!(
            diesel::delete(job::dsl::job.filter(job::dsl::id.eq(self.id))).execute(connection)
        );
    }

    /// Mark job as pending for execution.
    pub fn mark_pending(&self, connection: &mut SqliteConnection) {
        debug!("Marking job {} as pending", self.id);
        trace_error!(diesel::update(job::dsl::job.filter(job::dsl::id.eq(self.id)))
            .set(job::dsl::started_at.eq::<Option<NaiveDateTime>>(None))
            .execute(connection));
    }

    /// Mark job as currently executing.
    pub fn mark_started(connection: &mut SqliteConnection, id: i32) {
        debug!("Marking job {} as started", id);
        trace_error!(diesel::update(job::dsl::job.filter(job::dsl::id.eq(id)))
            .set(job::dsl::started_at.eq(Utc::now().naive_utc()))
            .execute(connection));
    }

    /// Remove `started_at` from stale jobs.
    pub fn update_stale(connection: &mut SqliteConnection) {
        debug!("Clearing in-progress state for stale jobs");
        let limit = Utc::now().naive_utc() - Duration::minutes(MAX_BENCH_MINUTES);
        trace_error!(diesel::update(job::dsl::job.filter(job::dsl::started_at.lt(limit)))
            .set(job::dsl::started_at.eq::<Option<NaiveDateTime>>(None))
            .execute(connection));
    }
}

#[derive(Insertable, Debug)]
#[diesel(table_name = job)]
pub struct NewJob {
    pub repository: String,
    pub comments_url: Option<String>,
    pub hash: Option<String>,
}

impl NewJob {
    /// Create a new job for insertion.
    pub fn new(
        repository: String,
        comments_url: impl Into<Option<String>>,
        hash: impl Into<Option<String>>,
    ) -> Self {
        Self { repository, comments_url: comments_url.into(), hash: hash.into() }
    }

    /// Insert the job in the database.
    pub fn insert(&self, connection: &mut SqliteConnection) {
        debug!("Staging new job: {self:?}");
        trace_error!(diesel::insert_into(job::table).values(self).execute(connection));
    }
}

#[derive(Serialize, Queryable, Debug)]
pub struct MasterBuild {
    pub id: i32,
    pub hash: String,
}

impl MasterBuild {
    /// Get the last master build entry.
    pub fn latest(connection: &mut SqliteConnection) -> Option<Self> {
        master_build::dsl::master_build
            .order_by(master_build::dsl::id.desc())
            .first(connection)
            .ok()
    }
}

#[derive(Insertable)]
#[diesel(table_name = master_build)]
pub struct NewMasterBuild {
    pub hash: String,
}

impl NewMasterBuild {
    /// Create a new master build entry.
    pub fn new(hash: String) -> Self {
        Self { hash }
    }

    /// Insert master build into the database.
    pub fn insert(&self, connection: &mut SqliteConnection) {
        debug!("Setting latest master build to {:?}", self.hash);
        trace_error!(diesel::insert_into(master_build::table).values(self).execute(connection));
    }
}

/// Connect to the database.
pub fn db_connection() -> SqliteConnection {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL environment variable missing");
    SqliteConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Unable to find DB: {}", database_url))
}
