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

### Supported Rust versions

The SDK currently requires a minimum of Rust 1.52.1, and is not guaranteed to build on compiler versions earlier than that. 

### Using packages

To access an Amazon or AWS service using the AWS SDK for Rust you must specify the service’s crate in your **Cargo.toml** file. 
For example, to access Amazon Simple Storage Service (Amazon S3) APIs using the v1.0 version of the Rust SDK, you must include the following entry in the `dependencies `section:

```
s3 = { git = "https://github.com/awslabs/aws-sdk-rust", tag = "v1.0", package = "aws-sdk-s3" }
```

## Using the SDK 

This section describes:

* How to specify your credentials
* The environment variables the SDK acknowledges, such as AWS_REGION
* How to create a client, including specifying an AWS Region and credentials

### Specifying your credentials and region

Currently the recommended way to specify your credentials is through environment variables. See the following section for information on setting the **AWS_ACCESS_KEY_ID** and **AWS_SECRET_ACCESS_KEY** environment variables.

You can also specify the default AWS Region in which your client is created by using the **AWS_REGION** environment variable. 

### Environment variables

The AWS SDK for Rust recognizes the following environment variables:

- **AWS_ACCESS_KEY_ID** is the AWS access key used as part of the credentials to authenticate you.
- **AWS_SECRET_ACCESS_KEY** is the AWS secret key used as part of the credentials to authenticate you.
- **AWS_REGION** is the AWS Region to which requests are sent.

To set an environment variable on Linux or MacOS, use the following command, where *VARIABLE* is the name of the environment variable and *VALUE* is the value to which it is set.:

```
export VARIABLE=VALUE
```

To do the same on Windows:

```
set VARIABLE=VALUE
```

### Creating a client

  To create a client that uses values specified through environment variables, use the Client's `from_env` function. 
The following example creates a client for Amazon S3:

```rust
 let client = s3::Client::from_env();
```

## API reference

You can find the API reference for the AWS SDK for Rust at [https://awslabs.github.io/aws-sdk-rust/](https://awslabs.github.io/aws-sdk-rust/).

## Code examples 

The AWS SDK for Rust examples can help you write your own Rust applications that use Amazon Web Services. The examples assume you have already set up and configured the SDK (that is, you have imported all required packages and set your credentials and region).

You can find the source code for these examples and others in the [sdk/examples](sdk/examples) section of this repository. To propose a new code example, create an  issue and describe what you want the cod example to do. 
The **sdk/examples** section contains code examples for the following services:

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
