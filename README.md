# The new AWS SDK for Rust 

This repo contains the new AWS SDK for Rust and its [public roadmap](https://github.com/awslabs/aws-sdk-rust/projects/1)

**Please Note: The SDK is currently released as an alpha and is intended strictly for feedback purposes only. Do not use this SDK for production workloads.**

This SDK for Rust is code generated from [Smithy models](https://awslabs.github.io/smithy/) that represent each AWS service. Code used to generate the SDK can be found in [smithy-rs](https://github.com/awslabs/smithy-rs).

## Getting Started with the SDK

The new AWS SDK for Rust is built with modular crates for each AWS service, which means each AWS service you use will need to be added as a separate cargo dependency within your Rust project. Additionally, [Tokio](https://crates.io/crates/tokio) must be added as a dependency within your Rust project to execute async code. During the alpha, the SDK will not be pushed to crates.io and must be used via a Git dependency:

The following instructions will provide you with a quick example of how to get started with the new AWS SDK for Rust and use DynamoDB to perform a simple operation.

1. Create a new Rust project 
2. Within your Cargo.toml file, add dependencies for DynamoDB and Tokio:

```toml
[dependencies]
dynamodb = { git = "https://awslabs/aws-sdk-rust", tag = "v0.0.1-alpha", package = "aws-sdk-dynamodb" }
tokio = { version = "1", features = ["full"] }
```
3. Input your AWS credentials into your terminal as environment variables **Note:** The alpha SDK only supports environment variable credential providers at this time. 

```bash
export AWS_ACCESS_KEY_ID=...
export AWS_SECRET_ACCESS_KEY=...
export AWS_DEFAULT_REGION=... # eg. us-east-1
```

4. Make a request using DynamoDB

```rust

#[tokio::main]
async fn main() -> Result<(), dynamodb::Error> {
    let client = dynamodb::Client::from_env();
    let req = client.list_tables().limit(10);
    let resp = req.send().await?;
    println!("Current DynamoDB tables: {:?}", tables.table_names);
    Ok(())
}
```

### Prerequisites

In order to use the SDK for Rust, you must already have Rust and Cargo installed. If you don't, these instructions will show you how to install Rust and Cargo: https://doc.rust-lang.org/book/ch01-01-installation.html

## Getting Help

* [GitHub discussions](https://github.com/awslabs/aws-sdk-rust/discussions) - Submit your questions on the discussion board
* *Public slack channel/Gitter/Discord?*

## Feedback and Contributing

### Feedback 

The alpha SDK uses **GitHub Issues** to track feature requests and issues with the SDK. In addition, we use **GitHub Projects** to provide users with a high level view of our roadmap and the features we're actively working on. 

You can provide feedback or report a bug  by submitting a **GitHub issue**. This is the preferred mechanism to give feedback so that other users can engage in the conversation, +1 issues, etc. Issues you open will be evaluated for our roadmap in the Developer Preview launch.

### Contributing

If you are interested in contributing to the new AWS SDK for Rust, please take a look at [CONTRIBUTING](CONTRIBUTING.md)

## AWS Services Supported

This alpha SDK currently does not provide support for every AWS service. You can see all the services currently supported on [AWS_SERVICES_SUPPORTED](AWS_SERVICES_SUPPORTED.md)

## Additional Resources

- Design docs - Design documentation for the SDK lives in the [design folder of smithy-rs](https://github.com/awslabs/smithy-rs/tree/main/design).
- Runtime / Handwritten code: The Rust Runtime code that underpins the SDK can be accessed [here](https://github.com/awslabs/smithy-rs/tree/main/rust-runtime) and [here](https://github.com/awslabs/smithy-rs/tree/main/aws/rust-runtime). This code is copied into this repo as part of code generation.
- [Code Examples](https://github.com/awslabs/aws-sdk-rust/tree/main/sdk/examples)
- API reference docs - You can generate API reference docs by calling `cargo doc --open` on your  project and navigating to the SDK crate

<!-- TODO
- Our alpha launch announcement
--> 
## Security

See [CONTRIBUTING](CONTRIBUTING.md#security-issue-notifications) for more information.

## License

This project is licensed under the Apache-2.0 License.
