use aws_sdk_eks::model::VpcConfigRequest;
use aws_sdk_eks::Region;
use aws_types::region;
use aws_types::region::ProvideRegion;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    /// The region
    #[structopt(short, long)]
    region: Option<String>,

    #[structopt(short, long)]
    cluster_name: String,

    /// Role ARN for the cluster
    /// To create a role-arn:
    ///
    /// 1. Follow instructions to create an IAM role:
    /// https://docs.aws.amazon.com/eks/latest/userguide/service_IAM_role.html
    ///
    /// 2. Copy role arn
    #[structopt(long)]
    role_arn: String,

    /// subnet id
    ///
    /// At least two subnet ids must be specified. The subnet ids must be in two separate AZs
    #[structopt(short, long)]
    subnet_id: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<(), aws_sdk_eks::Error> {
    let Opt {
        region,
        cluster_name,
        role_arn,
        subnet_id,
    } = Opt::from_args();
    let region = region
        .map(Region::new)
        .or_else(|| region::default_provider().region())
        .unwrap_or_else(|| Region::from_static("us-west-2"));
    let conf = aws_sdk_eks::Config::builder().region(region).build();
    let client = aws_sdk_eks::Client::from_conf(conf);
    let cluster = client
        .create_cluster()
        .name(&cluster_name)
        .role_arn(role_arn)
        .resources_vpc_config(
            VpcConfigRequest::builder()
                .set_subnet_ids(Some(subnet_id))
                .build(),
        )
        .send()
        .await?;
    println!("cluster created: {:?}", cluster);

    let cluster_deleted = client.delete_cluster().name(&cluster_name).send().await?;
    println!("cluster deleted: {:?}", cluster_deleted);
    Ok(())
}
