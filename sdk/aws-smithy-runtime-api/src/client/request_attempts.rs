/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_smithy_types::config_bag::{Storable, StoreReplace};

#[derive(Debug, Clone, Copy)]
pub struct RequestAttempts {
    attempts: u32,
}

impl RequestAttempts {
    #[cfg(any(feature = "test-util", test))]
    pub fn new(attempts: u32) -> Self {
        Self { attempts }
    }

    pub fn attempts(&self) -> u32 {
        self.attempts
    }
}

impl From<u32> for RequestAttempts {
    fn from(attempts: u32) -> Self {
        Self { attempts }
    }
}

impl Storable for RequestAttempts {
    type Storer = StoreReplace<Self>;
}
