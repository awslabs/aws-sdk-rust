# aws-sdk-lambdaweb

AWS Lambda Web Functions is a capability of AWS Lambda that you can use to run web applications and API backends as serverless HTTP servers. Your code listens on a network port and handles requests directly. You can use familiar Node.js web frameworks such as Express, Hono, or React Router with no Lambda-specific adapters or wrappers. You develop and run your server locally like any other Node.js web app.

Lambda creates an HTTPS endpoint, manages the execution environments, and scales your application with traffic, including to zero when it's idle. To reduce latency for globally distributed users, you can deploy across multiple AWS Regions behind a single endpoint.

A Web Function has one or more immutable revisions (code and configuration) and one or more endpoints that expose it over HTTPS.

Use Web Functions for workloads that receive and respond to HTTP traffic directly. Common use cases include:
  - Web applications – Deploy full-stack or front-end applications that serve HTML, assets, and API routes from a single deployment.
  - REST and GraphQL APIs – Build API backends that serve high request volumes.
  - Microservices – Run HTTP services that scale independently with traffic, including to zero when idle.
  - Server-side rendering (SSR) – Serve pages that are rendered on the server at request time rather than pre-built. A Web Function runs your framework's production server directly. This suits applications that render personalized or frequently changing content on each request.
  - Streaming responses – Stream output to the client as it is produced, such as responses from a large language model.

For event-driven processing, such as handling queue messages or file uploads, use [Lambda Event Functions](https://docs.aws.amazon.com/lambda/latest/dg/lambda-functions-chapter.html) instead. For more information about Web Functions, see [Lambda Web Functions](https://docs.aws.amazon.com/lambda/latest/dg/web-functions.html) in the _AWS Lambda Developer Guide_.

## Getting Started

> Examples are available for many services and operations, check out the
> [usage examples](https://github.com/awsdocs/aws-doc-sdk-examples/tree/main/rustv1).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-lambdaweb` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-lambdaweb = "1.3.0"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_lambdaweb as lambdaweb;

#[::tokio::main]
async fn main() -> Result<(), lambdaweb::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_lambdaweb::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-lambdaweb/latest/aws_sdk_lambdaweb/client/struct.Client.html)
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

