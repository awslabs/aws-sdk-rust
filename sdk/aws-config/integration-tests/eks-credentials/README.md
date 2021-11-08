# CDK Stack for EKS credentials provider testing

This project defines a CDK stack that launches an EKS cluster, creates a DynamoDB table, and sets up a service account role so that the DynamoDB pod can access the table.

`test.rs` is provided as an example script to run.

## Usage
```bsh
cdk bootstrap aws://accountid/region
cdk deploy
# make lunch, go for a bike ride, etc. ~1h.
kubectl exec rust-sdk-test -it bash
# write some rust code, e.g. test.rs, run it.
```
