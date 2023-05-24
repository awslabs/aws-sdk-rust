/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::client::auth::AuthSchemeId;

pub const HTTP_API_KEY_AUTH_SCHEME_ID: AuthSchemeId = AuthSchemeId::new("http-api-key-auth");
pub const HTTP_BASIC_AUTH_SCHEME_ID: AuthSchemeId = AuthSchemeId::new("http-basic-auth");
pub const HTTP_BEARER_AUTH_SCHEME_ID: AuthSchemeId = AuthSchemeId::new("http-bearer-auth");
pub const HTTP_DIGEST_AUTH_SCHEME_ID: AuthSchemeId = AuthSchemeId::new("http-digest-auth");
