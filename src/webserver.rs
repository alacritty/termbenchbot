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
    let mut jobs = Job::all(&mut model::db_connection());
    jobs.retain(|job| job.started_at.is_none());
    Json(jobs)
}

/// Mark a job as currently running.
#[patch("/jobs/<id>")]
fn mark_started(id: i32) {
    Job::mark_started(&mut model::db_connection(), id);
}

/// Submit benchmark results.
#[post("/jobs/<id>", data = "<data>")]
fn submit_results(id: i32, data: Option<Json<BenchmarkResults>>) -> (Status, String) {
    let mut connection = model::db_connection();

    let job = match Job::from_id(&mut connection, id) {
        Some(job) if job.started_at.is_some() => job,
        Some(_) => return (Status::BadRequest, "job must be marked as running first".into()),
        None => return (Status::NotFound, "".into()),
    };

    // Attempt to write results to the GitHub thread they were requested in.
    //
    // On failure, we stage the request for re-execution to attempt upload again at a later point
    // in time after potential network issues have been resolved.
    if let Some((comments_url, data)) = job.comments_url.as_ref().zip(data) {
        if let Err(err) = Comment::new(&comments_url, data.result) {
            job.mark_pending(&mut connection);
            return (Status::InternalServerError, err.to_string());
        }
    }

    job.delete(&mut connection);

    (Status::Ok, "".into())
}

/// Start the webserver.
pub async fn launch() -> Result<(), Error> {
    let _ =
        rocket::build().mount("/", routes![jobs, mark_started, submit_results]).launch().await?;
    Ok(())
}
