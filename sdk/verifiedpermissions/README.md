# aws-sdk-verifiedpermissions

Amazon Verified Permissions is a permissions management service from Amazon Web Services. You can use Verified Permissions to manage permissions for your application, and authorize user access based on those permissions. Using Verified Permissions, application developers can grant access based on information about the users, resources, and requested actions. You can also evaluate additional information like group membership, attributes of the resources, and session context, such as time of request and IP addresses. Verified Permissions manages these permissions by letting you create and store authorization policies for your applications, such as consumer-facing web sites and enterprise business systems.

Verified Permissions uses Cedar as the policy language to express your permission requirements. Cedar supports both role-based access control (RBAC) and attribute-based access control (ABAC) authorization models.

For more information about configuring, administering, and using Amazon Verified Permissions in your applications, see the [Amazon Verified Permissions User Guide](https://docs.aws.amazon.com/verifiedpermissions/latest/userguide/).

For more information about the Cedar policy language, see the [Cedar Policy Language Guide](https://docs.cedarpolicy.com/).

When you write Cedar policies that reference principals, resources and actions, you can define the unique identifiers used for each of those elements. We strongly recommend that you follow these best practices:
   - __Use values like universally unique identifiers (UUIDs) for all principal and resource identifiers.__ For example, if user jane leaves the company, and you later let someone else use the name jane, then that new user automatically gets access to everything granted by policies that still reference User::"jane". Cedar can’t distinguish between the new user and the old. This applies to both principal and resource identifiers. Always use identifiers that are guaranteed unique and never reused to ensure that you don’t unintentionally grant access because of the presence of an old identifier in a policy. Where you use a UUID for an entity, we recommend that you follow it with the // comment specifier and the ‘friendly’ name of your entity. This helps to make your policies easier to understand. For example: principal == User::"a1b2c3d4-e5f6-a1b2-c3d4-EXAMPLE11111", // alice
  - __Do not include personally identifying, confidential, or sensitive information as part of the unique identifier for your principals or resources.__ These identifiers are included in log entries shared in CloudTrail trails.

Several operations return structures that appear similar, but have different purposes. As new functionality is added to the product, the structure used in a parameter of one operation might need to change in a way that wouldn't make sense for the same parameter in a different operation. To help you understand the purpose of each, the following naming convention is used for the structures:
  - Parameter type structures that end in Detail are used in Get operations.
  - Parameter type structures that end in Item are used in List operations.
  - Parameter type structures that use neither suffix are used in the mutating (create and update) operations.

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-verifiedpermissions` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-verifiedpermissions = "0.0.0-local"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_verifiedpermissions as verifiedpermissions;

#[::tokio::main]
async fn main() -> Result<(), verifiedpermissions::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_verifiedpermissions::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-verifiedpermissions/latest/aws_sdk_verifiedpermissions/client/struct.Client.html)
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

