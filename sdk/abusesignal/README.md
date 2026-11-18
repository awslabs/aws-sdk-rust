# aws-sdk-abusesignal

Amazon Web Services Abuse Signal API provides programmatic means to report suspected abuse of Amazon Web Services resources. You can use the API to submit abuse reports about Amazon Web Services resources and to track the status of your reports. Amazon Web Services Trust & Safety uses the information that you provide to investigate and try to resolve the incident that you reported.

You submit reports in the eXtended Abuse Reporting Format (XARF), an open standard for structured abuse reporting. For more information, see [XARF on the XARF website](https://xarf.org/docs/introduction/). Reports can cover a wide range of abuse categories, including spam, phishing, distributed denial of service (DDoS), malware distribution, and copyright infringement.

With the Amazon Web Services Abuse Signal API, you can:
  - __Submit abuse reports__ – Create structured abuse reports by using the XARF format, with full support for all XARF v4 report types.
  - __Track report status__ – Monitor the processing status of submitted reports, from submission through resolution.
  - __List and filter reports__ – Retrieve the status of all reports, with optional filtering by correlation ID.
  - __Retry submissions safely__ – Safely retry report submissions without creating duplicates by using client tokens.

__Cross-Region data residency:__ By submitting a report, you instruct Amazon Web Services to store it in all supported Regions (us-east-1, us-west-2, and eu-west-2), including across the United States and Europe. If data must remain in a single Region, do not include it in the report.

## Getting Started

> Examples are available for many services and operations, check out the
> [usage examples](https://github.com/awsdocs/aws-doc-sdk-examples/tree/main/rustv1).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-abusesignal` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-abusesignal = "1.0.0"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_abusesignal as abusesignal;

#[::tokio::main]
async fn main() -> Result<(), abusesignal::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_abusesignal::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-abusesignal/latest/aws_sdk_abusesignal/client/struct.Client.html)
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

