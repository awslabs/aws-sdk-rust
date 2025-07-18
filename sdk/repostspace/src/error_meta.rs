// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
/// All possible error types for this service.
#[non_exhaustive]
#[derive(::std::fmt::Debug)]
pub enum Error {
    /// <p>User does not have sufficient access to perform this action.</p>
    AccessDeniedException(crate::types::error::AccessDeniedException),
    /// <p>Updating or deleting a resource can cause an inconsistent state.</p>
    ConflictException(crate::types::error::ConflictException),
    /// <p>Unexpected error during processing of request.</p>
    InternalServerException(crate::types::error::InternalServerException),
    /// <p>Request references a resource which does not exist.</p>
    ResourceNotFoundException(crate::types::error::ResourceNotFoundException),
    /// <p>Request would cause a service quota to be exceeded.</p>
    ServiceQuotaExceededException(crate::types::error::ServiceQuotaExceededException),
    /// <p>Request was denied due to request throttling.</p>
    ThrottlingException(crate::types::error::ThrottlingException),
    /// <p>The input fails to satisfy the constraints specified by an AWS service.</p>
    ValidationException(crate::types::error::ValidationException),
    /// An unexpected error occurred (e.g., invalid JSON returned by the service or an unknown error code).
    #[deprecated(note = "Matching `Unhandled` directly is not forwards compatible. Instead, match using a \
    variable wildcard pattern and check `.code()`:
     \
    &nbsp;&nbsp;&nbsp;`err if err.code() == Some(\"SpecificExceptionCode\") => { /* handle the error */ }`
     \
    See [`ProvideErrorMetadata`](#impl-ProvideErrorMetadata-for-Error) for what information is available for the error.")]
    Unhandled(crate::error::sealed_unhandled::Unhandled),
}
impl ::std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::AccessDeniedException(inner) => inner.fmt(f),
            Error::ConflictException(inner) => inner.fmt(f),
            Error::InternalServerException(inner) => inner.fmt(f),
            Error::ResourceNotFoundException(inner) => inner.fmt(f),
            Error::ServiceQuotaExceededException(inner) => inner.fmt(f),
            Error::ThrottlingException(inner) => inner.fmt(f),
            Error::ValidationException(inner) => inner.fmt(f),
            Error::Unhandled(_) => {
                if let ::std::option::Option::Some(code) = ::aws_smithy_types::error::metadata::ProvideErrorMetadata::code(self) {
                    write!(f, "unhandled error ({code})")
                } else {
                    f.write_str("unhandled error")
                }
            }
        }
    }
}
impl From<::aws_smithy_types::error::operation::BuildError> for Error {
    fn from(value: ::aws_smithy_types::error::operation::BuildError) -> Self {
        Error::Unhandled(crate::error::sealed_unhandled::Unhandled {
            source: value.into(),
            meta: ::std::default::Default::default(),
        })
    }
}
impl ::aws_smithy_types::error::metadata::ProvideErrorMetadata for Error {
    fn meta(&self) -> &::aws_smithy_types::error::metadata::ErrorMetadata {
        match self {
            Self::AccessDeniedException(inner) => inner.meta(),
            Self::ConflictException(inner) => inner.meta(),
            Self::InternalServerException(inner) => inner.meta(),
            Self::ResourceNotFoundException(inner) => inner.meta(),
            Self::ServiceQuotaExceededException(inner) => inner.meta(),
            Self::ThrottlingException(inner) => inner.meta(),
            Self::ValidationException(inner) => inner.meta(),
            Self::Unhandled(inner) => &inner.meta,
        }
    }
}
impl<R>
    From<
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::batch_add_channel_role_to_accessors::BatchAddChannelRoleToAccessorsError,
            R,
        >,
    > for Error
where
    R: Send + Sync + std::fmt::Debug + 'static,
{
    fn from(
        err: ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::batch_add_channel_role_to_accessors::BatchAddChannelRoleToAccessorsError,
            R,
        >,
    ) -> Self {
        match err {
            ::aws_smithy_runtime_api::client::result::SdkError::ServiceError(context) => Self::from(context.into_err()),
            _ => Error::Unhandled(crate::error::sealed_unhandled::Unhandled {
                meta: ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(&err).clone(),
                source: err.into(),
            }),
        }
    }
}
impl From<crate::operation::batch_add_channel_role_to_accessors::BatchAddChannelRoleToAccessorsError> for Error {
    fn from(err: crate::operation::batch_add_channel_role_to_accessors::BatchAddChannelRoleToAccessorsError) -> Self {
        match err {
            crate::operation::batch_add_channel_role_to_accessors::BatchAddChannelRoleToAccessorsError::AccessDeniedException(inner) => {
                Error::AccessDeniedException(inner)
            }
            crate::operation::batch_add_channel_role_to_accessors::BatchAddChannelRoleToAccessorsError::InternalServerException(inner) => {
                Error::InternalServerException(inner)
            }
            crate::operation::batch_add_channel_role_to_accessors::BatchAddChannelRoleToAccessorsError::ResourceNotFoundException(inner) => {
                Error::ResourceNotFoundException(inner)
            }
            crate::operation::batch_add_channel_role_to_accessors::BatchAddChannelRoleToAccessorsError::ThrottlingException(inner) => {
                Error::ThrottlingException(inner)
            }
            crate::operation::batch_add_channel_role_to_accessors::BatchAddChannelRoleToAccessorsError::ValidationException(inner) => {
                Error::ValidationException(inner)
            }
            crate::operation::batch_add_channel_role_to_accessors::BatchAddChannelRoleToAccessorsError::Unhandled(inner) => Error::Unhandled(inner),
        }
    }
}
impl<R> From<::aws_smithy_runtime_api::client::result::SdkError<crate::operation::batch_add_role::BatchAddRoleError, R>> for Error
where
    R: Send + Sync + std::fmt::Debug + 'static,
{
    fn from(err: ::aws_smithy_runtime_api::client::result::SdkError<crate::operation::batch_add_role::BatchAddRoleError, R>) -> Self {
        match err {
            ::aws_smithy_runtime_api::client::result::SdkError::ServiceError(context) => Self::from(context.into_err()),
            _ => Error::Unhandled(crate::error::sealed_unhandled::Unhandled {
                meta: ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(&err).clone(),
                source: err.into(),
            }),
        }
    }
}
impl From<crate::operation::batch_add_role::BatchAddRoleError> for Error {
    fn from(err: crate::operation::batch_add_role::BatchAddRoleError) -> Self {
        match err {
            crate::operation::batch_add_role::BatchAddRoleError::AccessDeniedException(inner) => Error::AccessDeniedException(inner),
            crate::operation::batch_add_role::BatchAddRoleError::InternalServerException(inner) => Error::InternalServerException(inner),
            crate::operation::batch_add_role::BatchAddRoleError::ResourceNotFoundException(inner) => Error::ResourceNotFoundException(inner),
            crate::operation::batch_add_role::BatchAddRoleError::ThrottlingException(inner) => Error::ThrottlingException(inner),
            crate::operation::batch_add_role::BatchAddRoleError::ValidationException(inner) => Error::ValidationException(inner),
            crate::operation::batch_add_role::BatchAddRoleError::Unhandled(inner) => Error::Unhandled(inner),
        }
    }
}
impl<R>
    From<
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::batch_remove_channel_role_from_accessors::BatchRemoveChannelRoleFromAccessorsError,
            R,
        >,
    > for Error
where
    R: Send + Sync + std::fmt::Debug + 'static,
{
    fn from(
        err: ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::batch_remove_channel_role_from_accessors::BatchRemoveChannelRoleFromAccessorsError,
            R,
        >,
    ) -> Self {
        match err {
            ::aws_smithy_runtime_api::client::result::SdkError::ServiceError(context) => Self::from(context.into_err()),
            _ => Error::Unhandled(crate::error::sealed_unhandled::Unhandled {
                meta: ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(&err).clone(),
                source: err.into(),
            }),
        }
    }
}
impl From<crate::operation::batch_remove_channel_role_from_accessors::BatchRemoveChannelRoleFromAccessorsError> for Error {
    fn from(err: crate::operation::batch_remove_channel_role_from_accessors::BatchRemoveChannelRoleFromAccessorsError) -> Self {
        match err {
            crate::operation::batch_remove_channel_role_from_accessors::BatchRemoveChannelRoleFromAccessorsError::AccessDeniedException(inner) => {
                Error::AccessDeniedException(inner)
            }
            crate::operation::batch_remove_channel_role_from_accessors::BatchRemoveChannelRoleFromAccessorsError::InternalServerException(inner) => {
                Error::InternalServerException(inner)
            }
            crate::operation::batch_remove_channel_role_from_accessors::BatchRemoveChannelRoleFromAccessorsError::ResourceNotFoundException(
                inner,
            ) => Error::ResourceNotFoundException(inner),
            crate::operation::batch_remove_channel_role_from_accessors::BatchRemoveChannelRoleFromAccessorsError::ThrottlingException(inner) => {
                Error::ThrottlingException(inner)
            }
            crate::operation::batch_remove_channel_role_from_accessors::BatchRemoveChannelRoleFromAccessorsError::ValidationException(inner) => {
                Error::ValidationException(inner)
            }
            crate::operation::batch_remove_channel_role_from_accessors::BatchRemoveChannelRoleFromAccessorsError::Unhandled(inner) => {
                Error::Unhandled(inner)
            }
        }
    }
}
impl<R> From<::aws_smithy_runtime_api::client::result::SdkError<crate::operation::batch_remove_role::BatchRemoveRoleError, R>> for Error
where
    R: Send + Sync + std::fmt::Debug + 'static,
{
    fn from(err: ::aws_smithy_runtime_api::client::result::SdkError<crate::operation::batch_remove_role::BatchRemoveRoleError, R>) -> Self {
        match err {
            ::aws_smithy_runtime_api::client::result::SdkError::ServiceError(context) => Self::from(context.into_err()),
            _ => Error::Unhandled(crate::error::sealed_unhandled::Unhandled {
                meta: ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(&err).clone(),
                source: err.into(),
            }),
        }
    }
}
impl From<crate::operation::batch_remove_role::BatchRemoveRoleError> for Error {
    fn from(err: crate::operation::batch_remove_role::BatchRemoveRoleError) -> Self {
        match err {
            crate::operation::batch_remove_role::BatchRemoveRoleError::AccessDeniedException(inner) => Error::AccessDeniedException(inner),
            crate::operation::batch_remove_role::BatchRemoveRoleError::InternalServerException(inner) => Error::InternalServerException(inner),
            crate::operation::batch_remove_role::BatchRemoveRoleError::ResourceNotFoundException(inner) => Error::ResourceNotFoundException(inner),
            crate::operation::batch_remove_role::BatchRemoveRoleError::ThrottlingException(inner) => Error::ThrottlingException(inner),
            crate::operation::batch_remove_role::BatchRemoveRoleError::ValidationException(inner) => Error::ValidationException(inner),
            crate::operation::batch_remove_role::BatchRemoveRoleError::Unhandled(inner) => Error::Unhandled(inner),
        }
    }
}
impl<R> From<::aws_smithy_runtime_api::client::result::SdkError<crate::operation::create_channel::CreateChannelError, R>> for Error
where
    R: Send + Sync + std::fmt::Debug + 'static,
{
    fn from(err: ::aws_smithy_runtime_api::client::result::SdkError<crate::operation::create_channel::CreateChannelError, R>) -> Self {
        match err {
            ::aws_smithy_runtime_api::client::result::SdkError::ServiceError(context) => Self::from(context.into_err()),
            _ => Error::Unhandled(crate::error::sealed_unhandled::Unhandled {
                meta: ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(&err).clone(),
                source: err.into(),
            }),
        }
    }
}
impl From<crate::operation::create_channel::CreateChannelError> for Error {
    fn from(err: crate::operation::create_channel::CreateChannelError) -> Self {
        match err {
            crate::operation::create_channel::CreateChannelError::AccessDeniedException(inner) => Error::AccessDeniedException(inner),
            crate::operation::create_channel::CreateChannelError::ConflictException(inner) => Error::ConflictException(inner),
            crate::operation::create_channel::CreateChannelError::InternalServerException(inner) => Error::InternalServerException(inner),
            crate::operation::create_channel::CreateChannelError::ResourceNotFoundException(inner) => Error::ResourceNotFoundException(inner),
            crate::operation::create_channel::CreateChannelError::ServiceQuotaExceededException(inner) => Error::ServiceQuotaExceededException(inner),
            crate::operation::create_channel::CreateChannelError::ThrottlingException(inner) => Error::ThrottlingException(inner),
            crate::operation::create_channel::CreateChannelError::ValidationException(inner) => Error::ValidationException(inner),
            crate::operation::create_channel::CreateChannelError::Unhandled(inner) => Error::Unhandled(inner),
        }
    }
}
impl<R> From<::aws_smithy_runtime_api::client::result::SdkError<crate::operation::create_space::CreateSpaceError, R>> for Error
where
    R: Send + Sync + std::fmt::Debug + 'static,
{
    fn from(err: ::aws_smithy_runtime_api::client::result::SdkError<crate::operation::create_space::CreateSpaceError, R>) -> Self {
        match err {
            ::aws_smithy_runtime_api::client::result::SdkError::ServiceError(context) => Self::from(context.into_err()),
            _ => Error::Unhandled(crate::error::sealed_unhandled::Unhandled {
                meta: ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(&err).clone(),
                source: err.into(),
            }),
        }
    }
}
impl From<crate::operation::create_space::CreateSpaceError> for Error {
    fn from(err: crate::operation::create_space::CreateSpaceError) -> Self {
        match err {
            crate::operation::create_space::CreateSpaceError::AccessDeniedException(inner) => Error::AccessDeniedException(inner),
            crate::operation::create_space::CreateSpaceError::ConflictException(inner) => Error::ConflictException(inner),
            crate::operation::create_space::CreateSpaceError::InternalServerException(inner) => Error::InternalServerException(inner),
            crate::operation::create_space::CreateSpaceError::ResourceNotFoundException(inner) => Error::ResourceNotFoundException(inner),
            crate::operation::create_space::CreateSpaceError::ServiceQuotaExceededException(inner) => Error::ServiceQuotaExceededException(inner),
            crate::operation::create_space::CreateSpaceError::ThrottlingException(inner) => Error::ThrottlingException(inner),
            crate::operation::create_space::CreateSpaceError::ValidationException(inner) => Error::ValidationException(inner),
            crate::operation::create_space::CreateSpaceError::Unhandled(inner) => Error::Unhandled(inner),
        }
    }
}
impl<R> From<::aws_smithy_runtime_api::client::result::SdkError<crate::operation::delete_space::DeleteSpaceError, R>> for Error
where
    R: Send + Sync + std::fmt::Debug + 'static,
{
    fn from(err: ::aws_smithy_runtime_api::client::result::SdkError<crate::operation::delete_space::DeleteSpaceError, R>) -> Self {
        match err {
            ::aws_smithy_runtime_api::client::result::SdkError::ServiceError(context) => Self::from(context.into_err()),
            _ => Error::Unhandled(crate::error::sealed_unhandled::Unhandled {
                meta: ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(&err).clone(),
                source: err.into(),
            }),
        }
    }
}
impl From<crate::operation::delete_space::DeleteSpaceError> for Error {
    fn from(err: crate::operation::delete_space::DeleteSpaceError) -> Self {
        match err {
            crate::operation::delete_space::DeleteSpaceError::AccessDeniedException(inner) => Error::AccessDeniedException(inner),
            crate::operation::delete_space::DeleteSpaceError::InternalServerException(inner) => Error::InternalServerException(inner),
            crate::operation::delete_space::DeleteSpaceError::ResourceNotFoundException(inner) => Error::ResourceNotFoundException(inner),
            crate::operation::delete_space::DeleteSpaceError::ThrottlingException(inner) => Error::ThrottlingException(inner),
            crate::operation::delete_space::DeleteSpaceError::ValidationException(inner) => Error::ValidationException(inner),
            crate::operation::delete_space::DeleteSpaceError::Unhandled(inner) => Error::Unhandled(inner),
        }
    }
}
impl<R> From<::aws_smithy_runtime_api::client::result::SdkError<crate::operation::deregister_admin::DeregisterAdminError, R>> for Error
where
    R: Send + Sync + std::fmt::Debug + 'static,
{
    fn from(err: ::aws_smithy_runtime_api::client::result::SdkError<crate::operation::deregister_admin::DeregisterAdminError, R>) -> Self {
        match err {
            ::aws_smithy_runtime_api::client::result::SdkError::ServiceError(context) => Self::from(context.into_err()),
            _ => Error::Unhandled(crate::error::sealed_unhandled::Unhandled {
                meta: ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(&err).clone(),
                source: err.into(),
            }),
        }
    }
}
impl From<crate::operation::deregister_admin::DeregisterAdminError> for Error {
    fn from(err: crate::operation::deregister_admin::DeregisterAdminError) -> Self {
        match err {
            crate::operation::deregister_admin::DeregisterAdminError::AccessDeniedException(inner) => Error::AccessDeniedException(inner),
            crate::operation::deregister_admin::DeregisterAdminError::InternalServerException(inner) => Error::InternalServerException(inner),
            crate::operation::deregister_admin::DeregisterAdminError::ResourceNotFoundException(inner) => Error::ResourceNotFoundException(inner),
            crate::operation::deregister_admin::DeregisterAdminError::ThrottlingException(inner) => Error::ThrottlingException(inner),
            crate::operation::deregister_admin::DeregisterAdminError::ValidationException(inner) => Error::ValidationException(inner),
            crate::operation::deregister_admin::DeregisterAdminError::Unhandled(inner) => Error::Unhandled(inner),
        }
    }
}
impl<R> From<::aws_smithy_runtime_api::client::result::SdkError<crate::operation::get_channel::GetChannelError, R>> for Error
where
    R: Send + Sync + std::fmt::Debug + 'static,
{
    fn from(err: ::aws_smithy_runtime_api::client::result::SdkError<crate::operation::get_channel::GetChannelError, R>) -> Self {
        match err {
            ::aws_smithy_runtime_api::client::result::SdkError::ServiceError(context) => Self::from(context.into_err()),
            _ => Error::Unhandled(crate::error::sealed_unhandled::Unhandled {
                meta: ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(&err).clone(),
                source: err.into(),
            }),
        }
    }
}
impl From<crate::operation::get_channel::GetChannelError> for Error {
    fn from(err: crate::operation::get_channel::GetChannelError) -> Self {
        match err {
            crate::operation::get_channel::GetChannelError::AccessDeniedException(inner) => Error::AccessDeniedException(inner),
            crate::operation::get_channel::GetChannelError::InternalServerException(inner) => Error::InternalServerException(inner),
            crate::operation::get_channel::GetChannelError::ResourceNotFoundException(inner) => Error::ResourceNotFoundException(inner),
            crate::operation::get_channel::GetChannelError::ThrottlingException(inner) => Error::ThrottlingException(inner),
            crate::operation::get_channel::GetChannelError::ValidationException(inner) => Error::ValidationException(inner),
            crate::operation::get_channel::GetChannelError::Unhandled(inner) => Error::Unhandled(inner),
        }
    }
}
impl<R> From<::aws_smithy_runtime_api::client::result::SdkError<crate::operation::get_space::GetSpaceError, R>> for Error
where
    R: Send + Sync + std::fmt::Debug + 'static,
{
    fn from(err: ::aws_smithy_runtime_api::client::result::SdkError<crate::operation::get_space::GetSpaceError, R>) -> Self {
        match err {
            ::aws_smithy_runtime_api::client::result::SdkError::ServiceError(context) => Self::from(context.into_err()),
            _ => Error::Unhandled(crate::error::sealed_unhandled::Unhandled {
                meta: ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(&err).clone(),
                source: err.into(),
            }),
        }
    }
}
impl From<crate::operation::get_space::GetSpaceError> for Error {
    fn from(err: crate::operation::get_space::GetSpaceError) -> Self {
        match err {
            crate::operation::get_space::GetSpaceError::AccessDeniedException(inner) => Error::AccessDeniedException(inner),
            crate::operation::get_space::GetSpaceError::InternalServerException(inner) => Error::InternalServerException(inner),
            crate::operation::get_space::GetSpaceError::ResourceNotFoundException(inner) => Error::ResourceNotFoundException(inner),
            crate::operation::get_space::GetSpaceError::ThrottlingException(inner) => Error::ThrottlingException(inner),
            crate::operation::get_space::GetSpaceError::ValidationException(inner) => Error::ValidationException(inner),
            crate::operation::get_space::GetSpaceError::Unhandled(inner) => Error::Unhandled(inner),
        }
    }
}
impl<R> From<::aws_smithy_runtime_api::client::result::SdkError<crate::operation::list_channels::ListChannelsError, R>> for Error
where
    R: Send + Sync + std::fmt::Debug + 'static,
{
    fn from(err: ::aws_smithy_runtime_api::client::result::SdkError<crate::operation::list_channels::ListChannelsError, R>) -> Self {
        match err {
            ::aws_smithy_runtime_api::client::result::SdkError::ServiceError(context) => Self::from(context.into_err()),
            _ => Error::Unhandled(crate::error::sealed_unhandled::Unhandled {
                meta: ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(&err).clone(),
                source: err.into(),
            }),
        }
    }
}
impl From<crate::operation::list_channels::ListChannelsError> for Error {
    fn from(err: crate::operation::list_channels::ListChannelsError) -> Self {
        match err {
            crate::operation::list_channels::ListChannelsError::AccessDeniedException(inner) => Error::AccessDeniedException(inner),
            crate::operation::list_channels::ListChannelsError::InternalServerException(inner) => Error::InternalServerException(inner),
            crate::operation::list_channels::ListChannelsError::ThrottlingException(inner) => Error::ThrottlingException(inner),
            crate::operation::list_channels::ListChannelsError::ValidationException(inner) => Error::ValidationException(inner),
            crate::operation::list_channels::ListChannelsError::Unhandled(inner) => Error::Unhandled(inner),
        }
    }
}
impl<R> From<::aws_smithy_runtime_api::client::result::SdkError<crate::operation::list_spaces::ListSpacesError, R>> for Error
where
    R: Send + Sync + std::fmt::Debug + 'static,
{
    fn from(err: ::aws_smithy_runtime_api::client::result::SdkError<crate::operation::list_spaces::ListSpacesError, R>) -> Self {
        match err {
            ::aws_smithy_runtime_api::client::result::SdkError::ServiceError(context) => Self::from(context.into_err()),
            _ => Error::Unhandled(crate::error::sealed_unhandled::Unhandled {
                meta: ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(&err).clone(),
                source: err.into(),
            }),
        }
    }
}
impl From<crate::operation::list_spaces::ListSpacesError> for Error {
    fn from(err: crate::operation::list_spaces::ListSpacesError) -> Self {
        match err {
            crate::operation::list_spaces::ListSpacesError::AccessDeniedException(inner) => Error::AccessDeniedException(inner),
            crate::operation::list_spaces::ListSpacesError::InternalServerException(inner) => Error::InternalServerException(inner),
            crate::operation::list_spaces::ListSpacesError::ThrottlingException(inner) => Error::ThrottlingException(inner),
            crate::operation::list_spaces::ListSpacesError::ValidationException(inner) => Error::ValidationException(inner),
            crate::operation::list_spaces::ListSpacesError::Unhandled(inner) => Error::Unhandled(inner),
        }
    }
}
impl<R> From<::aws_smithy_runtime_api::client::result::SdkError<crate::operation::list_tags_for_resource::ListTagsForResourceError, R>> for Error
where
    R: Send + Sync + std::fmt::Debug + 'static,
{
    fn from(err: ::aws_smithy_runtime_api::client::result::SdkError<crate::operation::list_tags_for_resource::ListTagsForResourceError, R>) -> Self {
        match err {
            ::aws_smithy_runtime_api::client::result::SdkError::ServiceError(context) => Self::from(context.into_err()),
            _ => Error::Unhandled(crate::error::sealed_unhandled::Unhandled {
                meta: ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(&err).clone(),
                source: err.into(),
            }),
        }
    }
}
impl From<crate::operation::list_tags_for_resource::ListTagsForResourceError> for Error {
    fn from(err: crate::operation::list_tags_for_resource::ListTagsForResourceError) -> Self {
        match err {
            crate::operation::list_tags_for_resource::ListTagsForResourceError::AccessDeniedException(inner) => Error::AccessDeniedException(inner),
            crate::operation::list_tags_for_resource::ListTagsForResourceError::InternalServerException(inner) => {
                Error::InternalServerException(inner)
            }
            crate::operation::list_tags_for_resource::ListTagsForResourceError::ResourceNotFoundException(inner) => {
                Error::ResourceNotFoundException(inner)
            }
            crate::operation::list_tags_for_resource::ListTagsForResourceError::ThrottlingException(inner) => Error::ThrottlingException(inner),
            crate::operation::list_tags_for_resource::ListTagsForResourceError::ValidationException(inner) => Error::ValidationException(inner),
            crate::operation::list_tags_for_resource::ListTagsForResourceError::Unhandled(inner) => Error::Unhandled(inner),
        }
    }
}
impl<R> From<::aws_smithy_runtime_api::client::result::SdkError<crate::operation::register_admin::RegisterAdminError, R>> for Error
where
    R: Send + Sync + std::fmt::Debug + 'static,
{
    fn from(err: ::aws_smithy_runtime_api::client::result::SdkError<crate::operation::register_admin::RegisterAdminError, R>) -> Self {
        match err {
            ::aws_smithy_runtime_api::client::result::SdkError::ServiceError(context) => Self::from(context.into_err()),
            _ => Error::Unhandled(crate::error::sealed_unhandled::Unhandled {
                meta: ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(&err).clone(),
                source: err.into(),
            }),
        }
    }
}
impl From<crate::operation::register_admin::RegisterAdminError> for Error {
    fn from(err: crate::operation::register_admin::RegisterAdminError) -> Self {
        match err {
            crate::operation::register_admin::RegisterAdminError::AccessDeniedException(inner) => Error::AccessDeniedException(inner),
            crate::operation::register_admin::RegisterAdminError::InternalServerException(inner) => Error::InternalServerException(inner),
            crate::operation::register_admin::RegisterAdminError::ResourceNotFoundException(inner) => Error::ResourceNotFoundException(inner),
            crate::operation::register_admin::RegisterAdminError::ThrottlingException(inner) => Error::ThrottlingException(inner),
            crate::operation::register_admin::RegisterAdminError::ValidationException(inner) => Error::ValidationException(inner),
            crate::operation::register_admin::RegisterAdminError::Unhandled(inner) => Error::Unhandled(inner),
        }
    }
}
impl<R> From<::aws_smithy_runtime_api::client::result::SdkError<crate::operation::send_invites::SendInvitesError, R>> for Error
where
    R: Send + Sync + std::fmt::Debug + 'static,
{
    fn from(err: ::aws_smithy_runtime_api::client::result::SdkError<crate::operation::send_invites::SendInvitesError, R>) -> Self {
        match err {
            ::aws_smithy_runtime_api::client::result::SdkError::ServiceError(context) => Self::from(context.into_err()),
            _ => Error::Unhandled(crate::error::sealed_unhandled::Unhandled {
                meta: ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(&err).clone(),
                source: err.into(),
            }),
        }
    }
}
impl From<crate::operation::send_invites::SendInvitesError> for Error {
    fn from(err: crate::operation::send_invites::SendInvitesError) -> Self {
        match err {
            crate::operation::send_invites::SendInvitesError::AccessDeniedException(inner) => Error::AccessDeniedException(inner),
            crate::operation::send_invites::SendInvitesError::InternalServerException(inner) => Error::InternalServerException(inner),
            crate::operation::send_invites::SendInvitesError::ResourceNotFoundException(inner) => Error::ResourceNotFoundException(inner),
            crate::operation::send_invites::SendInvitesError::ThrottlingException(inner) => Error::ThrottlingException(inner),
            crate::operation::send_invites::SendInvitesError::ValidationException(inner) => Error::ValidationException(inner),
            crate::operation::send_invites::SendInvitesError::Unhandled(inner) => Error::Unhandled(inner),
        }
    }
}
impl<R> From<::aws_smithy_runtime_api::client::result::SdkError<crate::operation::tag_resource::TagResourceError, R>> for Error
where
    R: Send + Sync + std::fmt::Debug + 'static,
{
    fn from(err: ::aws_smithy_runtime_api::client::result::SdkError<crate::operation::tag_resource::TagResourceError, R>) -> Self {
        match err {
            ::aws_smithy_runtime_api::client::result::SdkError::ServiceError(context) => Self::from(context.into_err()),
            _ => Error::Unhandled(crate::error::sealed_unhandled::Unhandled {
                meta: ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(&err).clone(),
                source: err.into(),
            }),
        }
    }
}
impl From<crate::operation::tag_resource::TagResourceError> for Error {
    fn from(err: crate::operation::tag_resource::TagResourceError) -> Self {
        match err {
            crate::operation::tag_resource::TagResourceError::AccessDeniedException(inner) => Error::AccessDeniedException(inner),
            crate::operation::tag_resource::TagResourceError::InternalServerException(inner) => Error::InternalServerException(inner),
            crate::operation::tag_resource::TagResourceError::ResourceNotFoundException(inner) => Error::ResourceNotFoundException(inner),
            crate::operation::tag_resource::TagResourceError::ThrottlingException(inner) => Error::ThrottlingException(inner),
            crate::operation::tag_resource::TagResourceError::ValidationException(inner) => Error::ValidationException(inner),
            crate::operation::tag_resource::TagResourceError::Unhandled(inner) => Error::Unhandled(inner),
        }
    }
}
impl<R> From<::aws_smithy_runtime_api::client::result::SdkError<crate::operation::untag_resource::UntagResourceError, R>> for Error
where
    R: Send + Sync + std::fmt::Debug + 'static,
{
    fn from(err: ::aws_smithy_runtime_api::client::result::SdkError<crate::operation::untag_resource::UntagResourceError, R>) -> Self {
        match err {
            ::aws_smithy_runtime_api::client::result::SdkError::ServiceError(context) => Self::from(context.into_err()),
            _ => Error::Unhandled(crate::error::sealed_unhandled::Unhandled {
                meta: ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(&err).clone(),
                source: err.into(),
            }),
        }
    }
}
impl From<crate::operation::untag_resource::UntagResourceError> for Error {
    fn from(err: crate::operation::untag_resource::UntagResourceError) -> Self {
        match err {
            crate::operation::untag_resource::UntagResourceError::AccessDeniedException(inner) => Error::AccessDeniedException(inner),
            crate::operation::untag_resource::UntagResourceError::InternalServerException(inner) => Error::InternalServerException(inner),
            crate::operation::untag_resource::UntagResourceError::ResourceNotFoundException(inner) => Error::ResourceNotFoundException(inner),
            crate::operation::untag_resource::UntagResourceError::ThrottlingException(inner) => Error::ThrottlingException(inner),
            crate::operation::untag_resource::UntagResourceError::ValidationException(inner) => Error::ValidationException(inner),
            crate::operation::untag_resource::UntagResourceError::Unhandled(inner) => Error::Unhandled(inner),
        }
    }
}
impl<R> From<::aws_smithy_runtime_api::client::result::SdkError<crate::operation::update_channel::UpdateChannelError, R>> for Error
where
    R: Send + Sync + std::fmt::Debug + 'static,
{
    fn from(err: ::aws_smithy_runtime_api::client::result::SdkError<crate::operation::update_channel::UpdateChannelError, R>) -> Self {
        match err {
            ::aws_smithy_runtime_api::client::result::SdkError::ServiceError(context) => Self::from(context.into_err()),
            _ => Error::Unhandled(crate::error::sealed_unhandled::Unhandled {
                meta: ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(&err).clone(),
                source: err.into(),
            }),
        }
    }
}
impl From<crate::operation::update_channel::UpdateChannelError> for Error {
    fn from(err: crate::operation::update_channel::UpdateChannelError) -> Self {
        match err {
            crate::operation::update_channel::UpdateChannelError::AccessDeniedException(inner) => Error::AccessDeniedException(inner),
            crate::operation::update_channel::UpdateChannelError::ConflictException(inner) => Error::ConflictException(inner),
            crate::operation::update_channel::UpdateChannelError::InternalServerException(inner) => Error::InternalServerException(inner),
            crate::operation::update_channel::UpdateChannelError::ResourceNotFoundException(inner) => Error::ResourceNotFoundException(inner),
            crate::operation::update_channel::UpdateChannelError::ThrottlingException(inner) => Error::ThrottlingException(inner),
            crate::operation::update_channel::UpdateChannelError::ValidationException(inner) => Error::ValidationException(inner),
            crate::operation::update_channel::UpdateChannelError::Unhandled(inner) => Error::Unhandled(inner),
        }
    }
}
impl<R> From<::aws_smithy_runtime_api::client::result::SdkError<crate::operation::update_space::UpdateSpaceError, R>> for Error
where
    R: Send + Sync + std::fmt::Debug + 'static,
{
    fn from(err: ::aws_smithy_runtime_api::client::result::SdkError<crate::operation::update_space::UpdateSpaceError, R>) -> Self {
        match err {
            ::aws_smithy_runtime_api::client::result::SdkError::ServiceError(context) => Self::from(context.into_err()),
            _ => Error::Unhandled(crate::error::sealed_unhandled::Unhandled {
                meta: ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(&err).clone(),
                source: err.into(),
            }),
        }
    }
}
impl From<crate::operation::update_space::UpdateSpaceError> for Error {
    fn from(err: crate::operation::update_space::UpdateSpaceError) -> Self {
        match err {
            crate::operation::update_space::UpdateSpaceError::AccessDeniedException(inner) => Error::AccessDeniedException(inner),
            crate::operation::update_space::UpdateSpaceError::ConflictException(inner) => Error::ConflictException(inner),
            crate::operation::update_space::UpdateSpaceError::InternalServerException(inner) => Error::InternalServerException(inner),
            crate::operation::update_space::UpdateSpaceError::ResourceNotFoundException(inner) => Error::ResourceNotFoundException(inner),
            crate::operation::update_space::UpdateSpaceError::ThrottlingException(inner) => Error::ThrottlingException(inner),
            crate::operation::update_space::UpdateSpaceError::ValidationException(inner) => Error::ValidationException(inner),
            crate::operation::update_space::UpdateSpaceError::Unhandled(inner) => Error::Unhandled(inner),
        }
    }
}
impl<O, E> ::std::convert::From<::aws_smithy_runtime_api::client::waiters::error::WaiterError<O, E>> for Error
where
    O: ::std::fmt::Debug + ::std::marker::Send + ::std::marker::Sync + 'static,
    E: ::std::error::Error + ::std::marker::Send + ::std::marker::Sync + 'static,
{
    fn from(err: ::aws_smithy_runtime_api::client::waiters::error::WaiterError<O, E>) -> Self {
        Error::Unhandled(crate::error::sealed_unhandled::Unhandled {
            meta: ::std::default::Default::default(),
            source: err.into(),
        })
    }
}
impl ::std::error::Error for Error {
    fn source(&self) -> std::option::Option<&(dyn ::std::error::Error + 'static)> {
        match self {
            Error::AccessDeniedException(inner) => inner.source(),
            Error::ConflictException(inner) => inner.source(),
            Error::InternalServerException(inner) => inner.source(),
            Error::ResourceNotFoundException(inner) => inner.source(),
            Error::ServiceQuotaExceededException(inner) => inner.source(),
            Error::ThrottlingException(inner) => inner.source(),
            Error::ValidationException(inner) => inner.source(),
            Error::Unhandled(inner) => ::std::option::Option::Some(&*inner.source),
        }
    }
}
impl ::aws_types::request_id::RequestId for Error {
    fn request_id(&self) -> Option<&str> {
        match self {
            Self::AccessDeniedException(e) => e.request_id(),
            Self::ConflictException(e) => e.request_id(),
            Self::InternalServerException(e) => e.request_id(),
            Self::ResourceNotFoundException(e) => e.request_id(),
            Self::ServiceQuotaExceededException(e) => e.request_id(),
            Self::ThrottlingException(e) => e.request_id(),
            Self::ValidationException(e) => e.request_id(),
            Self::Unhandled(e) => e.meta.request_id(),
        }
    }
}
