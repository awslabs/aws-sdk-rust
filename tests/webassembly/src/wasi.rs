/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

// Needed for WASI-compliant environment as it expects specific functions
// to be exported such as `cabi_realloc`, `_start`, etc.

wit_bindgen::generate!({
    inline: "
        package aws:component;

        interface run {
            run: func() -> result;
        }

        world main {
            export run;
        }
    ",
    exports: {
        "aws:component/run": Component
    }
});

struct Component;

impl exports::aws::component::run::Guest for Component {
    fn run() -> Result<(), ()> {
        Ok(())
    }
}
