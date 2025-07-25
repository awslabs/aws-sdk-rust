// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.

/// <p>Returns properties of the user that you specify.</p>
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::fmt::Debug)]
pub struct ListedUser {
    /// <p>Provides the unique Amazon Resource Name (ARN) for the user that you want to learn about.</p>
    pub arn: ::std::string::String,
    /// <p>The landing directory (folder) for a user when they log in to the server using the client.</p>
    /// <p>A <code>HomeDirectory</code> example is <code>/bucket_name/home/mydirectory</code>.</p><note>
    /// <p>You can use the <code>HomeDirectory</code> parameter for <code>HomeDirectoryType</code> when it is set to either <code>PATH</code> or <code>LOGICAL</code>.</p>
    /// </note>
    pub home_directory: ::std::option::Option<::std::string::String>,
    /// <p>The type of landing directory (folder) that you want your users' home directory to be when they log in to the server. If you set it to <code>PATH</code>, the user will see the absolute Amazon S3 bucket or Amazon EFS path as is in their file transfer protocol clients. If you set it to <code>LOGICAL</code>, you need to provide mappings in the <code>HomeDirectoryMappings</code> for how you want to make Amazon S3 or Amazon EFS paths visible to your users.</p><note>
    /// <p>If <code>HomeDirectoryType</code> is <code>LOGICAL</code>, you must provide mappings, using the <code>HomeDirectoryMappings</code> parameter. If, on the other hand, <code>HomeDirectoryType</code> is <code>PATH</code>, you provide an absolute path using the <code>HomeDirectory</code> parameter. You cannot have both <code>HomeDirectory</code> and <code>HomeDirectoryMappings</code> in your template.</p>
    /// </note>
    pub home_directory_type: ::std::option::Option<crate::types::HomeDirectoryType>,
    /// <p>The Amazon Resource Name (ARN) of the Identity and Access Management (IAM) role that controls your users' access to your Amazon S3 bucket or Amazon EFS file system. The policies attached to this role determine the level of access that you want to provide your users when transferring files into and out of your Amazon S3 bucket or Amazon EFS file system. The IAM role should also contain a trust relationship that allows the server to access your resources when servicing your users' transfer requests.</p><note>
    /// <p>The IAM role that controls your users' access to your Amazon S3 bucket for servers with <code>Domain=S3</code>, or your EFS file system for servers with <code>Domain=EFS</code>.</p>
    /// <p>The policies attached to this role determine the level of access you want to provide your users when transferring files into and out of your S3 buckets or EFS file systems.</p>
    /// </note>
    pub role: ::std::option::Option<::std::string::String>,
    /// <p>Specifies the number of SSH public keys stored for the user you specified.</p>
    pub ssh_public_key_count: ::std::option::Option<i32>,
    /// <p>Specifies the name of the user whose ARN was specified. User names are used for authentication purposes.</p>
    pub user_name: ::std::option::Option<::std::string::String>,
}
impl ListedUser {
    /// <p>Provides the unique Amazon Resource Name (ARN) for the user that you want to learn about.</p>
    pub fn arn(&self) -> &str {
        use std::ops::Deref;
        self.arn.deref()
    }
    /// <p>The landing directory (folder) for a user when they log in to the server using the client.</p>
    /// <p>A <code>HomeDirectory</code> example is <code>/bucket_name/home/mydirectory</code>.</p><note>
    /// <p>You can use the <code>HomeDirectory</code> parameter for <code>HomeDirectoryType</code> when it is set to either <code>PATH</code> or <code>LOGICAL</code>.</p>
    /// </note>
    pub fn home_directory(&self) -> ::std::option::Option<&str> {
        self.home_directory.as_deref()
    }
    /// <p>The type of landing directory (folder) that you want your users' home directory to be when they log in to the server. If you set it to <code>PATH</code>, the user will see the absolute Amazon S3 bucket or Amazon EFS path as is in their file transfer protocol clients. If you set it to <code>LOGICAL</code>, you need to provide mappings in the <code>HomeDirectoryMappings</code> for how you want to make Amazon S3 or Amazon EFS paths visible to your users.</p><note>
    /// <p>If <code>HomeDirectoryType</code> is <code>LOGICAL</code>, you must provide mappings, using the <code>HomeDirectoryMappings</code> parameter. If, on the other hand, <code>HomeDirectoryType</code> is <code>PATH</code>, you provide an absolute path using the <code>HomeDirectory</code> parameter. You cannot have both <code>HomeDirectory</code> and <code>HomeDirectoryMappings</code> in your template.</p>
    /// </note>
    pub fn home_directory_type(&self) -> ::std::option::Option<&crate::types::HomeDirectoryType> {
        self.home_directory_type.as_ref()
    }
    /// <p>The Amazon Resource Name (ARN) of the Identity and Access Management (IAM) role that controls your users' access to your Amazon S3 bucket or Amazon EFS file system. The policies attached to this role determine the level of access that you want to provide your users when transferring files into and out of your Amazon S3 bucket or Amazon EFS file system. The IAM role should also contain a trust relationship that allows the server to access your resources when servicing your users' transfer requests.</p><note>
    /// <p>The IAM role that controls your users' access to your Amazon S3 bucket for servers with <code>Domain=S3</code>, or your EFS file system for servers with <code>Domain=EFS</code>.</p>
    /// <p>The policies attached to this role determine the level of access you want to provide your users when transferring files into and out of your S3 buckets or EFS file systems.</p>
    /// </note>
    pub fn role(&self) -> ::std::option::Option<&str> {
        self.role.as_deref()
    }
    /// <p>Specifies the number of SSH public keys stored for the user you specified.</p>
    pub fn ssh_public_key_count(&self) -> ::std::option::Option<i32> {
        self.ssh_public_key_count
    }
    /// <p>Specifies the name of the user whose ARN was specified. User names are used for authentication purposes.</p>
    pub fn user_name(&self) -> ::std::option::Option<&str> {
        self.user_name.as_deref()
    }
}
impl ListedUser {
    /// Creates a new builder-style object to manufacture [`ListedUser`](crate::types::ListedUser).
    pub fn builder() -> crate::types::builders::ListedUserBuilder {
        crate::types::builders::ListedUserBuilder::default()
    }
}

/// A builder for [`ListedUser`](crate::types::ListedUser).
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::default::Default, ::std::fmt::Debug)]
#[non_exhaustive]
pub struct ListedUserBuilder {
    pub(crate) arn: ::std::option::Option<::std::string::String>,
    pub(crate) home_directory: ::std::option::Option<::std::string::String>,
    pub(crate) home_directory_type: ::std::option::Option<crate::types::HomeDirectoryType>,
    pub(crate) role: ::std::option::Option<::std::string::String>,
    pub(crate) ssh_public_key_count: ::std::option::Option<i32>,
    pub(crate) user_name: ::std::option::Option<::std::string::String>,
}
impl ListedUserBuilder {
    /// <p>Provides the unique Amazon Resource Name (ARN) for the user that you want to learn about.</p>
    /// This field is required.
    pub fn arn(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.arn = ::std::option::Option::Some(input.into());
        self
    }
    /// <p>Provides the unique Amazon Resource Name (ARN) for the user that you want to learn about.</p>
    pub fn set_arn(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.arn = input;
        self
    }
    /// <p>Provides the unique Amazon Resource Name (ARN) for the user that you want to learn about.</p>
    pub fn get_arn(&self) -> &::std::option::Option<::std::string::String> {
        &self.arn
    }
    /// <p>The landing directory (folder) for a user when they log in to the server using the client.</p>
    /// <p>A <code>HomeDirectory</code> example is <code>/bucket_name/home/mydirectory</code>.</p><note>
    /// <p>You can use the <code>HomeDirectory</code> parameter for <code>HomeDirectoryType</code> when it is set to either <code>PATH</code> or <code>LOGICAL</code>.</p>
    /// </note>
    pub fn home_directory(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.home_directory = ::std::option::Option::Some(input.into());
        self
    }
    /// <p>The landing directory (folder) for a user when they log in to the server using the client.</p>
    /// <p>A <code>HomeDirectory</code> example is <code>/bucket_name/home/mydirectory</code>.</p><note>
    /// <p>You can use the <code>HomeDirectory</code> parameter for <code>HomeDirectoryType</code> when it is set to either <code>PATH</code> or <code>LOGICAL</code>.</p>
    /// </note>
    pub fn set_home_directory(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.home_directory = input;
        self
    }
    /// <p>The landing directory (folder) for a user when they log in to the server using the client.</p>
    /// <p>A <code>HomeDirectory</code> example is <code>/bucket_name/home/mydirectory</code>.</p><note>
    /// <p>You can use the <code>HomeDirectory</code> parameter for <code>HomeDirectoryType</code> when it is set to either <code>PATH</code> or <code>LOGICAL</code>.</p>
    /// </note>
    pub fn get_home_directory(&self) -> &::std::option::Option<::std::string::String> {
        &self.home_directory
    }
    /// <p>The type of landing directory (folder) that you want your users' home directory to be when they log in to the server. If you set it to <code>PATH</code>, the user will see the absolute Amazon S3 bucket or Amazon EFS path as is in their file transfer protocol clients. If you set it to <code>LOGICAL</code>, you need to provide mappings in the <code>HomeDirectoryMappings</code> for how you want to make Amazon S3 or Amazon EFS paths visible to your users.</p><note>
    /// <p>If <code>HomeDirectoryType</code> is <code>LOGICAL</code>, you must provide mappings, using the <code>HomeDirectoryMappings</code> parameter. If, on the other hand, <code>HomeDirectoryType</code> is <code>PATH</code>, you provide an absolute path using the <code>HomeDirectory</code> parameter. You cannot have both <code>HomeDirectory</code> and <code>HomeDirectoryMappings</code> in your template.</p>
    /// </note>
    pub fn home_directory_type(mut self, input: crate::types::HomeDirectoryType) -> Self {
        self.home_directory_type = ::std::option::Option::Some(input);
        self
    }
    /// <p>The type of landing directory (folder) that you want your users' home directory to be when they log in to the server. If you set it to <code>PATH</code>, the user will see the absolute Amazon S3 bucket or Amazon EFS path as is in their file transfer protocol clients. If you set it to <code>LOGICAL</code>, you need to provide mappings in the <code>HomeDirectoryMappings</code> for how you want to make Amazon S3 or Amazon EFS paths visible to your users.</p><note>
    /// <p>If <code>HomeDirectoryType</code> is <code>LOGICAL</code>, you must provide mappings, using the <code>HomeDirectoryMappings</code> parameter. If, on the other hand, <code>HomeDirectoryType</code> is <code>PATH</code>, you provide an absolute path using the <code>HomeDirectory</code> parameter. You cannot have both <code>HomeDirectory</code> and <code>HomeDirectoryMappings</code> in your template.</p>
    /// </note>
    pub fn set_home_directory_type(mut self, input: ::std::option::Option<crate::types::HomeDirectoryType>) -> Self {
        self.home_directory_type = input;
        self
    }
    /// <p>The type of landing directory (folder) that you want your users' home directory to be when they log in to the server. If you set it to <code>PATH</code>, the user will see the absolute Amazon S3 bucket or Amazon EFS path as is in their file transfer protocol clients. If you set it to <code>LOGICAL</code>, you need to provide mappings in the <code>HomeDirectoryMappings</code> for how you want to make Amazon S3 or Amazon EFS paths visible to your users.</p><note>
    /// <p>If <code>HomeDirectoryType</code> is <code>LOGICAL</code>, you must provide mappings, using the <code>HomeDirectoryMappings</code> parameter. If, on the other hand, <code>HomeDirectoryType</code> is <code>PATH</code>, you provide an absolute path using the <code>HomeDirectory</code> parameter. You cannot have both <code>HomeDirectory</code> and <code>HomeDirectoryMappings</code> in your template.</p>
    /// </note>
    pub fn get_home_directory_type(&self) -> &::std::option::Option<crate::types::HomeDirectoryType> {
        &self.home_directory_type
    }
    /// <p>The Amazon Resource Name (ARN) of the Identity and Access Management (IAM) role that controls your users' access to your Amazon S3 bucket or Amazon EFS file system. The policies attached to this role determine the level of access that you want to provide your users when transferring files into and out of your Amazon S3 bucket or Amazon EFS file system. The IAM role should also contain a trust relationship that allows the server to access your resources when servicing your users' transfer requests.</p><note>
    /// <p>The IAM role that controls your users' access to your Amazon S3 bucket for servers with <code>Domain=S3</code>, or your EFS file system for servers with <code>Domain=EFS</code>.</p>
    /// <p>The policies attached to this role determine the level of access you want to provide your users when transferring files into and out of your S3 buckets or EFS file systems.</p>
    /// </note>
    pub fn role(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.role = ::std::option::Option::Some(input.into());
        self
    }
    /// <p>The Amazon Resource Name (ARN) of the Identity and Access Management (IAM) role that controls your users' access to your Amazon S3 bucket or Amazon EFS file system. The policies attached to this role determine the level of access that you want to provide your users when transferring files into and out of your Amazon S3 bucket or Amazon EFS file system. The IAM role should also contain a trust relationship that allows the server to access your resources when servicing your users' transfer requests.</p><note>
    /// <p>The IAM role that controls your users' access to your Amazon S3 bucket for servers with <code>Domain=S3</code>, or your EFS file system for servers with <code>Domain=EFS</code>.</p>
    /// <p>The policies attached to this role determine the level of access you want to provide your users when transferring files into and out of your S3 buckets or EFS file systems.</p>
    /// </note>
    pub fn set_role(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.role = input;
        self
    }
    /// <p>The Amazon Resource Name (ARN) of the Identity and Access Management (IAM) role that controls your users' access to your Amazon S3 bucket or Amazon EFS file system. The policies attached to this role determine the level of access that you want to provide your users when transferring files into and out of your Amazon S3 bucket or Amazon EFS file system. The IAM role should also contain a trust relationship that allows the server to access your resources when servicing your users' transfer requests.</p><note>
    /// <p>The IAM role that controls your users' access to your Amazon S3 bucket for servers with <code>Domain=S3</code>, or your EFS file system for servers with <code>Domain=EFS</code>.</p>
    /// <p>The policies attached to this role determine the level of access you want to provide your users when transferring files into and out of your S3 buckets or EFS file systems.</p>
    /// </note>
    pub fn get_role(&self) -> &::std::option::Option<::std::string::String> {
        &self.role
    }
    /// <p>Specifies the number of SSH public keys stored for the user you specified.</p>
    pub fn ssh_public_key_count(mut self, input: i32) -> Self {
        self.ssh_public_key_count = ::std::option::Option::Some(input);
        self
    }
    /// <p>Specifies the number of SSH public keys stored for the user you specified.</p>
    pub fn set_ssh_public_key_count(mut self, input: ::std::option::Option<i32>) -> Self {
        self.ssh_public_key_count = input;
        self
    }
    /// <p>Specifies the number of SSH public keys stored for the user you specified.</p>
    pub fn get_ssh_public_key_count(&self) -> &::std::option::Option<i32> {
        &self.ssh_public_key_count
    }
    /// <p>Specifies the name of the user whose ARN was specified. User names are used for authentication purposes.</p>
    pub fn user_name(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.user_name = ::std::option::Option::Some(input.into());
        self
    }
    /// <p>Specifies the name of the user whose ARN was specified. User names are used for authentication purposes.</p>
    pub fn set_user_name(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.user_name = input;
        self
    }
    /// <p>Specifies the name of the user whose ARN was specified. User names are used for authentication purposes.</p>
    pub fn get_user_name(&self) -> &::std::option::Option<::std::string::String> {
        &self.user_name
    }
    /// Consumes the builder and constructs a [`ListedUser`](crate::types::ListedUser).
    /// This method will fail if any of the following fields are not set:
    /// - [`arn`](crate::types::builders::ListedUserBuilder::arn)
    pub fn build(self) -> ::std::result::Result<crate::types::ListedUser, ::aws_smithy_types::error::operation::BuildError> {
        ::std::result::Result::Ok(crate::types::ListedUser {
            arn: self.arn.ok_or_else(|| {
                ::aws_smithy_types::error::operation::BuildError::missing_field(
                    "arn",
                    "arn was not specified but it is required when building ListedUser",
                )
            })?,
            home_directory: self.home_directory,
            home_directory_type: self.home_directory_type,
            role: self.role,
            ssh_public_key_count: self.ssh_public_key_count,
            user_name: self.user_name,
        })
    }
}
