Where did the files in this directory come from?
================================================

These test files were taken from the now defunct Signature Version 4 Test Suite documentation
from the [AWS General Reference](https://docs.aws.amazon.com/general/latest/gr/Welcome.html).

Signature Version 4 Test Suite
------------------------------

To assist you in the development of an AWS client that supports Signature Version 4, you can use the
files in the test suite to ensure your code is performing each step of the signing process correctly.

Each test group contains five files that you can use to validate each of the tasks described in
Signature Version 4 Signing Process. The following list describes the contents of each file.

- file-name.req - the web request to be signed.
- file-name.creq - the resulting canonical request.
- file-name.sts - the resulting string to sign.
- file-name.authz - the Authorization header.
- file-name.sreq - the signed request.

The examples in the test suite use the following credential scope:

```
AKIDEXAMPLE/20150830/us-east-1/service/aws4_request
```

The example secret key used for signing is:

```
wJalrXUtnFEMI/K7MDENG+bPxRfiCYEXAMPLEKEY
```

Changes Made to the Test Suite for the Rust SDK
-----------------------------------------------

Some additions were made to the test suite for the Rust SDK:
- `iam/iam.creq` was added to facilitate signature calculation unit tests.
- `file-name.qpsreq` was added to represent a request that was signed with query parameters.
