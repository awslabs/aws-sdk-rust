use aws_sdk_sfn::{Client, Error};
use std::process::exit;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    /// The Amazon Resource Name (ARN) of the state machine to execute.
    #[structopt(short, long)]
    arn: Option<String>,
}

/// Starts a state machine execution.
#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();

    let Opt {
        arn,
        input,
    } = Opt::from_args();

    let shared_config = aws_config::load_from_env().await;
    let client = Client::new(&shared_config);

    println!();

    if verbose {
        println!("SF arn: {}", &arn);
        println!();
    }
    
    let rsp = client
        .stop_execution()
        .execution_arn(&arn)
        .send()
        .await?;

    println!(
        "Step function response: `{:?}`",
        rsp
    );

    Ok(())
}
