# aws-sdk-route53

Amazon Route 53 is a highly available and scalable Domain Name System (DNS) web service. Route 53 performs four main functions:
  - __Domain registration__ – Route 53 lets you register domain names such as example.com.
  - __Domain Name System (DNS) service__ – Route 53 translates friendly domain names into IP addresses using a global network of authoritative DNS servers, and provides DNS resolution within your VPCs for private hosted zones.
  - __Health checking__ – Route 53 sends automated requests to your application, over the internet or from within your VPC for private endpoints, to verify that it's reachable, available, and functional.
  - __Resolver__ – Route 53 Resolver lets you forward DNS queries from a VPC that you created using Amazon VPC to DNS resolvers in your network, and from your network to resolvers in your VPC.

You can use the API to create and manage the following resources:

__Public and Private Hosted Zones__

A _public hosted zone_ is a container that holds information about how you want to route traffic on the internet for a domain, such as example.com, and its subdomains. A _private hosted zone_ is a container that holds information about how you want to route traffic for a domain and its subdomains within one or more VPCs that you created with the Amazon VPC service.

__Reusable Delegation Sets__

By default, each hosted zone that you create gets a different set of four name servers—a different delegation set. If you create a lot of hosted zones, maintaining different delegation sets can be difficult and time consuming. Route 53 lets you create a delegation set that you can reuse with multiple hosted zones.

__Resource Record Sets__

After you create a hosted zone for your domain, such as example.com, you create resource record sets to tell the Domain Name System (DNS) how to route traffic for that domain.

__Traffic Policies and Traffic Policy Instances__

You can create complex routing configurations, known as traffic policies, that use weighted, latency, failover, and geolocation resource record sets. You can then associate a traffic policy with a domain name or subdomain name, such as www.example.com, by creating a traffic policy instance. When users submit DNS queries for the domain or subdomain, Route 53 responds based on the traffic policy that you used to create the traffic policy instance.

__Health Checks__

Route 53 health checks monitor the health and performance of your web applications, web servers, and other resources. At regular intervals that you specify, Route 53 submits automated requests to your application, server, or other resource, over the internet or from within your VPC for private endpoints, to verify that it's reachable, available, and functional.

__Domain Registrations__

When you want to get a new domain name, such as example.com, you can register it with Route 53. You can also transfer the registration for existing domains from other registrars to Route 53.

__DNS–DNSSEC__

You can protect your domain from DNS spoofing or a man-in-the-middle attack, by configuring Domain Name System Security Extensions (DNSSEC).

__Query Logs__

You can configure Route 53 to log information about the DNS queries that Route 53 receives, such as the domain or subdomain that was requested, the date and time of the request, and the DNS record type (such as A or AAAA). You can also configure Route 53 Resolver to log information about the DNS queries that originate in Amazon VPCs.

__Outbound and Inbound Endpoints, and Rules__

You can configure Route 53 Resolver to forward DNS queries from your VPC to your network or vice versa. DNS queries pass through an outbound endpoint on their way from a VPC to your network, and they pass through an inbound endpoint on their way from your network to a VPC. For outbound queries, rules let you specify the domain names that you want to forward to your network and the IP addresses of the DNS resolvers in your network.

__Tags__

A tag is a label that you assign to an Amazon Web Services resource. Each tag consists of a key and a value, both of which you define. You can use tags for a variety of purposes; one common use is to categorize and track your Route 53 costs.

__Profiles__

A _profile_ is a named container that groups DNS configurations—such as private hosted zone associations, Resolver rules, and DNS Firewall rule groups—into a single reusable unit. You can associate a profile with multiple VPCs and share it across Amazon Web Services accounts so that each VPC inherits the same DNS settings without requiring individual configuration.

__DNS Firewall__

A _DNS Firewall rule group_ is an ordered collection of rules that each pair a domain list with an action—allow, block, or alert. You associate rule groups with your VPCs to filter outbound DNS queries made by VPC resources, preventing connections to known-malicious domains or restricting access to only approved domains.


You can also use the Route 53 API to get the current limit on Route 53 objects that you can create, such as hosted zones and health checks.

For information about Route 53 concepts and about how to use the Route 53 console, see the [Amazon Route 53 Developer Guide](https://docs.aws.amazon.com/Route53/latest/DeveloperGuide/).

## Getting Started

> Examples are available for many services and operations, check out the
> [usage examples](https://github.com/awsdocs/aws-doc-sdk-examples/tree/main/rustv1).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-route53` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-route53 = "1.139.0"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_route53 as route53;

#[::tokio::main]
async fn main() -> Result<(), route53::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_route53::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-route53/latest/aws_sdk_route53/client/struct.Client.html)
for information on what calls can be made, and the inputs and outputs for each of those calls.

## Using the SDK

Until the SDK is released, we will be adding information about using the SDK to the
[Developer Guide](https://docs.aws.amazon.com/sdk-for-rust/latest/dg/welcome.html). Feel free to suggest
additional sections for the guide by opening an issue and describing what you are trying to do.

## Getting Help

* [GitHub discussions](https://github.com/awslabs/aws-sdk-rust/discussions) - For ideas, RFCs & general questions
* [GitHub issues](https://github.com/awslabs/aws-sdk-rust/issues/new/choose) - For bug reports & feature requests
* [Generated Docs (latest version)](https://awslabs.github.io/aws-sdk-rust/)
* [Usage examples](https://github.com/awsdocs/aws-doc-sdk-examples/tree/main/rustv1)

## License

This project is licensed under the Apache-2.0 License.

