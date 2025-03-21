// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.

/// Matcher union: {"output":{"path":"ProjectVersionDescriptions[].Status","expected":"RUNNING","comparator":"allStringEquals"}}
pub(crate) fn match_describe_project_versions_ecdc1939edd26defa(
    _result: ::std::result::Result<
        &crate::operation::describe_project_versions::DescribeProjectVersionsOutput,
        &crate::operation::describe_project_versions::DescribeProjectVersionsError,
    >,
) -> bool {
    fn path_traversal<'a>(
        _output: &'a crate::operation::describe_project_versions::DescribeProjectVersionsOutput,
    ) -> ::std::option::Option<::std::vec::Vec<&'a crate::types::ProjectVersionStatus>> {
        let _fld_1 = _output.project_version_descriptions.as_ref()?;
        let _prj_3 = _fld_1
            .iter()
            .flat_map(|v| {
                #[allow(clippy::let_and_return)]
                fn map(_v: &crate::types::ProjectVersionDescription) -> ::std::option::Option<&crate::types::ProjectVersionStatus> {
                    let _fld_2 = _v.status.as_ref();
                    _fld_2
                }
                map(v)
            })
            .collect::<::std::vec::Vec<_>>();
        ::std::option::Option::Some(_prj_3)
    }
    _result
        .as_ref()
        .ok()
        .and_then(|output| path_traversal(output))
        .map(|value| {
            !value.is_empty()
                && value.iter().all(|value| {
                    let _tmp_2 = value.as_str();
                    let right = "RUNNING";
                    let _cmp_1 = _tmp_2 == right;
                    _cmp_1
                })
        })
        .unwrap_or_default()
}

/// Matcher union: {"output":{"path":"ProjectVersionDescriptions[].Status","expected":"FAILED","comparator":"anyStringEquals"}}
pub(crate) fn match_describe_project_versions_372e6b6443f871a1d(
    _result: ::std::result::Result<
        &crate::operation::describe_project_versions::DescribeProjectVersionsOutput,
        &crate::operation::describe_project_versions::DescribeProjectVersionsError,
    >,
) -> bool {
    fn path_traversal<'a>(
        _output: &'a crate::operation::describe_project_versions::DescribeProjectVersionsOutput,
    ) -> ::std::option::Option<::std::vec::Vec<&'a crate::types::ProjectVersionStatus>> {
        let _fld_1 = _output.project_version_descriptions.as_ref()?;
        let _prj_3 = _fld_1
            .iter()
            .flat_map(|v| {
                #[allow(clippy::let_and_return)]
                fn map(_v: &crate::types::ProjectVersionDescription) -> ::std::option::Option<&crate::types::ProjectVersionStatus> {
                    let _fld_2 = _v.status.as_ref();
                    _fld_2
                }
                map(v)
            })
            .collect::<::std::vec::Vec<_>>();
        ::std::option::Option::Some(_prj_3)
    }
    _result
        .as_ref()
        .ok()
        .and_then(|output| path_traversal(output))
        .map(|value| {
            value.iter().any(|value| {
                let _tmp_2 = value.as_str();
                let right = "FAILED";
                let _cmp_1 = _tmp_2 == right;
                _cmp_1
            })
        })
        .unwrap_or_default()
}

/// Matcher union: {"output":{"path":"ProjectVersionDescriptions[].Status","expected":"TRAINING_COMPLETED","comparator":"allStringEquals"}}
pub(crate) fn match_describe_project_versions_c0de0dba73a4550df(
    _result: ::std::result::Result<
        &crate::operation::describe_project_versions::DescribeProjectVersionsOutput,
        &crate::operation::describe_project_versions::DescribeProjectVersionsError,
    >,
) -> bool {
    fn path_traversal<'a>(
        _output: &'a crate::operation::describe_project_versions::DescribeProjectVersionsOutput,
    ) -> ::std::option::Option<::std::vec::Vec<&'a crate::types::ProjectVersionStatus>> {
        let _fld_1 = _output.project_version_descriptions.as_ref()?;
        let _prj_3 = _fld_1
            .iter()
            .flat_map(|v| {
                #[allow(clippy::let_and_return)]
                fn map(_v: &crate::types::ProjectVersionDescription) -> ::std::option::Option<&crate::types::ProjectVersionStatus> {
                    let _fld_2 = _v.status.as_ref();
                    _fld_2
                }
                map(v)
            })
            .collect::<::std::vec::Vec<_>>();
        ::std::option::Option::Some(_prj_3)
    }
    _result
        .as_ref()
        .ok()
        .and_then(|output| path_traversal(output))
        .map(|value| {
            !value.is_empty()
                && value.iter().all(|value| {
                    let _tmp_2 = value.as_str();
                    let right = "TRAINING_COMPLETED";
                    let _cmp_1 = _tmp_2 == right;
                    _cmp_1
                })
        })
        .unwrap_or_default()
}

/// Matcher union: {"output":{"path":"ProjectVersionDescriptions[].Status","expected":"TRAINING_FAILED","comparator":"anyStringEquals"}}
pub(crate) fn match_describe_project_versions_d574c78b065cace74(
    _result: ::std::result::Result<
        &crate::operation::describe_project_versions::DescribeProjectVersionsOutput,
        &crate::operation::describe_project_versions::DescribeProjectVersionsError,
    >,
) -> bool {
    fn path_traversal<'a>(
        _output: &'a crate::operation::describe_project_versions::DescribeProjectVersionsOutput,
    ) -> ::std::option::Option<::std::vec::Vec<&'a crate::types::ProjectVersionStatus>> {
        let _fld_1 = _output.project_version_descriptions.as_ref()?;
        let _prj_3 = _fld_1
            .iter()
            .flat_map(|v| {
                #[allow(clippy::let_and_return)]
                fn map(_v: &crate::types::ProjectVersionDescription) -> ::std::option::Option<&crate::types::ProjectVersionStatus> {
                    let _fld_2 = _v.status.as_ref();
                    _fld_2
                }
                map(v)
            })
            .collect::<::std::vec::Vec<_>>();
        ::std::option::Option::Some(_prj_3)
    }
    _result
        .as_ref()
        .ok()
        .and_then(|output| path_traversal(output))
        .map(|value| {
            value.iter().any(|value| {
                let _tmp_2 = value.as_str();
                let right = "TRAINING_FAILED";
                let _cmp_1 = _tmp_2 == right;
                _cmp_1
            })
        })
        .unwrap_or_default()
}
