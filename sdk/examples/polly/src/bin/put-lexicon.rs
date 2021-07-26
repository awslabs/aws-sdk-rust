/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use aws_types::region::ProvideRegion;
use polly::{Client, Config, Error, Region, PKG_VERSION};

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    /// The default AWS Region.
    #[structopt(short, long)]
    default_region: Option<String>,

    /// The name of the lexicon.
    #[structopt(short, long)]
    name: String,

    /// The word to replace.
    #[structopt(short, long)]
    from: String,

    /// The replacement.
    #[structopt(short, long)]
    to: String,

    /// Whether to show additional output.
    #[structopt(short, long)]
    verbose: bool,
}

/// Stores a pronunciation lexicon in a Region.
/// # Arguments
///
/// * `-f FROM` - The original text to customize.
/// * `-n NAME` - The name of the lexicon.
/// * `-t TO` - The customized version of the original text.
/// * `[-d DEFAULT-REGION]` - The Region in which the client is created.
///    If not supplied, uses the value of the **AWS_REGION** environment variable.
///    If the environment variable is not set, defaults to **us-west-2**.
/// * `[-v]` - Whether to display additional information.
#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();

    let Opt {
        from,
        name,
        default_region,
        to,
        verbose,
    } = Opt::from_args();

    let region = default_region
        .as_ref()
        .map(|region| Region::new(region.clone()))
        .or_else(|| aws_types::region::default_provider().region())
        .unwrap_or_else(|| Region::new("us-west-2"));

    println!();

    if verbose {
        println!("Polly version:    {}", PKG_VERSION);
        println!("Region:           {:?}", &region);
        println!("Lexicon name:     {}", &name);
        println!("Text to replace:  {}", &from);
        println!("Replacement text: {}", &to);
        println!();
    }

    let config = Config::builder().region(region).build();
    let client = Client::from_conf(config);

    let content = format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>
    <lexicon version=\"1.0\" xmlns=\"http://www.w3.org/2005/01/pronunciation-lexicon\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\"
    xsi:schemaLocation=\"http://www.w3.org/2005/01/pronunciation-lexicon http://www.w3.org/TR/2007/CR-pronunciation-lexicon-20071212/pls.xsd\"
    alphabet=\"ipa\" xml:lang=\"en-US\">
    <lexeme><grapheme>{}</grapheme><alias>{}</alias></lexeme>
    </lexicon>", from, to);

    client
        .put_lexicon()
        .name(name)
        .content(content)
        .send()
        .await?;

    println!("Added lexicon");

    Ok(())
}
