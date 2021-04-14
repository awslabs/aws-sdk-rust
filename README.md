# The new AWS SDK for Rust 

## About

This repo contains the new AWS SDK for Rust and its public roadmap

**Please Note: The SDK is currently released as an alpha and is intended strictly for feedback purposes only. Do not use this SDK for production workloads.**

This SDK for Rust is 100% code generated from Smithy models that represent each underlying AWS service. To view the code gen machinery, navigate to Smithy-RS repo: https://github.com/awslabs/smithy-rs. 

Because all of the code on this repo has been code generated, **please do not submit code changing PRs to this repo**. If you would like to make a code contribution to the SDK, please submit a PR to the code gen machinery on the Smithy-RS repo.

## Getting Started with the SDK

1. Create a new Rust project 
2. Within your Cargo.toml file, add dependencies for the AWS service(s) you a planning to use, Tokio, and Hyper
3. Input your AWS credentials into your terminal as environment variables

### Prerequisites

In order to use the SDK for Rust, you must already have Rust and Cargo installed. If you don't, these instructions will show you how to install Rust and Cargo: https://doc.rust-lang.org/book/ch01-01-installation.html

## Security

See [CONTRIBUTING](CONTRIBUTING.md#security-issue-notifications) for more information.

## License

This project is licensed under the Apache-2.0 License.

