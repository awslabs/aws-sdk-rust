---
name: 🐛 Bug Report
about: If something isn't working as expected 🤔.

---

## Bug Report
<!--
Thank you for reporting an issue.

Please fill in as much of the template below as you're able.
-->

### Version

<!--
List the versions & crates of the aws-rust-sdk you are using.

`cargo install cargo-tree`
(see install here: https://github.com/sfackler/cargo-tree)

Then:

`cargo tree | grep aws-sdk-`
-->

### Platform

<!---
Output of `uname -a` (UNIX), or version and 32 or 64-bit (Windows)
-->

### AWS Services

<!--
If relevant, please specify the impacted services. Otherwise, delete this
section.
-->

### Description

<!--

Enter your issue details below this comment.

One way to structure the description:

<short summary of the bug>

I tried this code:

<code sample that causes the bug>

I expected to see this happen: <explanation>

Instead, this happened: <explanation>

It's also helpful to enable trace logging and include the
log messages as these will show the actual HTTP requests and
responses. You can enable this by initializing `tracing-subscriber`
if you haven't already (e.g., `tracing_subscriber::fmt::init();`),
and then setting the environment variable `RUST_LOG` before
running your program, as follows:

`RUST_LOG='smithy_http_tower::dispatch=trace,smithy_http::middleware=trace'`

For example:

`RUST_LOG='smithy_http_tower::dispatch=trace,smithy_http::middleware=trace' cargo run`
-->
