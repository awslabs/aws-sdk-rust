# aws-sdk-codepipeline

**Please Note: The SDK is currently in Developer Preview and is intended strictly for
feedback purposes only. Do not use this SDK for production workloads.**

__Overview__

This is the AWS CodePipeline API Reference. This guide provides descriptions of the actions and data types for AWS CodePipeline. Some functionality for your pipeline can only be configured through the API. For more information, see the [AWS CodePipeline User Guide](https://docs.aws.amazon.com/codepipeline/latest/userguide/welcome.html).

You can use the AWS CodePipeline API to work with pipelines, stages, actions, and transitions.

_Pipelines_ are models of automated release processes. Each pipeline is uniquely named, and consists of stages, actions, and transitions.

You can work with pipelines by calling:
  - CreatePipeline, which creates a uniquely named pipeline.
  - DeletePipeline, which deletes the specified pipeline.
  - GetPipeline, which returns information about the pipeline structure and pipeline metadata, including the pipeline Amazon Resource Name (ARN).
  - GetPipelineExecution, which returns information about a specific execution of a pipeline.
  - GetPipelineState, which returns information about the current state of the stages and actions of a pipeline.
  - ListActionExecutions, which returns action-level details for past executions. The details include full stage and action-level details, including individual action duration, status, any errors that occurred during the execution, and input and output artifact location details.
  - ListPipelines, which gets a summary of all of the pipelines associated with your account.
  - ListPipelineExecutions, which gets a summary of the most recent executions for a pipeline.
  - StartPipelineExecution, which runs the most recent revision of an artifact through the pipeline.
  - StopPipelineExecution, which stops the specified pipeline execution from continuing through the pipeline.
  - UpdatePipeline, which updates a pipeline with edits or changes to the structure of the pipeline.

Pipelines include _stages_. Each stage contains one or more actions that must complete before the next stage begins. A stage results in success or failure. If a stage fails, the pipeline stops at that stage and remains stopped until either a new version of an artifact appears in the source location, or a user takes action to rerun the most recent artifact through the pipeline. You can call GetPipelineState, which displays the status of a pipeline, including the status of stages in the pipeline, or GetPipeline, which returns the entire structure of the pipeline, including the stages of that pipeline. For more information about the structure of stages and actions, see [AWS CodePipeline Pipeline Structure Reference](https://docs.aws.amazon.com/codepipeline/latest/userguide/pipeline-structure.html).

Pipeline stages include _actions_ that are categorized into categories such as source or build actions performed in a stage of a pipeline. For example, you can use a source action to import artifacts into a pipeline from a source such as Amazon S3. Like stages, you do not work with actions directly in most cases, but you do define and interact with actions when working with pipeline operations such as CreatePipeline and GetPipelineState. Valid action categories are:
  - Source
  - Build
  - Test
  - Deploy
  - Approval
  - Invoke

Pipelines also include _transitions_, which allow the transition of artifacts from one stage to the next in a pipeline after the actions in one stage complete.

You can work with transitions by calling:
  - DisableStageTransition, which prevents artifacts from transitioning to the next stage in a pipeline.
  - EnableStageTransition, which enables transition of artifacts between stages in a pipeline.

__Using the API to integrate with AWS CodePipeline__

For third-party integrators or developers who want to create their own integrations with AWS CodePipeline, the expected sequence varies from the standard API user. To integrate with AWS CodePipeline, developers need to work with the following items:

__Jobs__, which are instances of an action. For example, a job for a source action might import a revision of an artifact from a source.

You can work with jobs by calling:
  - AcknowledgeJob, which confirms whether a job worker has received the specified job.
  - GetJobDetails, which returns the details of a job.
  - PollForJobs, which determines whether there are any jobs to act on.
  - PutJobFailureResult, which provides details of a job failure.
  - PutJobSuccessResult, which provides details of a job success.

__Third party jobs__, which are instances of an action created by a partner action and integrated into AWS CodePipeline. Partner actions are created by members of the AWS Partner Network.

You can work with third party jobs by calling:
  - AcknowledgeThirdPartyJob, which confirms whether a job worker has received the specified job.
  - GetThirdPartyJobDetails, which requests the details of a job for a partner action.
  - PollForThirdPartyJobs, which determines whether there are any jobs to act on.
  - PutThirdPartyJobFailureResult, which provides details of a job failure.
  - PutThirdPartyJobSuccessResult, which provides details of a job success.

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-codepipeline` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = "0.14.0"
aws-sdk-codepipeline = "0.14.0"
tokio = { version = "1", features = ["full"] }
```

## Using the SDK

Until the SDK is released, we will be adding information about using the SDK to the
[Developer Guide](https://docs.aws.amazon.com/sdk-for-rust/latest/dg/welcome.html). Feel free to suggest
additional sections for the guide by opening an issue and describing what you are trying to do.

## Getting Help

* [GitHub discussions](https://github.com/awslabs/aws-sdk-rust/discussions) - For ideas, RFCs & general questions
* [GitHub issues](https://github.com/awslabs/aws-sdk-rust/issues/new/choose) – For bug reports & feature requests
* [Generated Docs (latest version)](https://awslabs.github.io/aws-sdk-rust/)
* [Usage examples](https://github.com/awslabs/aws-sdk-rust/tree/main/examples)

## License

This project is licensed under the Apache-2.0 License.

