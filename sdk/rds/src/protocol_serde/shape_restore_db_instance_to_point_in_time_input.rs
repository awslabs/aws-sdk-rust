// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_restore_db_instance_to_point_in_time_input_input_input(
    input: &crate::operation::restore_db_instance_to_point_in_time::RestoreDbInstanceToPointInTimeInput,
) -> ::std::result::Result<::aws_smithy_types::body::SdkBody, ::aws_smithy_types::error::operation::SerializationError> {
    let mut out = String::new();
    #[allow(unused_mut)]
    let mut writer = ::aws_smithy_query::QueryWriter::new(&mut out, "RestoreDBInstanceToPointInTime", "2014-10-31");
    #[allow(unused_mut)]
    let mut scope_1 = writer.prefix("SourceDBInstanceIdentifier");
    if let Some(var_2) = &input.source_db_instance_identifier {
        scope_1.string(var_2);
    }
    #[allow(unused_mut)]
    let mut scope_3 = writer.prefix("TargetDBInstanceIdentifier");
    if let Some(var_4) = &input.target_db_instance_identifier {
        scope_3.string(var_4);
    }
    #[allow(unused_mut)]
    let mut scope_5 = writer.prefix("RestoreTime");
    if let Some(var_6) = &input.restore_time {
        scope_5.date_time(var_6, ::aws_smithy_types::date_time::Format::DateTime)?;
    }
    #[allow(unused_mut)]
    let mut scope_7 = writer.prefix("UseLatestRestorableTime");
    if let Some(var_8) = &input.use_latest_restorable_time {
        scope_7.boolean(*var_8);
    }
    #[allow(unused_mut)]
    let mut scope_9 = writer.prefix("DBInstanceClass");
    if let Some(var_10) = &input.db_instance_class {
        scope_9.string(var_10);
    }
    #[allow(unused_mut)]
    let mut scope_11 = writer.prefix("Port");
    if let Some(var_12) = &input.port {
        scope_11.number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_12).into()),
        );
    }
    #[allow(unused_mut)]
    let mut scope_13 = writer.prefix("AvailabilityZone");
    if let Some(var_14) = &input.availability_zone {
        scope_13.string(var_14);
    }
    #[allow(unused_mut)]
    let mut scope_15 = writer.prefix("DBSubnetGroupName");
    if let Some(var_16) = &input.db_subnet_group_name {
        scope_15.string(var_16);
    }
    #[allow(unused_mut)]
    let mut scope_17 = writer.prefix("MultiAZ");
    if let Some(var_18) = &input.multi_az {
        scope_17.boolean(*var_18);
    }
    #[allow(unused_mut)]
    let mut scope_19 = writer.prefix("PubliclyAccessible");
    if let Some(var_20) = &input.publicly_accessible {
        scope_19.boolean(*var_20);
    }
    #[allow(unused_mut)]
    let mut scope_21 = writer.prefix("AutoMinorVersionUpgrade");
    if let Some(var_22) = &input.auto_minor_version_upgrade {
        scope_21.boolean(*var_22);
    }
    #[allow(unused_mut)]
    let mut scope_23 = writer.prefix("LicenseModel");
    if let Some(var_24) = &input.license_model {
        scope_23.string(var_24);
    }
    #[allow(unused_mut)]
    let mut scope_25 = writer.prefix("DBName");
    if let Some(var_26) = &input.db_name {
        scope_25.string(var_26);
    }
    #[allow(unused_mut)]
    let mut scope_27 = writer.prefix("Engine");
    if let Some(var_28) = &input.engine {
        scope_27.string(var_28);
    }
    #[allow(unused_mut)]
    let mut scope_29 = writer.prefix("Iops");
    if let Some(var_30) = &input.iops {
        scope_29.number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_30).into()),
        );
    }
    #[allow(unused_mut)]
    let mut scope_31 = writer.prefix("OptionGroupName");
    if let Some(var_32) = &input.option_group_name {
        scope_31.string(var_32);
    }
    #[allow(unused_mut)]
    let mut scope_33 = writer.prefix("CopyTagsToSnapshot");
    if let Some(var_34) = &input.copy_tags_to_snapshot {
        scope_33.boolean(*var_34);
    }
    #[allow(unused_mut)]
    let mut scope_35 = writer.prefix("Tags");
    if let Some(var_36) = &input.tags {
        let mut list_38 = scope_35.start_list(false, Some("Tag"));
        for item_37 in var_36 {
            #[allow(unused_mut)]
            let mut entry_39 = list_38.entry();
            crate::protocol_serde::shape_tag::ser_tag(entry_39, item_37)?;
        }
        list_38.finish();
    }
    #[allow(unused_mut)]
    let mut scope_40 = writer.prefix("StorageType");
    if let Some(var_41) = &input.storage_type {
        scope_40.string(var_41);
    }
    #[allow(unused_mut)]
    let mut scope_42 = writer.prefix("TdeCredentialArn");
    if let Some(var_43) = &input.tde_credential_arn {
        scope_42.string(var_43);
    }
    #[allow(unused_mut)]
    let mut scope_44 = writer.prefix("TdeCredentialPassword");
    if let Some(var_45) = &input.tde_credential_password {
        scope_44.string(var_45);
    }
    #[allow(unused_mut)]
    let mut scope_46 = writer.prefix("VpcSecurityGroupIds");
    if let Some(var_47) = &input.vpc_security_group_ids {
        let mut list_49 = scope_46.start_list(false, Some("VpcSecurityGroupId"));
        for item_48 in var_47 {
            #[allow(unused_mut)]
            let mut entry_50 = list_49.entry();
            entry_50.string(item_48);
        }
        list_49.finish();
    }
    #[allow(unused_mut)]
    let mut scope_51 = writer.prefix("Domain");
    if let Some(var_52) = &input.domain {
        scope_51.string(var_52);
    }
    #[allow(unused_mut)]
    let mut scope_53 = writer.prefix("DomainIAMRoleName");
    if let Some(var_54) = &input.domain_iam_role_name {
        scope_53.string(var_54);
    }
    #[allow(unused_mut)]
    let mut scope_55 = writer.prefix("DomainFqdn");
    if let Some(var_56) = &input.domain_fqdn {
        scope_55.string(var_56);
    }
    #[allow(unused_mut)]
    let mut scope_57 = writer.prefix("DomainOu");
    if let Some(var_58) = &input.domain_ou {
        scope_57.string(var_58);
    }
    #[allow(unused_mut)]
    let mut scope_59 = writer.prefix("DomainAuthSecretArn");
    if let Some(var_60) = &input.domain_auth_secret_arn {
        scope_59.string(var_60);
    }
    #[allow(unused_mut)]
    let mut scope_61 = writer.prefix("DomainDnsIps");
    if let Some(var_62) = &input.domain_dns_ips {
        let mut list_64 = scope_61.start_list(false, None);
        for item_63 in var_62 {
            #[allow(unused_mut)]
            let mut entry_65 = list_64.entry();
            entry_65.string(item_63);
        }
        list_64.finish();
    }
    #[allow(unused_mut)]
    let mut scope_66 = writer.prefix("EnableIAMDatabaseAuthentication");
    if let Some(var_67) = &input.enable_iam_database_authentication {
        scope_66.boolean(*var_67);
    }
    #[allow(unused_mut)]
    let mut scope_68 = writer.prefix("EnableCloudwatchLogsExports");
    if let Some(var_69) = &input.enable_cloudwatch_logs_exports {
        let mut list_71 = scope_68.start_list(false, None);
        for item_70 in var_69 {
            #[allow(unused_mut)]
            let mut entry_72 = list_71.entry();
            entry_72.string(item_70);
        }
        list_71.finish();
    }
    #[allow(unused_mut)]
    let mut scope_73 = writer.prefix("ProcessorFeatures");
    if let Some(var_74) = &input.processor_features {
        let mut list_76 = scope_73.start_list(false, Some("ProcessorFeature"));
        for item_75 in var_74 {
            #[allow(unused_mut)]
            let mut entry_77 = list_76.entry();
            crate::protocol_serde::shape_processor_feature::ser_processor_feature(entry_77, item_75)?;
        }
        list_76.finish();
    }
    #[allow(unused_mut)]
    let mut scope_78 = writer.prefix("UseDefaultProcessorFeatures");
    if let Some(var_79) = &input.use_default_processor_features {
        scope_78.boolean(*var_79);
    }
    #[allow(unused_mut)]
    let mut scope_80 = writer.prefix("DBParameterGroupName");
    if let Some(var_81) = &input.db_parameter_group_name {
        scope_80.string(var_81);
    }
    #[allow(unused_mut)]
    let mut scope_82 = writer.prefix("DeletionProtection");
    if let Some(var_83) = &input.deletion_protection {
        scope_82.boolean(*var_83);
    }
    #[allow(unused_mut)]
    let mut scope_84 = writer.prefix("SourceDbiResourceId");
    if let Some(var_85) = &input.source_dbi_resource_id {
        scope_84.string(var_85);
    }
    #[allow(unused_mut)]
    let mut scope_86 = writer.prefix("MaxAllocatedStorage");
    if let Some(var_87) = &input.max_allocated_storage {
        scope_86.number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_87).into()),
        );
    }
    #[allow(unused_mut)]
    let mut scope_88 = writer.prefix("SourceDBInstanceAutomatedBackupsArn");
    if let Some(var_89) = &input.source_db_instance_automated_backups_arn {
        scope_88.string(var_89);
    }
    #[allow(unused_mut)]
    let mut scope_90 = writer.prefix("EnableCustomerOwnedIp");
    if let Some(var_91) = &input.enable_customer_owned_ip {
        scope_90.boolean(*var_91);
    }
    #[allow(unused_mut)]
    let mut scope_92 = writer.prefix("CustomIamInstanceProfile");
    if let Some(var_93) = &input.custom_iam_instance_profile {
        scope_92.string(var_93);
    }
    #[allow(unused_mut)]
    let mut scope_94 = writer.prefix("BackupTarget");
    if let Some(var_95) = &input.backup_target {
        scope_94.string(var_95);
    }
    #[allow(unused_mut)]
    let mut scope_96 = writer.prefix("NetworkType");
    if let Some(var_97) = &input.network_type {
        scope_96.string(var_97);
    }
    #[allow(unused_mut)]
    let mut scope_98 = writer.prefix("StorageThroughput");
    if let Some(var_99) = &input.storage_throughput {
        scope_98.number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_99).into()),
        );
    }
    #[allow(unused_mut)]
    let mut scope_100 = writer.prefix("AllocatedStorage");
    if let Some(var_101) = &input.allocated_storage {
        scope_100.number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_101).into()),
        );
    }
    #[allow(unused_mut)]
    let mut scope_102 = writer.prefix("DedicatedLogVolume");
    if let Some(var_103) = &input.dedicated_log_volume {
        scope_102.boolean(*var_103);
    }
    #[allow(unused_mut)]
    let mut scope_104 = writer.prefix("CACertificateIdentifier");
    if let Some(var_105) = &input.ca_certificate_identifier {
        scope_104.string(var_105);
    }
    #[allow(unused_mut)]
    let mut scope_106 = writer.prefix("EngineLifecycleSupport");
    if let Some(var_107) = &input.engine_lifecycle_support {
        scope_106.string(var_107);
    }
    #[allow(unused_mut)]
    let mut scope_108 = writer.prefix("ManageMasterUserPassword");
    if let Some(var_109) = &input.manage_master_user_password {
        scope_108.boolean(*var_109);
    }
    #[allow(unused_mut)]
    let mut scope_110 = writer.prefix("MasterUserSecretKmsKeyId");
    if let Some(var_111) = &input.master_user_secret_kms_key_id {
        scope_110.string(var_111);
    }
    writer.finish();
    Ok(::aws_smithy_types::body::SdkBody::from(out))
}
