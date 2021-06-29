use aws_types::region::ProvideRegion;
use batch::{Client, Config, Error, Region};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    /// The default AWS Region.
    #[structopt(short, long)]
    default_region: Option<String>,

    /// Whether to display additional information.
    #[structopt(short, long)]
    verbose: bool,
}

/// Lists the names and the ARNs of your batch compute environments in a Region.
/// # Arguments
///
/// * `[-d DEFAULT-REGION]` - The Region in which the client is created.
///   If not supplied, uses the value of the **AWS_REGION** environment variable.
///   If the environment variable is not set, defaults to **us-west-2**.
/// * `[-v]` - Whether to display information.
#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();
    let Opt {
        default_region,
        verbose,
    } = Opt::from_args();

    let region = default_region
        .as_ref()
        .map(|region| Region::new(region.clone()))
        .or_else(|| aws_types::region::default_provider().region())
        .unwrap_or_else(|| Region::new("us-west-2"));

    println!();

    if verbose {
        println!("Batch client version: {}", batch::PKG_VERSION);
        println!("Region:               {:?}", &region);
        println!();
    }

    let conf = Config::builder().region(&region).build();
    let client = Client::from_conf(conf);
    let rsp = client.describe_compute_environments().send().await?;

    let compute_envs = rsp.compute_environments.unwrap_or_default();
    println!("Found {} compute environments:", compute_envs.len());
    for env in compute_envs {
        let arn = env.compute_environment_arn.as_deref().unwrap_or_default();
        let name = env.compute_environment_name.as_deref().unwrap_or_default();

        println!("  Name : {}", name);
        println!("  ARN:   {}", arn);
        println!();
    }

    Ok(())
}
