#[tokio::main]
async fn main() -> Result<(), cloudformation::Error> {
    let client = cloudformation::Client::from_env();
    let stacks = client.list_stacks().send().await?;

    for s in stacks.stack_summaries.unwrap_or_default() {
        let details = format!(
            "{}\t{}\t{:#?}\t{}",
            s.stack_id.as_deref().unwrap_or_default(),
            s.stack_name.as_deref().unwrap_or_default(),
            s.stack_status.unwrap(),
            s.stack_status_reason.as_deref().unwrap_or_default()
        );

        println!("{}", details);
    }

    Ok(())
}
