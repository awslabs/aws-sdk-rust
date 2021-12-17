#!/bin/bash
#
# Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0.
#

# If running this locally, you'll need to make a clone of awslabs/smithy-rs in
# the aws-sdk-rust project root.
#
# Inputs (environment variables)
# ------------------------------
#
# AWS_REGION:
#     (Used indirectly by the AWS CLI) The region to run the canary in
# AWS_PROFILE:
#     (Used indirectly by the AWS CLI) The credentials profile to use
# SDK_VERSION:
#     Version of the SDK to compile the canary against
# LAMBDA_CODE_S3_BUCKET_NAME:
#     The name of the S3 bucket to upload the canary binary bundle to
# LAMBDA_TEST_S3_BUCKET_NAME:
#     The name of the S3 bucket for the canary Lambda to interact with
# LAMBDA_EXECUTION_ROLE_ARN:
#     The ARN of the role that the Lambda will execute as

# TODO: Make the canary Cargo.toml generator output an empty workspace so that
# the aws-sdk-rust root manifest doesn't cause the build to fail. Then remove this `rm`.
rm -f "$(git rev-parse --show-toplevel)/Cargo.toml"

set -e
cd "$(git rev-parse --show-toplevel)/smithy-rs/tools/ci-cdk/canary-lambda"
pwd

echo "Generating canary Cargo.toml..."
./write-cargo-toml.py --sdk-version $SDK_VERSION
echo "Building the canary..."
./build-bundle.sh $SDK_VERSION > bundle-path

BUNDLE_PATH=$(cat bundle-path)
BUNDLE_FILE_NAME=$(basename ${BUNDLE_PATH})
BUNDLE_NAME=${BUNDLE_FILE_NAME%.*}

# Note: the `&>/dev/null` is to avoid logging sensitive information from the CLI
echo "Uploading Lambda code bundle to S3..."
aws s3 cp ${BUNDLE_PATH} s3://${LAMBDA_CODE_S3_BUCKET_NAME}/${BUNDLE_FILE_NAME} &>/dev/null

# Note: the `&>/dev/null` is to avoid logging sensitive information from the CLI
echo "Creating Lambda function named ${BUNDLE_NAME}..."
aws lambda create-function \
    --function-name ${BUNDLE_NAME} \
    --runtime provided.al2 \
    --role ${LAMBDA_EXECUTION_ROLE_ARN} \
    --handler aws-sdk-rust-lambda-canary \
    --code S3Bucket=${LAMBDA_CODE_S3_BUCKET_NAME},S3Key=${BUNDLE_FILE_NAME} \
    --publish \
    --environment "Variables={RUST_BACKTRACE=1,CANARY_S3_BUCKET_NAME=${LAMBDA_TEST_S3_BUCKET_NAME},CANARY_EXPECTED_TRANSCRIBE_RESULT='Good day to you transcribe. This is Polly talking to you from the Rust ST K.'}" \
    --timeout 60 &>/dev/null

echo "Waiting for Lambda function to be ready..."
aws lambda wait function-active --function-name ${BUNDLE_NAME}

# Don't automatically fail the script for the next step since we would like to clean up after
set +e

# TODO: Don't log environment variables in the canary's implementation so
# that `--log-type` can safely be set to `Tail`. Using `--log-type None` currently
# since the canary will log ARNs and other things that shouldn't output here
echo "Invoking the canary Lambda..."
CANARY_FAILED=0
aws lambda invoke \
    --function-name ${BUNDLE_NAME} \
    --invocation-type RequestResponse \
    --log-type None \
    --payload '{}' \
    canary-output
if [[ $? -ne 0 ]]; then
    echo "Canary Lambda execution failed. Look up log group /aws/lambda/${BUNDLE_NAME} in CloudWatch to see logs."
    CANARY_FAILED=1
fi

echo "Canary result:"
cat canary-output
echo
grep --quiet '"result":"success"' canary-output
if [[ $? -ne 0 ]]; then
    echo "Canary failed!"
    CANARY_FAILED=1
fi

set -e
echo "Deleting the canary Lambda..."
aws lambda delete-function \
    --function-name ${BUNDLE_NAME}

exit ${CANARY_FAILED}
