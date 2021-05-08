# Duct Tape

In the legendary [Knuth v McIlroy word count competition](http://www.leancrew.com/all-this/2011/12/more-shell-less-egg/), most would say the shell won.

These are examples of interfacing with the [AWS CLI](https://github.com/aws/aws-cli), or the shell in general, to get things done.


```bash

## Sign an S3 bucket so it can be shared
aws s3 presign s3://awsexamplebucket/test2.txt --expires-in 604800

## Delelop on a remote EC2 instance 
aws ec2 start-instances i-INSTANCE_ID
aws ec2 describe-instances # TODO pipe to jq to get IP address
ssh  username@INSTANCE_IP       -i keyfile   # alternatively connect with vscode
# shut down for the day
aws ec2 stop-instances --instance-ids i-INSTANCE_ID
```



