$version: "1.0"

namespace com.amazonaws.sqs
use smithy.test#httpRequestTests

apply ChangeMessageVisibility @httpRequestTests([
    {
        id: "SqsSetVisibilityZero",
        documentation: "This test case validates a bug found here: https://github.com/aws/aws-sdk-go-v2/issues/1087",
        params: {
            QueueUrl: "http://somequeue.amazon.com",
            ReceiptHandle: "handlehandle",
            VisibilityTimeout: 0
        },
        body: "Action=ChangeMessageVisibility&Version=2012-11-05&QueueUrl=http%3A%2F%2Fsomequeue.amazon.com&ReceiptHandle=handlehandle&VisibilityTimeout=0",
        protocol: "aws.protocols#awsQuery",
        bodyMediaType: "application/x-www-formurl-encoded",
        method: "POST",
        uri: "/"
    }
])
