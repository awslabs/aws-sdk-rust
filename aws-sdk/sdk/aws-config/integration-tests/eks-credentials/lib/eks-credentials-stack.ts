/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

import * as cdk from '@aws-cdk/core';
import * as eks from '@aws-cdk/aws-eks';
import * as dynamodb from '@aws-cdk/aws-dynamodb';
export class EksCredentialsStack extends cdk.Stack {
  constructor(scope: cdk.Construct, id: string, props?: cdk.StackProps) {
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
