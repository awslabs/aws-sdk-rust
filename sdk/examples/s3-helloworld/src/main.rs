use s3::ByteStream;
use s3::Region;
use std::error::Error;
use std::path::Path;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::fmt::SubscriberBuilder;

// Change these to your bucket & key
const BUCKET: &str = "demo-bucket";
const KEY: &str = "demo-object";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    SubscriberBuilder::default()
        .with_env_filter("info")
        .with_span_events(FmtSpan::CLOSE)
        .init();
    let conf = s3::Config::builder()
        .region(Region::new("us-east-2"))
        .build();
    let client = s3::Client::from_conf(conf);
    let resp = client.list_buckets().send().await?;
    for bucket in resp.buckets.unwrap_or_default() {
        println!("bucket: {:?}", bucket.name.expect("buckets have names"))
    }
    let body = ByteStream::from_path(Path::new("Cargo.toml")).await?;
    let resp = client
        .put_object()
        .bucket(BUCKET)
        .key(KEY)
        .body(body)
        .send();
    let resp = resp.await?;
    println!("Upload success. Version: {:?}", resp.version_id);

    let resp = client.get_object().bucket(BUCKET).key(KEY).send().await?;
    let data = resp.body.collect().await?;
    println!("data: {:?}", data.into_bytes());
    Ok(())
}
