use aws_sdk_s3::types::ByteStream;
use aws_sdk_s3::{Client, Endpoint, Error};
use tokio::io::AsyncSeekExt;
use aws_sdk_s3::model::CompletedMultipartUpload;
use aws_sdk_s3::model::CompletedPart;

/// Upload file reading files in chunks.
///
/// ## Usage
/// ```
/// upload-file-multipart <profile> <url> <bucket> <key> <input file> <number of parts>
/// ```
///
#[tokio::main]
async fn main() -> Result<(), aws_sdk_s3::Error> {
    const REGION: &str = "us-east-1";
    let args = std::env::args().collect::<Vec<_>>();
    let usage = format!("{} <profile> <url> <bucket> <key> <input file> <number of parts>", args[0]);
    let profile = args.get(1).expect(&usage);
    let url = args.get(2).expect(&usage);
    let bucket = args.get(3).expect(&usage);
    let key = args.get(4).expect(&usage);
    let file_name = args.get(5).expect(&usage);
    let num_parts = args.get(6).expect(&usage).parse::<usize>().expect("Error parsing num parts");
    // credentials are read from .aws/credentials file
    let conf = aws_config::from_env()
        .region(REGION)
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
    upload_multipart(&client, &bucket, &file_name, &key, num_parts).await?;
    Ok(())
}

/// Multipart upload
pub async fn upload_multipart(
    client: &Client,
    bucket: &str,
    file_name: &str,
    key: &str,
    num_parts: usize
) -> Result<(), Error> {
    let len: u64 = std::fs::metadata(file_name).map_err(|err| Error::Unhandled(Box::new(err)))?.len();
    let num_parts = num_parts as u64;
    let file = tokio::fs::File::open(file_name).await.map_err(|err| Error::Unhandled(Box::new(err)))?;
    let chunk_size = len / num_parts;
    let last_chunk_size = chunk_size + len % num_parts;
    // Initiate multipart upload and store upload id.
    let u = client
        .create_multipart_upload()
        .bucket(bucket)
        .key(key)
        .send()
        .await?;
    let uid = u.upload_id().ok_or(
        Error::NoSuchUpload(aws_sdk_s3::error::NoSuchUpload::builder().message("No upload ID").build()))?;
    // Iterate over file chunks, changing the file pointer at each iteration
    // and storing part id and associated etag into vector.
    let mut completed_parts: Vec<CompletedPart> = Vec::new();
    for i in 0..num_parts {
       let mut file = file.try_clone().await.unwrap();
       let size = if i != (num_parts - 1) { chunk_size } else { last_chunk_size };
       file.seek(std::io::SeekFrom::Start((i * len / num_parts) as u64)).await.map_err(|err| Error::Unhandled(Box::new(err)))?;
       let body = ByteStream::from_file_chunk(file, size).await.map_err(|err| Error::Unhandled(Box::new(err)))?;
       let up = client
            .upload_part()
            .bucket(bucket)
            .key(key)
            .content_length(size as i64)
            .upload_id(uid.clone())
            .part_number((i + 1) as i32)
            .body(body)
            .send()
            .await?;
        let cp = CompletedPart::builder().set_e_tag(up.e_tag).part_number((i+1) as i32).build();
        completed_parts.push(cp);
        
    }
    // Complete multipart upload, sending the (etag, part id) list along the request.
    let b = CompletedMultipartUpload::builder().set_parts(Some(completed_parts)).build();
    let completed = client.complete_multipart_upload().multipart_upload(b).
                    upload_id(uid.clone()).bucket(bucket).key(key).send().await?;
    // Print etag removing quotes.
    if let Some(etag) = completed.e_tag {
        println!("{}", etag.replace("\"",""));
    } else {
        eprintln!("Error receiving etag");
    }
    Ok(())
}
