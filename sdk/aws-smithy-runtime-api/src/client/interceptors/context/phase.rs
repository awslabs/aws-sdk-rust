/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

#[derive(Debug)]
#[non_exhaustive]
pub enum Phase {
    /// Represents the phase of an operation prior to serialization.
    BeforeSerialization,
    /// Represents the phase of an operation where the request is serialized.
    Serialization,
    /// Represents the phase of an operation prior to transmitting a request over the network.
    BeforeTransmit,
    /// Represents the phase of an operation where the request is transmitted over the network.
    Transmit,
    /// Represents the phase of an operation prior to parsing a response.
    BeforeDeserialization,
    /// Represents the phase of an operation where the response is parsed.
    Deserialization,
    /// Represents the phase of an operation after parsing a response.
    AfterDeserialization,
}

impl Phase {
    pub fn is_before_serialization(&self) -> bool {
        matches!(self, Self::BeforeSerialization)
    }

    pub fn is_serialization(&self) -> bool {
        matches!(self, Self::Serialization)
    }

    pub fn is_before_transmit(&self) -> bool {
        matches!(self, Self::BeforeTransmit)
    }

    pub fn is_transmit(&self) -> bool {
        matches!(self, Self::Transmit)
    }

    pub fn is_before_deserialization(&self) -> bool {
        matches!(self, Self::BeforeDeserialization)
    }

    pub fn is_deserialization(&self) -> bool {
        matches!(self, Self::Deserialization)
    }

    pub fn is_after_deserialization(&self) -> bool {
        matches!(self, Self::AfterDeserialization)
    }
}
