# aws-sdk-opsworkscm

AWS OpsWorks for configuration management (CM) is a service that runs and manages configuration management servers. You can use AWS OpsWorks CM to create and manage AWS OpsWorks for Chef Automate and AWS OpsWorks for Puppet Enterprise servers, and add or remove nodes for the servers to manage.

__Glossary of terms__
  - __Server__: A configuration management server that can be highly-available. The configuration management server runs on an Amazon Elastic Compute Cloud (EC2) instance, and may use various other AWS services, such as Amazon Relational Database Service (RDS) and Elastic Load Balancing. A server is a generic abstraction over the configuration manager that you want to use, much like Amazon RDS. In AWS OpsWorks CM, you do not start or stop servers. After you create servers, they continue to run until they are deleted.
  - __Engine__: The engine is the specific configuration manager that you want to use. Valid values in this release include ChefAutomate and Puppet.
  - __Backup__: This is an application-level backup of the data that the configuration manager stores. AWS OpsWorks CM creates an S3 bucket for backups when you launch the first server. A backup maintains a snapshot of a server's configuration-related attributes at the time the backup starts.
  - __Events__: Events are always related to a server. Events are written during server creation, when health checks run, when backups are created, when system maintenance is performed, etc. When you delete a server, the server's events are also deleted.
  - __Account attributes__: Every account has attributes that are assigned in the AWS OpsWorks CM database. These attributes store information about configuration limits (servers, backups, etc.) and your customer account.

__Endpoints__

AWS OpsWorks CM supports the following endpoints, all HTTPS. You must connect to one of the following endpoints. Your servers can only be accessed or managed within the endpoint in which they are created.
  - opsworks-cm.us-east-1.amazonaws.com
  - opsworks-cm.us-east-2.amazonaws.com
  - opsworks-cm.us-west-1.amazonaws.com
  - opsworks-cm.us-west-2.amazonaws.com
  - opsworks-cm.ap-northeast-1.amazonaws.com
  - opsworks-cm.ap-southeast-1.amazonaws.com
  - opsworks-cm.ap-southeast-2.amazonaws.com
  - opsworks-cm.eu-central-1.amazonaws.com
  - opsworks-cm.eu-west-1.amazonaws.com

For more information, see [AWS OpsWorks endpoints and quotas](https://docs.aws.amazon.com/general/latest/gr/opsworks-service.html) in the AWS General Reference.

__Throttling limits__

All API operations allow for five requests per second with a burst of 10 requests per second.

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-opsworkscm` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-opsworkscm = "0.0.0-local"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_opsworkscm as opsworkscm;

#[::tokio::main]
async fn main() -> Result<(), opsworkscm::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_opsworkscm::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-opsworkscm/latest/aws_sdk_opsworkscm/client/struct.Client.html)
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

