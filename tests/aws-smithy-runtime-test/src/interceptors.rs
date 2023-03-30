/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

// type TxReq = http::Request<SdkBody>;
// type TxRes = http::Response<SdkBody>;
//
// pub struct SigV4SigningConfigInterceptor {
//     pub signing_service: &'static str,
//     pub signing_region: Option<aws_types::region::Region>,
// }

// // Mount the interceptors
// let mut interceptors = Interceptors::new();
// let sig_v4_signing_config_interceptor = SigV4SigningConfigInterceptor {
//     signing_region: service_config.region.clone(),
//     signing_service: service_config.signing_service(),
// };
// let credentials_cache_interceptor = CredentialsCacheInterceptor {
//     shared_credentials_cache: service_config.credentials_cache.clone(),
// };
// let checksum_interceptor = ChecksumInterceptor {
//     checksum_mode: input.checksum_mode().cloned(),
// };
// interceptors
//     .with_interceptor(sig_v4_signing_config_interceptor)
//     .with_interceptor(credentials_cache_interceptor)
//     .with_interceptor(checksum_interceptor);

// let token_bucket = Box::new(standard::TokenBucket::builder().max_tokens(500).build());
//
// impl<ModReq, ModRes> Interceptor<ModReq, TxReq, TxRes, ModRes> for SigV4SigningConfigInterceptor {
//     fn modify_before_signing(
//         &mut self,
//         context: &mut InterceptorContext<ModReq, TxReq, TxRes, ModRes>,
//     ) -> Result<(), InterceptorError> {
//         let mut props = context.properties_mut();
//
//         let mut signing_config = OperationSigningConfig::default_config();
//         signing_config.signing_options.content_sha256_header = true;
//         signing_config.signing_options.double_uri_encode = false;
//         signing_config.signing_options.normalize_uri_path = false;
//         props.insert(signing_config);
//         props.insert(aws_types::SigningService::from_static(self.signing_service));
//
//         if let Some(signing_region) = self.signing_region.as_ref() {
//             props.insert(aws_types::region::SigningRegion::from(
//                 signing_region.clone(),
//             ));
//         }
//
//         Ok(())
//     }
// }
//
// pub struct CredentialsCacheInterceptor {
//     pub shared_credentials_cache: SharedCredentialsCache,
// }
//
// impl<ModReq, ModRes> Interceptor<ModReq, TxReq, TxRes, ModRes> for CredentialsCacheInterceptor {
//     fn modify_before_signing(
//         &mut self,
//         context: &mut InterceptorContext<ModReq, TxReq, TxRes, ModRes>,
//     ) -> Result<(), InterceptorError> {
//         match self
//             .shared_credentials_cache
//             .as_ref()
//             .provide_cached_credentials()
//             .now_or_never()
//         {
//             Some(Ok(creds)) => {
//                 context.properties_mut().insert(creds);
//             }
//             // ignore the case where there is no credentials cache wired up
//             Some(Err(CredentialsError::CredentialsNotLoaded { .. })) => {
//                 tracing::info!("credentials cache returned CredentialsNotLoaded, ignoring")
//             }
//             // if we get another error class, there is probably something actually wrong that the user will
//             // want to know about
//             Some(Err(other)) => return Err(InterceptorError::ModifyBeforeSigning(other.into())),
//             None => unreachable!("fingers crossed that creds are always available"),
//         }
//
//         Ok(())
//     }
// }
//
// pub struct ChecksumInterceptor {
//     pub checksum_mode: Option<ChecksumMode>,
// }
//
// impl<ModReq, ModRes> Interceptor<ModReq, TxReq, TxRes, ModRes> for ChecksumInterceptor {
//     fn modify_before_serialization(
//         &mut self,
//         context: &mut InterceptorContext<ModReq, TxReq, TxRes, ModRes>,
//     ) -> Result<(), InterceptorError> {
//         let mut props = context.properties_mut();
//         props.insert(self.checksum_mode.clone());
//
//         Ok(())
//     }
// }
