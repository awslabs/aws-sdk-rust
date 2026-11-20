# aws-sdk-payments

Amazon Web Services Payments manages how money moves between you and Amazon Web Services. It holds the payment instruments you use to pay Amazon Web Services and to receive money from Amazon Web Services, such as cards and bank accounts. It collects payment for your Amazon Web Services invoices, applies your payments and credit memos to them, and returns refunds.

A payment instrument is a channel that moves money between you and Amazon Web Services, and its payment method is the kind of instrument, such as a card or a bank account. A financial record is an invoice or credit memo. It shows how much of its amount is outstanding, processing, settled, and canceled as Amazon Web Services applies payments and credits.

With the Amazon Web Services Payments API, you can read your financial records and account balances, and read your payment instruments and their status.

The Amazon Web Services Payments API provides the following endpoint:

https://payments.us-east-1.api.aws

## Getting Started

> Examples are available for many services and operations, check out the
> [usage examples](https://github.com/awsdocs/aws-doc-sdk-examples/tree/main/rustv1).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-payments` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-payments = "1.0.0"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_payments as payments;

#[::tokio::main]
async fn main() -> Result<(), payments::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_payments::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-payments/latest/aws_sdk_payments/client/struct.Client.html)
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

