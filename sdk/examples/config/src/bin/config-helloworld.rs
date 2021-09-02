use aws_config::meta::region::RegionProviderChain;

use config::model::ResourceType;
use config::{Client, Error, Region};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    /// The default AWS Region.
    #[structopt(short, long)]
    region: Option<String>,

    /// The resource id.
    #[structopt(long)]
    resource_id: String,

    /// The resource type, eg. "AWS::EC2::SecurityGroup"
    #[structopt(long)]
    resource_type: String,

    /// Whether to display additional information.
    #[structopt(short, long)]
    verbose: bool,
}

/// Lists the configuration history for a resource
///
/// NOTE: AWS Config must be enabled to discover resources
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
        region,
        resource_id,
        resource_type,
        verbose,
    } = Opt::from_args();

    let region_provider = RegionProviderChain::first_try(region.map(Region::new))
        .or_default_provider()
        .or_else(Region::new("us-west-2"));
    let shared_config = aws_config::from_env().region(region_provider).load().await;
    let client = Client::new(&shared_config);

    println!();

    if verbose {
        println!("Config client version: {}", config::PKG_VERSION);
        println!(
            "Region:               {:?}",
            shared_config.region().unwrap()
        );
        println!();
    }

    // parse resource type from user input
    let parsed = ResourceType::from(resource_type.as_str());
    if matches!(parsed, ResourceType::Unknown(_)) {
        panic!(
            "unknown resource type: `{}`. Valid resource types: {:#?}",
            &resource_type,
            ResourceType::values()
        )
    }
    let rsp = client
        .get_resource_config_history()
        .resource_id(&resource_id)
        .resource_type(parsed)
        .send()
        .await?;
    println!("configuration history for {}:", resource_id);
    for item in rsp.configuration_items.unwrap_or_default() {
        println!("item: {:?}", item);
    }

    Ok(())
}
