# The new AWS SDK for Rust 

Jump to:
- [Getting Started](#Getting-Started-with-the-SDK)
- [Getting Help](#Getting-Help)
- [Feedback and contributing](#Feedback-and-Contributing)
- [AWS Services Supported](#AWS-Services-Supported)
- [Additional Resources](#Additional-Resources)

## About

This repo contains the new AWS SDK for Rust and its [public roadmap](https://github.com/awslabs/aws-sdk-rust/projects/1)

**Please Note: The SDK is currently released as an alpha and is intended strictly for feedback purposes only. Do not use this SDK for production workloads.**

This SDK for Rust is code generated from [Smithy models](https://awslabs.github.io/smithy/) that represent each underlying AWS service. To view the code gen machinery, navigate to [Smithy-Rust code gen machinery repo](https://github.com/awslabs/smithy-rs).

## Getting Started with the SDK

The new AWS SDK for Rust is built with modularized packages for each AWS service, which means each AWS service you use will need to be added as a separate cargo dependency within your Rust project. Additionally, the alpha SDK requires you to add [Hyper](https://crates.io/crates/hyper) and [Tokio](https://crates.io/crates/tokio)as dependencies within your Rust project to dispatch your HTTP requests and perform I/O.

The following instructions will provide you with a quick example of how to get started with the new AWS SDK for Rust and use DynamoDB to perform a simple operation.

1. Create a new Rust project 
2. Within your Cargo.toml file, add dependencies for DynamoDB, Tokio, and Hyper

```
[dependencies]
aws-dynamodb = 0.1-alpha // add a dependency for each AWS service you are planning to use    
aws-hyper = 0.1-alpha
tokio = "1" # or a subset if you know what you want
```
3. Input your AWS credentials into your terminal as environment variables **Note:** The alpha SDK only supports environment variable credential providers at this time. 

```
export AWS_ACCESS_KEY_ID=...
export AWS_SECRET_ACCESS_KEY=...
```

4. Make a request using DynamoDB

```
use std::error::Error;

use dynamodb::{operation, Region};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config = dynamodb::Config::builder()
        .region(Region::new("us-east-1"))
        .build();
    let client = aws_hyper::Client::https();

    let op = operation::ListTables::builder().limit(10).build(&config);
    let tables = client.call(op).await?;
    println!("Current DynamoDB tables: {:?}", tables);
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

- Design docs - Design documentation for the SDK live on the [Smithy-Rust code gen machinery repo](https://github.com/awslabs/smithy-rs). You can directly access them through the links [here](https://github.com/awslabs/smithy-rs/tree/main/rust-runtime) and [here](https://github.com/awslabs/smithy-rs/tree/main/aws/rust-runtime)
- [Code Examples](https://github.com/awslabs/aws-sdk-rust/tree/main/sdk/examples)
- API reference docs - You can generate API reference docs by calling `cargo doc --open` on your  project and navigating to the SDK crate
- Our alpha launch announcement

## Security

See [CONTRIBUTING](CONTRIBUTING.md#security-issue-notifications) for more information.

## License

This project is licensed under the Apache-2.0 License.

