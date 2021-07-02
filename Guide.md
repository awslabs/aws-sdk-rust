# AWS SDK for Rust Developer Guide

This document is a preliminary version of the Developer Guide for the AWS SDK for Rust (the SDK).

This version of the guide is a pared down version of a typical developer guide, with the goal of helping the user quickly and easily download, consume, and use the SDK.

## Getting started 

This section briefly describes:

* How to get an Amazon account
* How to get your AWS access keys
* How to install the SDK
* How to add a dependency to an Amazon or AWS package in Cargo.toml

### Getting an Amazon account

Before you can use the AWS SDK for Rust, you must have an Amazon account. See [How do I create and activate a new Amazon Web Services account?](https://aws.amazon.com/premiumsupport/knowledge-center/create-and-activate-aws-account) for details.

### Getting your AWS access keys

Once you have an Amazon account, you need access keys to call any service API using the AWS SDK for Rust. 
For instructions on creating an access key for an existing IAM user, see [Programmatic access](https://docs.aws.amazon.com/general/latest/gr/aws-sec-cred-types.html#access-keys-and-secret-access-keys) in the [IAM User Guide](https://docs.aws.amazon.com/IAM/latest/UserGuide/).

### Installing the SDK

During the early development stage of the SDK, you can install the SDK by unzipping the asset from any of the [releases](https://github.com/awslabs/aws-sdk-rust/releases).

The SDK currently requires a minimum of Rust 1.52.1, and is not guaranteed to build on compiler versions earlier than that. 

### Using packages

To access an Amazon or AWS service using the AWS SDK for Rust you must specify the service’s crate in your Cargo.toml file. 
For example, to access Amazon Simple Storage Service (Amazon S3) APIs, you must include the following entry in the `dependencies `section:

```
s3 = { git = "https://github.com/awslabs/aws-sdk-rust", tag = "v1.0", package = "aws-sdk-s3" }
```

## Using the SDK 

This section describes:

* How to specify your credentials
* The environment variables the SDK acknowledges, such as AWS_REGION
* How to create a client, including specifying an AWS Region and credentials

### Specifying your credentials and region

Currently the recommended way to specify your credentials is through environment variables. See the following section for information on setting the __AWS_ACCESS_KEY_ID__ and __AWS_SECRET_ACCESS_KEY__ environment variables.

You can also specify the default AWS Region in which your client is created by using the __AWS_REGION__ environment variable. 

### Environment variables

The AWS SDK for Rust recognizes the following environment variables:

- __AWS_ACCESS_KEY_ID__ is the AWS access key used as part of the credentials to authenticate you.
- __AWS_REGION__ is the AWS Region to send requests to for commands requested using your profile.
- __AWS_SECRET_ACCESS_KEY__ is the AWS secret key used as part of the credentials to authenticate you.

To set an environment variable on Linux or MacOS, use the following command, where _VARIABLE_ is the name of the environment variable and _VALUE_ is the value to which it is set.:

```
export VARIABLE=VALUE
```

To do the same on Windows:

```
set VARIABLE=VALUE
```

### Creating a client

To create a client that uses values specified through environment variables, use the Client's __from_env__ function. 
The following example creates a client for Amazon S3:

```
 let client = s3::Client::from_env();
```

## API reference

You can find the API reference for the AWS SDK for Rust at [https://awslabs.github.io/aws-sdk-rust/](https://awslabs.github.io/aws-sdk-rust/).

## Code examples 

The AWS SDK for Rust examples can help you write your own Rust applications that use Amazon Web Services. The examples assume you have already set up and configured the SDK (that is, you have imported all required packages and set your credentials and region).

You can find the source code for these examples and others in the [AWS documentation code examples repository](https://github.com/awsdocs/aws-doc-sdk-examples) on GitHub. To propose a new code example for the AWS documentation team to consider producing, create a new request. The team is looking to produce code examples that cover broader scenarios and use cases, versus simple code snippets that cover only individual API calls. For instructions, see the "Proposing new code examples" section in the Readme on GitHub.

This section contains code examples for the following services:

[AWS Batch](sdk/examples/batch)
[AWS CloudFormation](sdk/examples/cloudformation)
[Amazon DynamoDB](sdk/examples/dynamodb)
[Amazon EC2](sdk/examples/ec2)
[Amazon Kinesis](sdk/examples/kinesis)
[AWS KMS](sdk/examples/kms)
[AWS Lambda](sdk/examples/lambda)
[AWS Elemental MediaLive](sdk/examples/medialive)
[AWS Elemental MediaPackage](sdk/examples/mediapackage)
[Amazon Polly](sdk/examples/polly)
[Amazon QLDB](sdk/examples/qldb)
[Amazon RDS](sdk/examples/rds)
[Amazon RDS Data](sdk/examples/rdsdata)
[Amazon Route 53](sdk/examples/route53)
[Amazon S3](sdk/examples/s3)
[Amazon SageMaker](sdk/examples/sagemaker)
[AWS Secrets Manager](sdk/examples/secretsmanager)
[Amazon SES](sdk/examples/ses)
[Amazon SNS](sdk/examples/sns)
[Amazon SQS](sdk/examples/sqs)
[AWS Systems Manager](sdk/examples/ssm)
[AWS STS](sdk/examples/sts)
