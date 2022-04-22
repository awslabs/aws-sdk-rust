use aws_sdk_s3::types::ByteStream;
use aws_sdk_s3::{Client, Endpoint, Error}; // snippet-end:[s3.rust.client-use]
use std::path::Path;


/// Upload a file chunk to an S3 object
#[tokio::main]
async fn main() -> Result<(), aws_sdk_s3::Error> {
    let args = std::env::args().collect::<Vec<_>>();
    let usage = format!("{} <profile> <url> <bucket> <key> <input file> <start offset> <chunk size, 0 for whole file>", args[0]);
    let profile = args.get(1).expect(&usage);
    let url = args.get(2).expect(&usage);
    let bucket = args.get(3).expect(&usage);
    let key = args.get(4).expect(&usage);
    let file_name = args.get(5).expect(&usage);
    let start_offset = args.get(6).expect(&usage).parse::<u64>().expect("Error parsing offset");
    let chunk_size = args.get(7).expect(&usage).parse::<u64>().expect("Error parsing chunk size");
    let md = std::fs::metadata(file_name).map_err(|err| Error::Unhandled(Box::new(err)))?;
    let chunk_size = if chunk_size == 0 { md.len()} else {chunk_size};

    // credentials are read from .aws/credentials file
    let conf = aws_config::from_env()
        .region("us-east-1")
        .credentials_provider(
            aws_config::profile::ProfileFileCredentialsProvider::builder()
                .profile_name(profile)
                .build(),
        )
        .load()
        .await;
    let uri = url.parse::<http::uri::Uri>().expect("Invalid URL");
    let ep = Endpoint::immutable(uri);
    let s3_conf = aws_sdk_s3::config::Builder::from(&conf)
        .endpoint_resolver(ep)
        .build();
    let client = Client::from_conf(s3_conf);
    upload_chunk(&client, &bucket, &file_name, &key, start_offset, chunk_size).await?;
    Ok(())
}

/// Upload file chunk to bucket/key
pub async fn upload_chunk(
    client: &Client,
    bucket: &str,
    file_name: &str,
    key: &str,
    start_offset: u64,
    chunk_size: u64
) -> Result<(), Error> {
    let body = ByteStream::from_path_chunk(Path::new(file_name), start_offset, chunk_size)
        .await
        .expect(&format!("Cannot read from {}", file_name));
    client
        .put_object()
        .bucket(bucket)
        .key(key)
        .body(body)
        .send()
        .await?;
    println!(
        "Uploaded chunk of size {} from file {}",
        chunk_size,
        file_name,
    );
    Ok(())
}
