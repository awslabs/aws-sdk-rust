# CDK Stack for EKS credentials provider testing

This project defines a CDK stack that launches an EKS cluster, creates a DynamoDB table, and sets up:
1. A service account role that allows a pod to access the table via IRSA.
2. A service account and pod identity association that allows a pod to access the table via EKS Pod Identity.

`test.rs` is provided as an example script to run.

## Usage
```bsh
cdk bootstrap aws://accountid/region
cdk deploy
# make lunch, go for a bike ride, etc. ~1h.
kubectl exec rust-sdk-test-irsa -it bash
# write some rust code, e.g. test.rs, run it. will have irsa identity
kubectl exec rust-sdk-test-pod-identity -it bash
# run more rust code. will have eks pod identity
```
