/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

#[tokio::main]
async fn main() -> Result<(), sagemaker::Error> {
    let client = sagemaker::Client::from_env();
    let notebooks = client.list_notebook_instances().send().await?;

    for n in notebooks.notebook_instances.unwrap_or_default() {
        let n_instance_type = n.instance_type.unwrap();
        let n_status = n.notebook_instance_status.unwrap();
        let n_name = n.notebook_instance_name.as_deref().unwrap_or_default();

        let details = format!(
            "Notebook Name : {}, Notebook Status : {:#?}, Notebook Instance Type : {:#?}",
            n_name, n_status, n_instance_type
        );
        println!("{}", details);
    }

    Ok(())
}
