# The new AWS SDK for Rust 

Jump to:
- [Getting Started](#Getting-Started-with-the-SDK)
- [Getting Help](#Getting-Help)
- [Feedback and contributing](#Feedback-and-contributing)
- [Design](#Design-Docs)
- [AWS Services Supported](#AWS-Services-Supported)
- [Additional Resources](#Additional-Resources)

## About

This repo contains the new AWS SDK for Rust and its public roadmap

**Please Note: The SDK is currently released as an alpha and is intended strictly for feedback purposes only. Do not use this SDK for production workloads.**

This SDK for Rust is code generated from [Smithy models](https://awslabs.github.io/smithy/) that represent each underlying AWS service. To view the code gen machinery, navigate to [Smithy-Rust code generator repo](https://github.com/awslabs/smithy-rs).

## Getting Started with the SDK

1. Create a new Rust project 
2. Within your Cargo.toml file, add dependencies for the AWS service(s) you a planning to use, Tokio, and Hyper
3. Input your AWS credentials into your terminal as environment variables

### Prerequisites

In order to use the SDK for Rust, you must already have Rust and Cargo installed. If you don't, these instructions will show you how to install Rust and Cargo: https://doc.rust-lang.org/book/ch01-01-installation.html

## Getting Help

* Public slack channel
* Github discussions

## Providing Feedback 

We are using GitHub Issues to track 

## Contributing

If you are interested in contributing to the new AWS SDK for Rust, please take a look at [CONTRIBUTING](CONTRIBUTING.MD)

## Design Docs

If you're interested, we encourage you to take a look at the design documentation on the [Smithy-Rust code generator repo](https://github.com/awslabs/smithy-rs). You can find docs [here](https://github.com/awslabs/smithy-rs/tree/main/rust-runtime) and [here](https://github.com/awslabs/smithy-rs/tree/main/aws/rust-runtime)

## AWS Services Supported

This alpha SDK currently does not provide support for every AWS service. You can see all the services currently by the SDK on [AWS_SERVICES_SUPPORTED](AWS_SERVICES_SUPPORTED.md)

## Additional Resources

## Security

See [CONTRIBUTING](CONTRIBUTING.md#security-issue-notifications) for more information.

## License

This project is licensed under the Apache-2.0 License.

