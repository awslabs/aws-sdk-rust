/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_smithy_types::config_bag::{Storable, StoreReplace};

#[derive(Debug, Clone, Copy)]
pub struct RequestAttempts {
    attempts: usize,
}

impl RequestAttempts {
    #[cfg(any(feature = "test-util", test))]
    pub fn new(attempts: usize) -> Self {
        Self { attempts }
    }

    pub fn attempts(&self) -> usize {
        self.attempts
    }
}

impl Storable for RequestAttempts {
    type Storer = StoreReplace<Self>;
}

impl From<usize> for RequestAttempts {
    fn from(attempts: usize) -> Self {
        Self { attempts }
    }
}
