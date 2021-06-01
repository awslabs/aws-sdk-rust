#[tokio::main]
async fn main() -> Result<(), aws_sdk_ecs::Error> {
    let client = aws_sdk_ecs::Client::from_env();
    let cluster = client
        .create_cluster()
        .cluster_name("test_cluster")
        .send()
        .await?;
    println!("cluster created: {:?}", cluster);

    let cluster_deleted = client
        .delete_cluster()
        .cluster("test_cluster")
        .send()
        .await?;
    println!("cluster deleted: {:?}", cluster_deleted);
    Ok(())
}
