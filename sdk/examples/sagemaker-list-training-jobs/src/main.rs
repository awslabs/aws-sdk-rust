#[tokio::main]
async fn main() -> Result<(), sagemaker::Error> {
    let client = sagemaker::Client::from_env();
    let job_details = client.list_training_jobs().send().await?;

    println!("Job Name\tCreation DateTime\tDuration\tStatus");
    for j in job_details.training_job_summaries.unwrap_or_default() {
        let name = j.training_job_name.as_deref().unwrap_or_default();
        let creation_time = j.creation_time.unwrap().to_chrono();
        let training_end_time = j.training_end_time.unwrap().to_chrono();

        let status = j.training_job_status.unwrap();
        let duration = training_end_time - creation_time;

        let deets = format!(
            "{}\t{}\t{}\t{:#?}",
            name,
            creation_time.format("%Y-%m-%d@%H:%M:%S"),
            duration.num_seconds(),
            status
        );
        println!("{}", deets);
    }

    Ok(())
}
