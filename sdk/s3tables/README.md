# aws-sdk-s3tables

An Amazon S3 table represents a structured dataset consisting of tabular data in [Apache Parquet](https://parquet.apache.org/docs/) format and related metadata. This data is stored inside an S3 table as a subresource. All tables in a table bucket are stored in the [Apache Iceberg](https://iceberg.apache.org/docs/latest/) table format. Through integration with the [Amazon Web Services Glue Data Catalog](https://docs.aws.amazon.com/https:/docs.aws.amazon.com/glue/latest/dg/catalog-and-crawler.html) you can interact with your tables using Amazon Web Services analytics services, such as [Amazon Athena](https://docs.aws.amazon.com/https:/docs.aws.amazon.com/athena/) and [Amazon Redshift](https://docs.aws.amazon.com/https:/docs.aws.amazon.com/redshift/). Amazon S3 manages maintenance of your tables through automatic file compaction and snapshot management. For more information, see [Amazon S3 table buckets](https://docs.aws.amazon.com/AmazonS3/latest/userguide/s3-tables-buckets.html).

## Getting Started

> Examples are available for many services and operations, check out the
> [usage examples](https://github.com/awsdocs/aws-doc-sdk-examples/tree/main/rustv1).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-s3tables` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-s3tables = "1.39.0"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_s3tables as s3tables;

#[::tokio::main]
async fn main() -> Result<(), s3tables::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_s3tables::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-s3tables/latest/aws_sdk_s3tables/client/struct.Client.html)
for information on what calls can be made, and the inputs and outputs for each of those calls.

## Using the SDK

Until the SDK is released, we will be adding information about using the SDK to the
[Developer Guide](https://docs.aws.amazon.com/sdk-for-rust/latest/dg/welcome.html). Feel free to suggest
additional sections for the guide by opening an issue and describing what you are trying to do.

## Getting Help

* [GitHub discussions](https://github.com/awslabs/aws-sdk-rust/discussions) - For ideas, RFCs & general questions
* [GitHub issues](https://github.com/awslabs/aws-sdk-rust/issues/new/choose) - For bug reports & feature requests
* [Generated Docs (latest version)](https://awslabs.github.io/aws-sdk-rust/)
* [Usage examples](https://github.com/awsdocs/aws-doc-sdk-examples/tree/main/rustv1)

## License

This project is licensed under the Apache-2.0 License.

