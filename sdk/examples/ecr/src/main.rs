use aws_config::meta::region::RegionProviderChain;
use aws_sdk_ecr::Region;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    /// The region
    #[structopt(short, long)]
    region: Option<String>,

    #[structopt(long)]
    repository: String,

    #[structopt(short, long)]
    verbose: bool,
}
#[tokio::main]
async fn main() -> Result<(), aws_sdk_ecr::Error> {
    let Opt {
        region,
        repository,
        verbose,
    } = Opt::from_args();
    if verbose {
        tracing_subscriber::fmt::init();
    }
    let region_provider = RegionProviderChain::first_try(region.map(Region::new))
        .or_default_provider()
        .or_else(Region::new("us-west-2"));
    let shared_config = aws_config::from_env().region(region_provider).load().await;
    let client = aws_sdk_ecr::Client::new(&shared_config);
    let rsp = client
        .list_images()
        .repository_name(&repository)
        .send()
        .await?;
    let images = rsp.image_ids.unwrap_or_default();
    println!("found {} images", images.len());
    for image in images {
        println!(
            "image: {}:{}",
            image.image_tag.unwrap(),
            image.image_digest.unwrap()
        );
    }
    Ok(())
}
