#[tokio::main]
async fn main() -> Result<(), route53::Error> {
    env_logger::init();

    let client = route53::Client::from_env();
    let hosted_zone_count = client.get_hosted_zone_count().send().await?;

    println!(
        "Number of hosted zones in the account : {}",
        hosted_zone_count.hosted_zone_count.unwrap_or_default(),
    );

    let hosted_zones = client.list_hosted_zones().send().await?;

    for hz in hosted_zones.hosted_zones.unwrap_or_default() {
        let zone_name = hz.name.as_deref().unwrap_or_default();
        let zone_id = hz.id.as_deref().unwrap_or_default();

        println!("Zone ID : {}, Zone Name : {}", zone_id, zone_name);
    }

    println!("Hello, world!");

    Ok(())
}
