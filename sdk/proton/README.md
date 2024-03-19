# aws-sdk-proton

This is the Proton Service API Reference. It provides descriptions, syntax and usage examples for each of the [actions](https://docs.aws.amazon.com/proton/latest/APIReference/API_Operations.html) and [data types](https://docs.aws.amazon.com/proton/latest/APIReference/API_Types.html) for the Proton service.

The documentation for each action shows the Query API request parameters and the XML response.

Alternatively, you can use the Amazon Web Services CLI to access an API. For more information, see the [Amazon Web Services Command Line Interface User Guide](https://docs.aws.amazon.com/cli/latest/userguide/cli-chap-welcome.html).

The Proton service is a two-pronged automation framework. Administrators create service templates to provide standardized infrastructure and deployment tooling for serverless and container based applications. Developers, in turn, select from the available service templates to automate their application or service deployments.

Because administrators define the infrastructure and tooling that Proton deploys and manages, they need permissions to use all of the listed API operations.

When developers select a specific infrastructure and tooling set, Proton deploys their applications. To monitor their applications that are running on Proton, developers need permissions to the service _create_, _list_, _update_ and _delete_ API operations and the service instance _list_ and _update_ API operations.

To learn more about Proton, see the [Proton User Guide](https://docs.aws.amazon.com/proton/latest/userguide/Welcome.html).

__Ensuring Idempotency__

When you make a mutating API request, the request typically returns a result before the asynchronous workflows of the operation are complete. Operations might also time out or encounter other server issues before they're complete, even if the request already returned a result. This might make it difficult to determine whether the request succeeded. Moreover, you might need to retry the request multiple times to ensure that the operation completes successfully. However, if the original request and the subsequent retries are successful, the operation occurs multiple times. This means that you might create more resources than you intended.

_Idempotency_ ensures that an API request action completes no more than one time. With an idempotent request, if the original request action completes successfully, any subsequent retries complete successfully without performing any further actions. However, the result might contain updated information, such as the current creation status.

The following lists of APIs are grouped according to methods that ensure idempotency.

__Idempotent create APIs with a client token__

The API actions in this list support idempotency with the use of a _client token_. The corresponding Amazon Web Services CLI commands also support idempotency using a client token. A client token is a unique, case-sensitive string of up to 64 ASCII characters. To make an idempotent API request using one of these actions, specify a client token in the request. We recommend that you _don't_ reuse the same client token for other API requests. If you don’t provide a client token for these APIs, a default client token is automatically provided by SDKs.

Given a request action that has succeeded:

If you retry the request using the same client token and the same parameters, the retry succeeds without performing any further actions other than returning the original resource detail data in the response.

If you retry the request using the same client token, but one or more of the parameters are different, the retry throws a ValidationException with an IdempotentParameterMismatch error.

Client tokens expire eight hours after a request is made. If you retry the request with the expired token, a new resource is created.

If the original resource is deleted and you retry the request, a new resource is created.

Idempotent create APIs with a client token:
  - CreateEnvironmentTemplateVersion
  - CreateServiceTemplateVersion
  - CreateEnvironmentAccountConnection

__Idempotent create APIs__

Given a request action that has succeeded:

If you retry the request with an API from this group, and the original resource _hasn't_ been modified, the retry succeeds without performing any further actions other than returning the original resource detail data in the response.

If the original resource has been modified, the retry throws a ConflictException.

If you retry with different input parameters, the retry throws a ValidationException with an IdempotentParameterMismatch error.

Idempotent create APIs:
  - CreateEnvironmentTemplate
  - CreateServiceTemplate
  - CreateEnvironment
  - CreateService

__Idempotent delete APIs__

Given a request action that has succeeded:

When you retry the request with an API from this group and the resource was deleted, its metadata is returned in the response.

If you retry and the resource doesn't exist, the response is empty.

In both cases, the retry succeeds.

Idempotent delete APIs:
  - DeleteEnvironmentTemplate
  - DeleteEnvironmentTemplateVersion
  - DeleteServiceTemplate
  - DeleteServiceTemplateVersion
  - DeleteEnvironmentAccountConnection

__Asynchronous idempotent delete APIs__

Given a request action that has succeeded:

If you retry the request with an API from this group, if the original request delete operation status is DELETE_IN_PROGRESS, the retry returns the resource detail data in the response without performing any further actions.

If the original request delete operation is complete, a retry returns an empty response.

Asynchronous idempotent delete APIs:
  - DeleteEnvironment
  - DeleteService

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-proton` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-proton = "0.0.0-local"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_proton as proton;

#[::tokio::main]
async fn main() -> Result<(), proton::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_proton::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-proton/latest/aws_sdk_proton/client/struct.Client.html)
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

