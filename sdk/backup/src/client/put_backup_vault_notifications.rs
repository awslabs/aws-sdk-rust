// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
impl super::Client {
    /// Constructs a fluent builder for the [`PutBackupVaultNotifications`](crate::operation::put_backup_vault_notifications::builders::PutBackupVaultNotificationsFluentBuilder) operation.
    ///
    /// - The fluent builder is configurable:
    ///   - [`backup_vault_name(impl Into<String>)`](crate::operation::put_backup_vault_notifications::builders::PutBackupVaultNotificationsFluentBuilder::backup_vault_name) / [`set_backup_vault_name(Option<String>)`](crate::operation::put_backup_vault_notifications::builders::PutBackupVaultNotificationsFluentBuilder::set_backup_vault_name):<br>required: **true**<br><p>The name of a logical container where backups are stored. Backup vaults are identified by names that are unique to the account used to create them and the Amazon Web Services Region where they are created.</p><br>
    ///   - [`sns_topic_arn(impl Into<String>)`](crate::operation::put_backup_vault_notifications::builders::PutBackupVaultNotificationsFluentBuilder::sns_topic_arn) / [`set_sns_topic_arn(Option<String>)`](crate::operation::put_backup_vault_notifications::builders::PutBackupVaultNotificationsFluentBuilder::set_sns_topic_arn):<br>required: **true**<br><p>The Amazon Resource Name (ARN) that specifies the topic for a backup vault’s events; for example, <code>arn:aws:sns:us-west-2:111122223333:MyVaultTopic</code>.</p><br>
    ///   - [`backup_vault_events(BackupVaultEvent)`](crate::operation::put_backup_vault_notifications::builders::PutBackupVaultNotificationsFluentBuilder::backup_vault_events) / [`set_backup_vault_events(Option<Vec::<BackupVaultEvent>>)`](crate::operation::put_backup_vault_notifications::builders::PutBackupVaultNotificationsFluentBuilder::set_backup_vault_events):<br>required: **true**<br><p>An array of events that indicate the status of jobs to back up resources to the backup vault. For the list of supported events, common use cases, and code samples, see <a href="https://docs.aws.amazon.com/aws-backup/latest/devguide/backup-notifications.html">Notification options with Backup</a>.</p><br>
    /// - On success, responds with [`PutBackupVaultNotificationsOutput`](crate::operation::put_backup_vault_notifications::PutBackupVaultNotificationsOutput)
    /// - On failure, responds with [`SdkError<PutBackupVaultNotificationsError>`](crate::operation::put_backup_vault_notifications::PutBackupVaultNotificationsError)
    pub fn put_backup_vault_notifications(
        &self,
    ) -> crate::operation::put_backup_vault_notifications::builders::PutBackupVaultNotificationsFluentBuilder {
        crate::operation::put_backup_vault_notifications::builders::PutBackupVaultNotificationsFluentBuilder::new(self.handle.clone())
    }
}
