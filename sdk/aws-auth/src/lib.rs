/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

//! AWS authentication middleware used to store and retrieve credentials from the property bag

pub mod middleware;

use aws_types::credentials::SharedCredentialsProvider;
use smithy_http::property_bag::PropertyBag;

pub fn set_provider(bag: &mut PropertyBag, provider: SharedCredentialsProvider) {
    bag.insert(provider);
}
