/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub enum Type {
    Bool,
    Null,
    Undefined,
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    Int,
    F16,
    F32,
    F64,
    Simple,
    Bytes,
    BytesIndef,
    String,
    StringIndef,
    Array,
    ArrayIndef,
    Map,
    MapIndef,
    Tag,
    Break,
    Unknown(u8),
}

impl Type {
    pub(crate) fn new(ty: minicbor::data::Type) -> Self {
        match ty {
            minicbor::data::Type::Bool => Self::Bool,
            minicbor::data::Type::Null => Self::Null,
            minicbor::data::Type::Undefined => Self::Undefined,
            minicbor::data::Type::U8 => Self::U8,
            minicbor::data::Type::U16 => Self::U16,
            minicbor::data::Type::U32 => Self::U32,
            minicbor::data::Type::U64 => Self::U64,
            minicbor::data::Type::I8 => Self::I8,
            minicbor::data::Type::I16 => Self::I16,
            minicbor::data::Type::I32 => Self::I32,
            minicbor::data::Type::I64 => Self::I64,
            minicbor::data::Type::Int => Self::Int,
            minicbor::data::Type::F16 => Self::F16,
            minicbor::data::Type::F32 => Self::F32,
            minicbor::data::Type::F64 => Self::F64,
            minicbor::data::Type::Simple => Self::Simple,
            minicbor::data::Type::Bytes => Self::Bytes,
            minicbor::data::Type::BytesIndef => Self::BytesIndef,
            minicbor::data::Type::String => Self::String,
            minicbor::data::Type::StringIndef => Self::StringIndef,
            minicbor::data::Type::Array => Self::Array,
            minicbor::data::Type::ArrayIndef => Self::ArrayIndef,
            minicbor::data::Type::Map => Self::Map,
            minicbor::data::Type::MapIndef => Self::MapIndef,
            minicbor::data::Type::Tag => Self::Tag,
            minicbor::data::Type::Break => Self::Break,
            minicbor::data::Type::Unknown(byte) => Self::Unknown(byte),
        }
    }

    // This is just the reverse mapping of `new`.
    pub(crate) fn into_minicbor_type(self) -> minicbor::data::Type {
        match self {
            Type::Bool => minicbor::data::Type::Bool,
            Type::Null => minicbor::data::Type::Null,
            Type::Undefined => minicbor::data::Type::Undefined,
            Type::U8 => minicbor::data::Type::U8,
            Type::U16 => minicbor::data::Type::U16,
            Type::U32 => minicbor::data::Type::U32,
            Type::U64 => minicbor::data::Type::U64,
            Type::I8 => minicbor::data::Type::I8,
            Type::I16 => minicbor::data::Type::I16,
            Type::I32 => minicbor::data::Type::I32,
            Type::I64 => minicbor::data::Type::I64,
            Type::Int => minicbor::data::Type::Int,
            Type::F16 => minicbor::data::Type::F16,
            Type::F32 => minicbor::data::Type::F32,
            Type::F64 => minicbor::data::Type::F64,
            Type::Simple => minicbor::data::Type::Simple,
            Type::Bytes => minicbor::data::Type::Bytes,
            Type::BytesIndef => minicbor::data::Type::BytesIndef,
            Type::String => minicbor::data::Type::String,
            Type::StringIndef => minicbor::data::Type::StringIndef,
            Type::Array => minicbor::data::Type::Array,
            Type::ArrayIndef => minicbor::data::Type::ArrayIndef,
            Type::Map => minicbor::data::Type::Map,
            Type::MapIndef => minicbor::data::Type::MapIndef,
            Type::Tag => minicbor::data::Type::Tag,
            Type::Break => minicbor::data::Type::Break,
            Type::Unknown(byte) => minicbor::data::Type::Unknown(byte),
        }
    }
}
