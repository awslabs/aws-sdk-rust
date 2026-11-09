/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::{prelude, Schema, ShapeId};
use std::fmt;

/// The identity and schemas of one Smithy operation.
///
/// An operation is not itself serialized, so it has no [`Schema`]; this descriptor relates the
/// operation's shape ID to the schemas a protocol needs to call it: the input, the output, and
/// the errors it can return.
///
/// The descriptor borrows everything, so it is cheap to construct and to clone. It is built
/// with [`OperationSchema::builder`], whose methods are `const`, so it can be declared as a
/// `static` alongside generated schemas:
///
/// ```
/// use aws_smithy_schema::{shape_id, OperationSchema, Schema, ShapeType};
///
/// static INPUT: Schema<'static> =
///     Schema::new_struct(shape_id!("com.example", "GetThingInput"), ShapeType::Structure, &[]);
/// static OUTPUT: Schema<'static> =
///     Schema::new_struct(shape_id!("com.example", "GetThingOutput"), ShapeType::Structure, &[]);
/// static NOT_FOUND: Schema<'static> =
///     Schema::new_struct(shape_id!("com.example", "NotFound"), ShapeType::Structure, &[]);
///
/// static GET_THING: OperationSchema<'static> =
///     OperationSchema::builder(shape_id!("com.example", "GetThing"))
///         .input(&INPUT)
///         .output(&OUTPUT)
///         .errors(&[&NOT_FOUND])
///         .build();
///
/// assert_eq!(GET_THING.id().as_str(), "com.example#GetThing");
/// assert_eq!(GET_THING.input().shape_id().as_str(), "com.example#GetThingInput");
/// assert_eq!(GET_THING.errors().len(), 1);
/// ```
///
/// # Input and output schemas
///
/// Protocols read operation-level behavior, such as the `@http` trait and body hints, from the
/// input schema, and S3's unwrapped XML output from the output schema. The input and output
/// schemas of a descriptor are therefore specific to the operation: when two operations share
/// an input structure, each has its own input schema with that structure's ID, members, and
/// traits plus its own operation-level behavior.
///
/// # Errors
///
/// [`errors`](Self::errors) are the operation's errors in the context the descriptor was built
/// for. For an operation of a service, that is the service's common errors followed by the
/// operation's own, without duplicates. Without a service, it is the operation's own errors.
///
/// # Lifetimes
///
/// A descriptor cannot outlive the schemas it references:
///
/// ```compile_fail,E0597
/// use aws_smithy_schema::{shape_id, OperationSchema, Schema, ShapeType};
///
/// let operation = {
///     let unit = Schema::new(shape_id!("smithy.api", "Unit"), ShapeType::Structure);
///     OperationSchema::builder(shape_id!("com.example", "Op"))
///         .input(&unit)
///         .build()
/// };
/// # let _ = operation;
/// ```
#[derive(Clone)]
pub struct OperationSchema<'a> {
    id: ShapeId<'a>,
    input: &'a Schema<'a>,
    output: &'a Schema<'a>,
    errors: &'a [&'a Schema<'a>],
}

impl<'a> OperationSchema<'a> {
    /// A builder for the descriptor of the operation `id`.
    ///
    /// The input and output default to [`prelude::UNIT`], as in a Smithy model that omits
    /// them, and the errors default to none.
    pub const fn builder(id: ShapeId<'a>) -> OperationSchemaBuilder<'a> {
        OperationSchemaBuilder {
            id,
            input: &prelude::UNIT,
            output: &prelude::UNIT,
            errors: &[],
        }
    }

    /// The operation's shape ID.
    ///
    /// Returned by value: a [`ShapeId`] only borrows its strings, so this does not allocate,
    /// and the result is not tied to the lifetime of the descriptor value itself.
    pub fn id(&self) -> ShapeId<'a> {
        self.id.clone()
    }

    /// The input schema for this operation, to pass to a protocol's `serialize_request`.
    pub fn input(&self) -> &'a Schema<'a> {
        self.input
    }

    /// The output schema for this operation, to pass to a protocol's `deserialize_response`.
    pub fn output(&self) -> &'a Schema<'a> {
        self.output
    }

    /// The error structures the operation can return.
    pub fn errors(&self) -> &'a [&'a Schema<'a>] {
        self.errors
    }
}

impl fmt::Debug for OperationSchema<'_> {
    /// Shows shape IDs only, not the complete schemas.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        debug_ids(
            f,
            "OperationSchema",
            &self.id,
            self.input,
            self.output,
            self.errors,
        )
    }
}

/// Builds an [`OperationSchema`]. Created by [`OperationSchema::builder`].
///
/// Every method is `const`, so a descriptor can be built in a `static` initializer. Settings
/// added in the future will have defaults, so existing builders keep compiling.
#[derive(Clone)]
#[must_use]
pub struct OperationSchemaBuilder<'a> {
    id: ShapeId<'a>,
    input: &'a Schema<'a>,
    output: &'a Schema<'a>,
    errors: &'a [&'a Schema<'a>],
}

impl<'a> OperationSchemaBuilder<'a> {
    /// Sets the input schema, to pass to a protocol's `serialize_request`. Defaults to
    /// [`prelude::UNIT`].
    pub const fn input(mut self, input: &'a Schema<'a>) -> Self {
        self.input = input;
        self
    }

    /// Sets the output schema, to pass to a protocol's `deserialize_response`. Defaults to
    /// [`prelude::UNIT`].
    pub const fn output(mut self, output: &'a Schema<'a>) -> Self {
        self.output = output;
        self
    }

    /// Sets the error structures the operation can return, in the context the descriptor is
    /// built for (see [`OperationSchema::errors`]). Defaults to none.
    pub const fn errors(mut self, errors: &'a [&'a Schema<'a>]) -> Self {
        self.errors = errors;
        self
    }

    /// Builds the descriptor.
    pub const fn build(self) -> OperationSchema<'a> {
        OperationSchema {
            id: self.id,
            input: self.input,
            output: self.output,
            errors: self.errors,
        }
    }
}

impl fmt::Debug for OperationSchemaBuilder<'_> {
    /// Shows shape IDs only, not the complete schemas.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        debug_ids(
            f,
            "OperationSchemaBuilder",
            &self.id,
            self.input,
            self.output,
            self.errors,
        )
    }
}

fn debug_ids(
    f: &mut fmt::Formatter<'_>,
    name: &str,
    id: &ShapeId<'_>,
    input: &Schema<'_>,
    output: &Schema<'_>,
    errors: &[&Schema<'_>],
) -> fmt::Result {
    struct Ids<'s, 'a>(&'s [&'a Schema<'a>]);
    impl fmt::Debug for Ids<'_, '_> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_list()
                .entries(self.0.iter().map(|schema| schema.shape_id().as_str()))
                .finish()
        }
    }
    f.debug_struct(name)
        .field("id", &id.as_str())
        .field("input", &input.shape_id().as_str())
        .field("output", &output.shape_id().as_str())
        .field("errors", &Ids(errors))
        .finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{shape_id, ShapeType};

    static INPUT: Schema<'static> =
        Schema::new_struct(shape_id!("com.example", "Input"), ShapeType::Structure, &[]);
    static OUTPUT: Schema<'static> = Schema::new_struct(
        shape_id!("com.example", "Output"),
        ShapeType::Structure,
        &[],
    );
    static ERROR_A: Schema<'static> = Schema::new_struct(
        shape_id!("com.example", "ErrorA"),
        ShapeType::Structure,
        &[],
    );
    static ERROR_B: Schema<'static> = Schema::new_struct(
        shape_id!("com.example", "ErrorB"),
        ShapeType::Structure,
        &[],
    );

    static OPERATION: OperationSchema<'static> =
        OperationSchema::builder(shape_id!("com.example", "Operation"))
            .input(&INPUT)
            .output(&OUTPUT)
            .errors(&[&ERROR_B, &ERROR_A])
            .build();

    #[test]
    fn static_descriptor() {
        let id = OPERATION.id();
        assert_eq!(id.as_str(), "com.example#Operation");
        assert_eq!(id.namespace(), "com.example");
        assert_eq!(id.shape_name(), "Operation");
        assert_eq!(id.member_name(), None);
        assert!(std::ptr::eq(OPERATION.input(), &INPUT));
        assert!(std::ptr::eq(OPERATION.output(), &OUTPUT));
        let errors: Vec<_> = OPERATION
            .errors()
            .iter()
            .map(|error| error.shape_id().as_str())
            .collect();
        // Order is preserved.
        assert_eq!(errors, ["com.example#ErrorB", "com.example#ErrorA"]);
    }

    #[test]
    fn borrowed_descriptor() {
        let fqn = String::from("com.example#Borrowed");
        let id = ShapeId::from_parts(&fqn, &fqn[..11], &fqn[12..]);
        let input = Schema::new_struct(id.clone(), ShapeType::Structure, &[]);
        let errors = [&ERROR_A];
        let operation = OperationSchema::builder(id)
            .input(&input)
            .output(&OUTPUT)
            .errors(&errors)
            .build();
        // The ID outlives the descriptor value it came from.
        let id = operation.clone().id();
        assert_eq!(id.as_str(), "com.example#Borrowed");
        assert_eq!(id.namespace(), "com.example");
        assert_eq!(id.shape_name(), "Borrowed");
        assert!(std::ptr::eq(id.as_str(), fqn.as_str()));
        assert!(std::ptr::eq(operation.input(), &input));
    }

    #[test]
    fn builder_defaults_match_an_omitted_input_and_output() {
        static DEFAULTS: OperationSchema<'static> =
            OperationSchema::builder(shape_id!("com.example", "Ping")).build();
        assert!(std::ptr::eq(DEFAULTS.input(), &prelude::UNIT));
        assert!(std::ptr::eq(DEFAULTS.output(), &prelude::UNIT));
        assert!(DEFAULTS.errors().is_empty());
        // A later setter replaces an earlier one.
        let operation = OperationSchema::builder(shape_id!("com.example", "Ping"))
            .input(&INPUT)
            .input(&OUTPUT)
            .build();
        assert!(std::ptr::eq(operation.input(), &OUTPUT));
    }

    #[test]
    fn debug_shows_ids_only() {
        assert_eq!(
            format!("{OPERATION:?}"),
            r#"OperationSchema { id: "com.example#Operation", input: "com.example#Input", output: "com.example#Output", errors: ["com.example#ErrorB", "com.example#ErrorA"] }"#
        );
        assert_eq!(
            format!(
                "{:?}",
                OperationSchema::builder(shape_id!("com.example", "Ping"))
            ),
            r#"OperationSchemaBuilder { id: "com.example#Ping", input: "smithy.api#Unit", output: "smithy.api#Unit", errors: [] }"#
        );
    }

    #[test]
    #[cfg(target_pointer_width = "64")]
    fn size_stays_within_its_budget() {
        // A 64-byte `ShapeId`, two schema references, and an error slice. Update deliberately.
        assert_eq!(std::mem::size_of::<OperationSchema<'static>>(), 96);
    }
}
