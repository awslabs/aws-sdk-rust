/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

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

impl From<usize> for RequestAttempts {
    fn from(attempts: usize) -> Self {
        Self { attempts }
    }
}
