/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

import { Stack, StackProps } from "aws-cdk-lib";
import * as eks from "aws-cdk-lib/aws-eks";
import * as iam from "aws-cdk-lib/aws-iam";
import * as dynamodb from "aws-cdk-lib/aws-dynamodb";
import { Construct } from "constructs";
import {KubectlV28Layer} from "@aws-cdk/lambda-layer-kubectl-v28";

export class EksCredentialsStack extends Stack {
  constructor(scope: Construct, id: string, props?: StackProps) {
    super(scope, id, props);


    // create a cluster
    const cluster = new eks.Cluster(this, 'hello-eks', {
      version: eks.KubernetesVersion.V1_28,
      kubectlLayer: new KubectlV28Layer(this, 'hello-eks-kubectl'),
    });

    const podIdentityAddon = new eks.CfnAddon(this, 'eks-pod-identity-addon', {
      addonName: 'eks-pod-identity-agent',
      clusterName: cluster.clusterName,
      addonVersion: 'v1.1.0-eksbuild.1',
    });

    const serviceAccountIRSA = cluster.addServiceAccount("eks-service-account-irsa");
    const serviceAccountPodIdentity = cluster.addManifest("eks-service-account-pod-identity", {
      apiVersion: "v1",
      kind: "ServiceAccount",
      metadata: {
        name: "eks-service-account-pod-identity",
        namespace: "default",
        labels: {"app.kubernetes.io/name": "eks-service-account-pod-identity"},
      }
    })

    const table = new dynamodb.Table(this, 'Table', {
      partitionKey: { name: 'id', type: dynamodb.AttributeType.STRING }
    });
    table.grantReadWriteData(serviceAccountIRSA);

    let podIdentityPrincipal = new iam.ServicePrincipal("pods.eks.amazonaws.com")

    let podIdentityRole = new iam.Role(cluster, "eks-role-pod-identity", {
      assumedBy: podIdentityPrincipal,
    })
    podIdentityRole.assumeRolePolicy?.addStatements(iam.PolicyStatement.fromJson({
          "Sid": "AllowEksAuthToAssumeRoleForPodIdentity",
          "Effect": "Allow",
          "Principal": {
            "Service": "pods.eks.amazonaws.com"
          },
          "Action": [
            "sts:AssumeRole",
            "sts:TagSession"
          ]
    }))
    table.grantReadWriteData(podIdentityRole);

    let podIdentityAssociation = new eks.CfnPodIdentityAssociation(cluster, "eks-pod-identity-associations", {
      roleArn: podIdentityRole.roleArn,
      clusterName: cluster.clusterName,
      namespace: "default",
      serviceAccount: "eks-service-account-pod-identity",
    })

    // apply a kubernetes manifest to the cluster
    const podIRSA = cluster.addManifest('rust-sdk-test-irsa', {
      apiVersion: 'v1',
      kind: 'Pod',
      metadata: { name: 'rust-sdk-test-irsa' },
      spec: {
        serviceAccountName: serviceAccountIRSA.serviceAccountName,
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
    podIRSA.node.addDependency(serviceAccountIRSA);

    const podPodIdentity = cluster.addManifest('rust-sdk-test-pod-identity', {
      apiVersion: 'v1',
      kind: 'Pod',
      metadata: { name: 'rust-sdk-test-pod-identity' },
      spec: {
        serviceAccountName: "eks-service-account-pod-identity",
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
    podPodIdentity.node.addDependency(serviceAccountPodIdentity);
    podPodIdentity.node.addDependency(podIdentityAssociation);
    podPodIdentity.node.addDependency(podIdentityAddon)
  }
}
