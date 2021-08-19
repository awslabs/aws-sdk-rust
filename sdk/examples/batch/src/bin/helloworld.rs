use batch::Region;

#[tokio::main]
async fn main() -> Result<(), batch::Error> {
    tracing_subscriber::fmt::init();

    let conf = batch::Config::builder()
        .region(Region::new("us-east-2"))
        .build();
    let client = batch::Client::from_conf(conf);
    let rsp = client.describe_compute_environments().send().await?;

    let compute_envs = rsp.compute_environments.unwrap_or_default();
    println!("Compute environments ({}):", compute_envs.len());
    for env in compute_envs {
        let arn = env.compute_environment_arn.as_deref().unwrap_or_default();
        let name = env.compute_environment_name.as_deref().unwrap_or_default();

        println!(
            "  Compute Environment Name : {}, Compute Environment ARN : {}",
            name, arn
        );
    }

    Ok(())
}
