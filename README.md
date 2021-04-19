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

1. Create a new Rust project 
2. Within your Cargo.toml file, add dependencies for the AWS service(s) you a planning to use, Tokio, and Hyper
3. Input your AWS credentials into your terminal as environment variables

### Prerequisites

In order to use the SDK for Rust, you must already have Rust and Cargo installed. If you don't, these instructions will show you how to install Rust and Cargo: https://doc.rust-lang.org/book/ch01-01-installation.html

## Getting Help

* Public slack channel
* Github discussions

## Feedback and Contributing

### Feedback 

The alpha SDK will use GitHub Issues to track feature requests and issues with the SDK. In addition, we'll use GitHub Projects to provide users with a high level view of our roadmap and the features we're actively tracking. 

You can provide feedback or report a bug  by submitting a **GitHub issue**. This is the preferred mechanism to give feedback so that other users can engage in the conversation, +1 issues, etc. Issues you open will be evaluated for our roadmap in the Developer Preview launch.

### Contributing

If you are interested in contributing to the new AWS SDK for Rust, please take a look at [CONTRIBUTING](CONTRIBUTING.md)

## AWS Services Supported

This alpha SDK currently does not provide support for every AWS service. You can see all the services currently supported on [AWS_SERVICES_SUPPORTED](AWS_SERVICES_SUPPORTED.md)

## Additional Resources

- Design docs - If you're interested in understanding the design of the SDK we encourage you to take a look at the design documentation on the [Smithy-Rust code gen machinery repo](https://github.com/awslabs/smithy-rs). You can directly access them through the links [here](https://github.com/awslabs/smithy-rs/tree/main/rust-runtime) and [here](https://github.com/awslabs/smithy-rs/tree/main/aws/rust-runtime)
- [Code Examples](https://github.com/awslabs/aws-sdk-rust/tree/main/sdk/examples)
- Our alpha launch announcement

## Security

See [CONTRIBUTING](CONTRIBUTING.md#security-issue-notifications) for more information.

## License

This project is licensed under the Apache-2.0 License.

