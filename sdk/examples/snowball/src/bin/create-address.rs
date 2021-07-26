/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use aws_sdk_snowball::model::Address;
use aws_sdk_snowball::{Config, Region};
use aws_types::region;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    /// The default region
    #[structopt(short, long)]
    region: Option<String>,

    // Address information
    #[structopt(long)]
    city: Option<String>,

    #[structopt(long)]
    company: Option<String>,

    #[structopt(long)]
    country: Option<String>,

    #[structopt(long)]
    landmark: Option<String>,

    #[structopt(long)]
    name: Option<String>,

    #[structopt(long)]
    phone_number: Option<String>,

    #[structopt(long)]
    postal_code: Option<String>,

    #[structopt(long)]
    prefecture_or_district: Option<String>,

    #[structopt(long)]
    state: Option<String>,

    #[structopt(long)]
    street1: Option<String>,

    #[structopt(long)]
    street2: Option<String>,

    #[structopt(long)]
    street3: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), aws_sdk_snowball::Error> {
    tracing_subscriber::fmt::init();

    let Opt {
        region,
        city,
        company,
        country,
        landmark,
        name,
        phone_number,
        postal_code,
        prefecture_or_district,
        state,
        street1,
        street2,
        street3,
    } = Opt::from_args();

    let region_provider = region::ChainProvider::first_try(region.map(Region::new))
        .or_default_provider()
        .or_else(Region::new("us-west-2"));

    let new_address = Address::builder()
        .set_address_id(None)
        .set_name(name)
        .set_company(company)
        .set_street1(street1)
        .set_street2(street2)
        .set_street3(street3)
        .set_city(city)
        .set_state_or_province(state)
        .set_prefecture_or_district(prefecture_or_district)
        .set_landmark(landmark)
        .set_country(country)
        .set_postal_code(postal_code)
        .set_phone_number(phone_number)
        .set_is_restricted(Some(false))
        .build();

    let conf = Config::builder().region(region_provider).build();
    let client = aws_sdk_snowball::Client::from_conf(conf);

    let result = client.create_address().address(new_address).send().await?;

    println!("Address: {:?}", result.address_id.unwrap());

    Ok(())
}
