/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

import { Stack, StackProps } from "aws-cdk-lib";
import * as eks from "aws-cdk-lib/aws-eks";
import * as dynamodb from "aws-cdk-lib/aws-dynamodb";
import { Construct } from "constructs";

export class EksCredentialsStack extends Stack {
  constructor(scope: Construct, id: string, props?: StackProps) {
    super(scope, id, props);


    // create a cluster
    const cluster = new eks.Cluster(this, 'hello-eks', {
      version: eks.KubernetesVersion.V1_21,
    });
    const serviceAccount = cluster.addServiceAccount("eks-service-account");
    const table = new dynamodb.Table(this, 'Table', {
      partitionKey: { name: 'id', type: dynamodb.AttributeType.STRING }
    });
    table.grantReadWriteData(serviceAccount);

    // apply a kubernetes manifest to the cluster
    const pod = cluster.addManifest('rust-sdk-test', {
      apiVersion: 'v1',
      kind: 'Pod',
      metadata: { name: 'rust-sdk-test' },
      spec: {
        serviceAccountName: serviceAccount.serviceAccountName,
        containers: [
          {
            name: 'hello',
            image: 'rust:buster',
            ports: [{ containerPort: 8080 }],
            command: ['sh', '-c', 'sleep infinity'],
            env: [{name: 'DYNAMO_TABLE', value: table.tableName}]
          }
        ]
      }
    });
    pod.node.addDependency(serviceAccount);
  }
}
