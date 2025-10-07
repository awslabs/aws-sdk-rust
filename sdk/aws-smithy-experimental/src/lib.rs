/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

/* Automatically managed default lints */
#![cfg_attr(docsrs, feature(doc_cfg))]
/* End of automatically managed default lints */

#[deprecated(
    since = "0.2.0",
    note = "support for hyper-1.x is now enabled by the `aws-smithy-http-client` crate"
)]
pub mod hyper_1_0 {}
