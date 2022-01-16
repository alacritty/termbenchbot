use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{get, patch, post, routes, Error};
use serde::Deserialize;

use crate::github::Comment;
use crate::model::{self, Job};

/// Benchmark results request data.
#[derive(Deserialize)]
struct BenchmarkResults<'a> {
    result: &'a str,
}

/// List all pending jobs.
#[get("/jobs")]
fn jobs() -> Json<Vec<Job>> {
    let mut jobs = Job::all(&model::db_connection());
    jobs.retain(|job| job.started_at.is_none());
    Json(jobs)
}

/// Mark a job as currently running.
#[patch("/jobs/<id>")]
fn mark_started(id: i32) {
    Job::mark_started(&model::db_connection(), id);
}

/// Submit benchmark results.
#[post("/jobs/<id>", data = "<data>")]
fn submit_results(id: i32, data: Json<BenchmarkResults>) -> (Status, &str) {
    let connection = model::db_connection();

    let job = match Job::from_id(&connection, id) {
        Some(job) if job.started_at.is_some() => job,
        Some(_) => return (Status::BadRequest, "job must be marked as running first"),
        None => return (Status::NotFound, ""),
    };

    match Comment::new(&job.comments_url, data.result) {
        Ok(_) => job.delete(&connection),
        Err(_) => job.mark_pending(&connection),
    }

    (Status::Ok, "")
}

/// Start the webserver.
pub async fn launch() -> Result<(), Error> {
    rocket::build().mount("/", routes![jobs, mark_started, submit_results]).launch().await
}
