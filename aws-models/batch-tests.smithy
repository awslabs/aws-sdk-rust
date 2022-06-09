$version: "1.0"

namespace com.amazonaws.batch
use smithy.test#httpResponseTests
apply DescribeComputeEnvironments @httpResponseTests([
    {
        id: "DeserializeDescribeCompute",
        documentation: "This test case validates a bug where unboxed primitives were incorrectly marked as required",
        body: """
            {
                "computeEnvironments":[{
                    "computeEnvironmentName":"test-batch-compute",
                    "computeEnvironmentArn":"arn",
                    "ecsClusterArn":"clusteran",
                    "tags":{"foo": "bar"},
                    "type":"MANAGED",
                    "state":"ENABLED",
                    "status":"VALID",
                    "statusReason":"ComputeEnvironment Healthy",
                    "computeResources":{
                        "type":"EC2",
                        "minvCpus":0,
                        "maxvCpus":256,
                        "desiredvCpus":0,
                        "instanceTypes":["optimal"],
                        "subnets":["subnet-c745b79c","subnet-d4e24fe8"],
                        "securityGroupIds":["sg-06a55e7b"],
                        "instanceRole":"instancerole",
                        "tags":{"Name":"batch-compute"},
                        "ec2Configuration":[{"imageType":"ECS_AL1"}]
                    },
                    "serviceRole":"arn:aws:iam::432762038596:role/service-role/AWSBatchServiceRole"
                }]
            }
        """,
        code: 200,
        protocol: "aws.protocols#restJson1",
        params: {
                "computeEnvironments":[{
                    "computeEnvironmentName":"test-batch-compute",
                    "computeEnvironmentArn":"arn",
                    "ecsClusterArn":"clusteran",
                    "tags":{"foo": "bar"},
                    "type":"MANAGED",
                    "state":"ENABLED",
                    "status":"VALID",
                    "statusReason":"ComputeEnvironment Healthy",
                    "computeResources":{
                        "type":"EC2",
                        "minvCpus":0,
                        "maxvCpus":256,
                        "desiredvCpus":0,
                        "instanceTypes":["optimal"],
                        "subnets":["subnet-c745b79c","subnet-d4e24fe8"],
                        "securityGroupIds":["sg-06a55e7b"],
                        "instanceRole":"instancerole",
                        "tags":{"Name":"batch-compute"},
                        "ec2Configuration":[{"imageType":"ECS_AL1"}]
                    },
                    "serviceRole":"arn:aws:iam::432762038596:role/service-role/AWSBatchServiceRole"
                }]
            }
    }
])
