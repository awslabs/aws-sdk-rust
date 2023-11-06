# aws-sdk-transcribestreaming

**Please Note: The SDK is currently in Developer Preview and is intended strictly for
feedback purposes only. Do not use this SDK for production workloads.**

Amazon Transcribe streaming offers three main types of real-time transcription: __Standard__, __Medical__, and __Call Analytics__.
  - __Standard transcriptions__ are the most common option. Refer to for details.
  - __Medical transcriptions__ are tailored to medical professionals and incorporate medical terms. A common use case for this service is transcribing doctor-patient dialogue in real time, so doctors can focus on their patient instead of taking notes. Refer to for details.
  - __Call Analytics transcriptions__ are designed for use with call center audio on two different channels; if you're looking for insight into customer service calls, use this option. Refer to for details.

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-transcribestreaming` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = "0.57.1"
aws-sdk-transcribestreaming = "0.35.0"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_transcribestreaming as transcribestreaming;

#[::tokio::main]
async fn main() -> Result<(), transcribestreaming::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_transcribestreaming::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-transcribestreaming/latest/aws_sdk_transcribestreaming/client/struct.Client.html)
for information on what calls can be made, and the inputs and outputs for each of those calls.

## Using the SDK

Until the SDK is released, we will be adding information about using the SDK to the
[Developer Guide](https://docs.aws.amazon.com/sdk-for-rust/latest/dg/welcome.html). Feel free to suggest
additional sections for the guide by opening an issue and describing what you are trying to do.

## Getting Help

* [GitHub discussions](https://github.com/awslabs/aws-sdk-rust/discussions) - For ideas, RFCs & general questions
* [GitHub issues](https://github.com/awslabs/aws-sdk-rust/issues/new/choose) - For bug reports & feature requests
* [Generated Docs (latest version)](https://awslabs.github.io/aws-sdk-rust/)
* [Usage examples](https://github.com/awslabs/aws-sdk-rust/tree/main/examples)

## License

This project is licensed under the Apache-2.0 License.

