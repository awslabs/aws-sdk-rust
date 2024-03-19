# aws-sdk-lambda

__Overview__

Lambda is a compute service that lets you run code without provisioning or managing servers. Lambda runs your code on a high-availability compute infrastructure and performs all of the administration of the compute resources, including server and operating system maintenance, capacity provisioning and automatic scaling, code monitoring and logging. With Lambda, you can run code for virtually any type of application or backend service. For more information about the Lambda service, see [What is Lambda](https://docs.aws.amazon.com/lambda/latest/dg/welcome.html) in the __Lambda Developer Guide__.

The _Lambda API Reference_ provides information about each of the API methods, including details about the parameters in each API request and response.

You can use Software Development Kits (SDKs), Integrated Development Environment (IDE) Toolkits, and command line tools to access the API. For installation instructions, see [Tools for Amazon Web Services](http://aws.amazon.com/tools/).

For a list of Region-specific endpoints that Lambda supports, see [Lambda endpoints and quotas](https://docs.aws.amazon.com/general/latest/gr/lambda-service.html/) in the _Amazon Web Services General Reference._.

When making the API calls, you will need to authenticate your request by providing a signature. Lambda supports signature version 4. For more information, see [Signature Version 4 signing process](https://docs.aws.amazon.com/general/latest/gr/signature-version-4.html) in the _Amazon Web Services General Reference._.

__CA certificates__

Because Amazon Web Services SDKs use the CA certificates from your computer, changes to the certificates on the Amazon Web Services servers can cause connection failures when you attempt to use an SDK. You can prevent these failures by keeping your computer's CA certificates and operating system up-to-date. If you encounter this issue in a corporate environment and do not manage your own computer, you might need to ask an administrator to assist with the update process. The following list shows minimum operating system and Java versions:
  - Microsoft Windows versions that have updates from January 2005 or later installed contain at least one of the required CAs in their trust list.
  - Mac OS X 10.4 with Java for Mac OS X 10.4 Release 5 (February 2007), Mac OS X 10.5 (October 2007), and later versions contain at least one of the required CAs in their trust list.
  - Red Hat Enterprise Linux 5 (March 2007), 6, and 7 and CentOS 5, 6, and 7 all contain at least one of the required CAs in their default trusted CA list.
  - Java 1.4.2_12 (May 2006), 5 Update 2 (March 2005), and all later versions, including Java 6 (December 2006), 7, and 8, contain at least one of the required CAs in their default trusted CA list.

When accessing the Lambda management console or Lambda API endpoints, whether through browsers or programmatically, you will need to ensure your client machines support any of the following CAs:
  - Amazon Root CA 1
  - Starfield Services Root Certificate Authority - G2
  - Starfield Class 2 Certification Authority

Root certificates from the first two authorities are available from [Amazon trust services](https://www.amazontrust.com/repository/), but keeping your computer up-to-date is the more straightforward solution. To learn more about ACM-provided certificates, see [Amazon Web Services Certificate Manager FAQs.](http://aws.amazon.com/certificate-manager/faqs/#certificates)

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-lambda` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-lambda = "0.0.0-local"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_lambda as lambda;

#[::tokio::main]
async fn main() -> Result<(), lambda::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_lambda::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-lambda/latest/aws_sdk_lambda/client/struct.Client.html)
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

