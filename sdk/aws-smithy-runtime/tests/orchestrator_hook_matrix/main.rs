/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

#![cfg(all(
    feature = "client",
    any(feature = "test-util", feature = "legacy-test-util")
))]

//! A table-driven regression net over every interceptor hook.
//!
//! Rather than one targeted test per bug, this module asserts the *expected invocation count of
//! every hook in every scenario*. A deviation in either direction fails: a hook that stops firing,
//! and a hook that starts firing when it should not. That is what makes it a regression net rather
//! than a fix-specific test.
//!
//! Three things are asserted for each run:
//!
//! 1. **Counts** — symbolic, because attempt-level and execution-level hooks have different
//!    cardinality. See [`Expect`].
//! 2. **Error fidelity** — what the finalizer hooks actually *observed*. A count-only matrix would
//!    accept a fix where all four finalizers fire but see an empty context, which would leave
//!    `modify_before_attempt_completion` and `modify_before_completion` unable to do their
//!    documented job.
//! 3. **Lifecycle ordering** — every step advances through `known_hooks!()` order, with one
//!    permitted exception: a new attempt resets to `read_before_attempt`.
//!
//! The hook list itself is checked against `known_hooks!()` so that adding a hook to the runtime
//! cannot silently leave it untested.
//!
//! Whether a hook is dispatched at all is gated by the `overridden_hooks()` bitmask, which defaults
//! to `all()`. The matrix recorder implements every hook, so its bitmask is all-ones and the gate is
//! effectively a no-op here; [`one_overridden_hook_still_dispatches`] covers the gate with a sparse
//! bitmask instead, which is the only way to tell that each hook consults the right flag.

mod matrix;
mod support;
