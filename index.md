# AWS SDK for Rust
The AWS SDK for Rust contains one crate for each AWS service, as well as [aws-config](https://crates.io/crates/aws-config) ([docs](https://docs.rs/aws-config)), a crate implementing configuration loading such as credential providers. For usage documentation see the [Developer Guide](https://docs.aws.amazon.com/sdk-for-rust/latest/dg/welcome.html).
## AWS Services

| Service | Package |
| ------- | ------- |
| AWS ARC - Zonal Shift | [aws-sdk-arczonalshift](https://crates.io/crates/aws-sdk-arczonalshift) ([docs](https://docs.rs/aws-sdk-arczonalshift)) |
| AWS Account | [aws-sdk-account](https://crates.io/crates/aws-sdk-account) ([docs](https://docs.rs/aws-sdk-account)) |
| AWS Amplify | [aws-sdk-amplify](https://crates.io/crates/aws-sdk-amplify) ([docs](https://docs.rs/aws-sdk-amplify)) |
| AWS Amplify UI Builder | [aws-sdk-amplifyuibuilder](https://crates.io/crates/aws-sdk-amplifyuibuilder) ([docs](https://docs.rs/aws-sdk-amplifyuibuilder)) |
| AWS App Mesh | [aws-sdk-appmesh](https://crates.io/crates/aws-sdk-appmesh) ([docs](https://docs.rs/aws-sdk-appmesh)) |
| AWS App Runner | [aws-sdk-apprunner](https://crates.io/crates/aws-sdk-apprunner) ([docs](https://docs.rs/aws-sdk-apprunner)) |
| AWS AppConfig Data | [aws-sdk-appconfigdata](https://crates.io/crates/aws-sdk-appconfigdata) ([docs](https://docs.rs/aws-sdk-appconfigdata)) |
| AWS AppSync | [aws-sdk-appsync](https://crates.io/crates/aws-sdk-appsync) ([docs](https://docs.rs/aws-sdk-appsync)) |
| AWS Application Cost Profiler | [aws-sdk-applicationcostprofiler](https://crates.io/crates/aws-sdk-applicationcostprofiler) ([docs](https://docs.rs/aws-sdk-applicationcostprofiler)) |
| AWS Application Discovery Service | [aws-sdk-applicationdiscovery](https://crates.io/crates/aws-sdk-applicationdiscovery) ([docs](https://docs.rs/aws-sdk-applicationdiscovery)) |
| AWS Audit Manager | [aws-sdk-auditmanager](https://crates.io/crates/aws-sdk-auditmanager) ([docs](https://docs.rs/aws-sdk-auditmanager)) |
| AWS Auto Scaling Plans | [aws-sdk-autoscalingplans](https://crates.io/crates/aws-sdk-autoscalingplans) ([docs](https://docs.rs/aws-sdk-autoscalingplans)) |
| AWS B2B Data Interchange | [aws-sdk-b2bi](https://crates.io/crates/aws-sdk-b2bi) ([docs](https://docs.rs/aws-sdk-b2bi)) |
| AWS Backup | [aws-sdk-backup](https://crates.io/crates/aws-sdk-backup) ([docs](https://docs.rs/aws-sdk-backup)) |
| AWS Backup Gateway | [aws-sdk-backupgateway](https://crates.io/crates/aws-sdk-backupgateway) ([docs](https://docs.rs/aws-sdk-backupgateway)) |
| AWS Backup Storage | [aws-sdk-backupstorage](https://crates.io/crates/aws-sdk-backupstorage) ([docs](https://docs.rs/aws-sdk-backupstorage)) |
| AWS Batch | [aws-sdk-batch](https://crates.io/crates/aws-sdk-batch) ([docs](https://docs.rs/aws-sdk-batch)) |
| AWS Billing and Cost Management Data Exports | [aws-sdk-bcmdataexports](https://crates.io/crates/aws-sdk-bcmdataexports) ([docs](https://docs.rs/aws-sdk-bcmdataexports)) |
| AWS Budgets | [aws-sdk-budgets](https://crates.io/crates/aws-sdk-budgets) ([docs](https://docs.rs/aws-sdk-budgets)) |
| AWS Certificate Manager | [aws-sdk-acm](https://crates.io/crates/aws-sdk-acm) ([docs](https://docs.rs/aws-sdk-acm)) |
| AWS Certificate Manager Private Certificate Authority | [aws-sdk-acmpca](https://crates.io/crates/aws-sdk-acmpca) ([docs](https://docs.rs/aws-sdk-acmpca)) |
| AWS Clean Rooms ML | [aws-sdk-cleanroomsml](https://crates.io/crates/aws-sdk-cleanroomsml) ([docs](https://docs.rs/aws-sdk-cleanroomsml)) |
| AWS Clean Rooms Service | [aws-sdk-cleanrooms](https://crates.io/crates/aws-sdk-cleanrooms) ([docs](https://docs.rs/aws-sdk-cleanrooms)) |
| AWS Cloud Control API | [aws-sdk-cloudcontrol](https://crates.io/crates/aws-sdk-cloudcontrol) ([docs](https://docs.rs/aws-sdk-cloudcontrol)) |
| AWS Cloud Map | [aws-sdk-servicediscovery](https://crates.io/crates/aws-sdk-servicediscovery) ([docs](https://docs.rs/aws-sdk-servicediscovery)) |
| AWS Cloud9 | [aws-sdk-cloud9](https://crates.io/crates/aws-sdk-cloud9) ([docs](https://docs.rs/aws-sdk-cloud9)) |
| AWS CloudFormation | [aws-sdk-cloudformation](https://crates.io/crates/aws-sdk-cloudformation) ([docs](https://docs.rs/aws-sdk-cloudformation)) |
| AWS CloudHSM V2 | [aws-sdk-cloudhsmv2](https://crates.io/crates/aws-sdk-cloudhsmv2) ([docs](https://docs.rs/aws-sdk-cloudhsmv2)) |
| AWS CloudTrail | [aws-sdk-cloudtrail](https://crates.io/crates/aws-sdk-cloudtrail) ([docs](https://docs.rs/aws-sdk-cloudtrail)) |
| AWS CloudTrail Data Service | [aws-sdk-cloudtraildata](https://crates.io/crates/aws-sdk-cloudtraildata) ([docs](https://docs.rs/aws-sdk-cloudtraildata)) |
| AWS CodeBuild | [aws-sdk-codebuild](https://crates.io/crates/aws-sdk-codebuild) ([docs](https://docs.rs/aws-sdk-codebuild)) |
| AWS CodeCommit | [aws-sdk-codecommit](https://crates.io/crates/aws-sdk-codecommit) ([docs](https://docs.rs/aws-sdk-codecommit)) |
| AWS CodeDeploy | [aws-sdk-codedeploy](https://crates.io/crates/aws-sdk-codedeploy) ([docs](https://docs.rs/aws-sdk-codedeploy)) |
| AWS CodePipeline | [aws-sdk-codepipeline](https://crates.io/crates/aws-sdk-codepipeline) ([docs](https://docs.rs/aws-sdk-codepipeline)) |
| AWS CodeStar | [aws-sdk-codestar](https://crates.io/crates/aws-sdk-codestar) ([docs](https://docs.rs/aws-sdk-codestar)) |
| AWS CodeStar Notifications | [aws-sdk-codestarnotifications](https://crates.io/crates/aws-sdk-codestarnotifications) ([docs](https://docs.rs/aws-sdk-codestarnotifications)) |
| AWS CodeStar connections | [aws-sdk-codestarconnections](https://crates.io/crates/aws-sdk-codestarconnections) ([docs](https://docs.rs/aws-sdk-codestarconnections)) |
| AWS Comprehend Medical | [aws-sdk-comprehendmedical](https://crates.io/crates/aws-sdk-comprehendmedical) ([docs](https://docs.rs/aws-sdk-comprehendmedical)) |
| AWS Compute Optimizer | [aws-sdk-computeoptimizer](https://crates.io/crates/aws-sdk-computeoptimizer) ([docs](https://docs.rs/aws-sdk-computeoptimizer)) |
| AWS Config | [aws-sdk-config](https://crates.io/crates/aws-sdk-config) ([docs](https://docs.rs/aws-sdk-config)) |
| AWS Control Tower | [aws-sdk-controltower](https://crates.io/crates/aws-sdk-controltower) ([docs](https://docs.rs/aws-sdk-controltower)) |
| AWS Cost Explorer Service | [aws-sdk-costexplorer](https://crates.io/crates/aws-sdk-costexplorer) ([docs](https://docs.rs/aws-sdk-costexplorer)) |
| AWS Cost and Usage Report Service | [aws-sdk-costandusagereport](https://crates.io/crates/aws-sdk-costandusagereport) ([docs](https://docs.rs/aws-sdk-costandusagereport)) |
| AWS Data Exchange | [aws-sdk-dataexchange](https://crates.io/crates/aws-sdk-dataexchange) ([docs](https://docs.rs/aws-sdk-dataexchange)) |
| AWS Data Pipeline | [aws-sdk-datapipeline](https://crates.io/crates/aws-sdk-datapipeline) ([docs](https://docs.rs/aws-sdk-datapipeline)) |
| AWS DataSync | [aws-sdk-datasync](https://crates.io/crates/aws-sdk-datasync) ([docs](https://docs.rs/aws-sdk-datasync)) |
| AWS Database Migration Service | [aws-sdk-databasemigration](https://crates.io/crates/aws-sdk-databasemigration) ([docs](https://docs.rs/aws-sdk-databasemigration)) |
| AWS Device Farm | [aws-sdk-devicefarm](https://crates.io/crates/aws-sdk-devicefarm) ([docs](https://docs.rs/aws-sdk-devicefarm)) |
| AWS Direct Connect | [aws-sdk-directconnect](https://crates.io/crates/aws-sdk-directconnect) ([docs](https://docs.rs/aws-sdk-directconnect)) |
| AWS Directory Service | [aws-sdk-directory](https://crates.io/crates/aws-sdk-directory) ([docs](https://docs.rs/aws-sdk-directory)) |
| AWS EC2 Instance Connect | [aws-sdk-ec2instanceconnect](https://crates.io/crates/aws-sdk-ec2instanceconnect) ([docs](https://docs.rs/aws-sdk-ec2instanceconnect)) |
| AWS Elastic Beanstalk | [aws-sdk-elasticbeanstalk](https://crates.io/crates/aws-sdk-elasticbeanstalk) ([docs](https://docs.rs/aws-sdk-elasticbeanstalk)) |
| AWS Elemental MediaConvert | [aws-sdk-mediaconvert](https://crates.io/crates/aws-sdk-mediaconvert) ([docs](https://docs.rs/aws-sdk-mediaconvert)) |
| AWS Elemental MediaLive | [aws-sdk-medialive](https://crates.io/crates/aws-sdk-medialive) ([docs](https://docs.rs/aws-sdk-medialive)) |
| AWS Elemental MediaPackage | [aws-sdk-mediapackage](https://crates.io/crates/aws-sdk-mediapackage) ([docs](https://docs.rs/aws-sdk-mediapackage)) |
| AWS Elemental MediaPackage VOD | [aws-sdk-mediapackagevod](https://crates.io/crates/aws-sdk-mediapackagevod) ([docs](https://docs.rs/aws-sdk-mediapackagevod)) |
| AWS Elemental MediaPackage v2 | [aws-sdk-mediapackagev2](https://crates.io/crates/aws-sdk-mediapackagev2) ([docs](https://docs.rs/aws-sdk-mediapackagev2)) |
| AWS Elemental MediaStore | [aws-sdk-mediastore](https://crates.io/crates/aws-sdk-mediastore) ([docs](https://docs.rs/aws-sdk-mediastore)) |
| AWS Elemental MediaStore Data Plane | [aws-sdk-mediastoredata](https://crates.io/crates/aws-sdk-mediastoredata) ([docs](https://docs.rs/aws-sdk-mediastoredata)) |
| AWS EntityResolution | [aws-sdk-entityresolution](https://crates.io/crates/aws-sdk-entityresolution) ([docs](https://docs.rs/aws-sdk-entityresolution)) |
| AWS Fault Injection Simulator | [aws-sdk-fis](https://crates.io/crates/aws-sdk-fis) ([docs](https://docs.rs/aws-sdk-fis)) |
| AWS Free Tier | [aws-sdk-freetier](https://crates.io/crates/aws-sdk-freetier) ([docs](https://docs.rs/aws-sdk-freetier)) |
| AWS Global Accelerator | [aws-sdk-globalaccelerator](https://crates.io/crates/aws-sdk-globalaccelerator) ([docs](https://docs.rs/aws-sdk-globalaccelerator)) |
| AWS Glue | [aws-sdk-glue](https://crates.io/crates/aws-sdk-glue) ([docs](https://docs.rs/aws-sdk-glue)) |
| AWS Glue DataBrew | [aws-sdk-databrew](https://crates.io/crates/aws-sdk-databrew) ([docs](https://docs.rs/aws-sdk-databrew)) |
| AWS Greengrass | [aws-sdk-greengrass](https://crates.io/crates/aws-sdk-greengrass) ([docs](https://docs.rs/aws-sdk-greengrass)) |
| AWS Ground Station | [aws-sdk-groundstation](https://crates.io/crates/aws-sdk-groundstation) ([docs](https://docs.rs/aws-sdk-groundstation)) |
| AWS Health APIs and Notifications | [aws-sdk-health](https://crates.io/crates/aws-sdk-health) ([docs](https://docs.rs/aws-sdk-health)) |
| AWS Health Imaging | [aws-sdk-medicalimaging](https://crates.io/crates/aws-sdk-medicalimaging) ([docs](https://docs.rs/aws-sdk-medicalimaging)) |
| AWS Identity and Access Management | [aws-sdk-iam](https://crates.io/crates/aws-sdk-iam) ([docs](https://docs.rs/aws-sdk-iam)) |
| AWS IoT | [aws-sdk-iot](https://crates.io/crates/aws-sdk-iot) ([docs](https://docs.rs/aws-sdk-iot)) |
| AWS IoT 1-Click Devices Service | [aws-sdk-iot1clickdevices](https://crates.io/crates/aws-sdk-iot1clickdevices) ([docs](https://docs.rs/aws-sdk-iot1clickdevices)) |
| AWS IoT 1-Click Projects Service | [aws-sdk-iot1clickprojects](https://crates.io/crates/aws-sdk-iot1clickprojects) ([docs](https://docs.rs/aws-sdk-iot1clickprojects)) |
| AWS IoT Analytics | [aws-sdk-iotanalytics](https://crates.io/crates/aws-sdk-iotanalytics) ([docs](https://docs.rs/aws-sdk-iotanalytics)) |
| AWS IoT Core Device Advisor | [aws-sdk-iotdeviceadvisor](https://crates.io/crates/aws-sdk-iotdeviceadvisor) ([docs](https://docs.rs/aws-sdk-iotdeviceadvisor)) |
| AWS IoT Data Plane | [aws-sdk-iotdataplane](https://crates.io/crates/aws-sdk-iotdataplane) ([docs](https://docs.rs/aws-sdk-iotdataplane)) |
| AWS IoT Events | [aws-sdk-iotevents](https://crates.io/crates/aws-sdk-iotevents) ([docs](https://docs.rs/aws-sdk-iotevents)) |
| AWS IoT Events Data | [aws-sdk-ioteventsdata](https://crates.io/crates/aws-sdk-ioteventsdata) ([docs](https://docs.rs/aws-sdk-ioteventsdata)) |
| AWS IoT Fleet Hub | [aws-sdk-iotfleethub](https://crates.io/crates/aws-sdk-iotfleethub) ([docs](https://docs.rs/aws-sdk-iotfleethub)) |
| AWS IoT FleetWise | [aws-sdk-iotfleetwise](https://crates.io/crates/aws-sdk-iotfleetwise) ([docs](https://docs.rs/aws-sdk-iotfleetwise)) |
| AWS IoT Greengrass V2 | [aws-sdk-greengrassv2](https://crates.io/crates/aws-sdk-greengrassv2) ([docs](https://docs.rs/aws-sdk-greengrassv2)) |
| AWS IoT Jobs Data Plane | [aws-sdk-iotjobsdataplane](https://crates.io/crates/aws-sdk-iotjobsdataplane) ([docs](https://docs.rs/aws-sdk-iotjobsdataplane)) |
| AWS IoT RoboRunner | [aws-sdk-iotroborunner](https://crates.io/crates/aws-sdk-iotroborunner) ([docs](https://docs.rs/aws-sdk-iotroborunner)) |
| AWS IoT Secure Tunneling | [aws-sdk-iotsecuretunneling](https://crates.io/crates/aws-sdk-iotsecuretunneling) ([docs](https://docs.rs/aws-sdk-iotsecuretunneling)) |
| AWS IoT SiteWise | [aws-sdk-iotsitewise](https://crates.io/crates/aws-sdk-iotsitewise) ([docs](https://docs.rs/aws-sdk-iotsitewise)) |
| AWS IoT Things Graph | [aws-sdk-iotthingsgraph](https://crates.io/crates/aws-sdk-iotthingsgraph) ([docs](https://docs.rs/aws-sdk-iotthingsgraph)) |
| AWS IoT TwinMaker | [aws-sdk-iottwinmaker](https://crates.io/crates/aws-sdk-iottwinmaker) ([docs](https://docs.rs/aws-sdk-iottwinmaker)) |
| AWS IoT Wireless | [aws-sdk-iotwireless](https://crates.io/crates/aws-sdk-iotwireless) ([docs](https://docs.rs/aws-sdk-iotwireless)) |
| AWS Key Management Service | [aws-sdk-kms](https://crates.io/crates/aws-sdk-kms) ([docs](https://docs.rs/aws-sdk-kms)) |
| AWS Lake Formation | [aws-sdk-lakeformation](https://crates.io/crates/aws-sdk-lakeformation) ([docs](https://docs.rs/aws-sdk-lakeformation)) |
| AWS Lambda | [aws-sdk-lambda](https://crates.io/crates/aws-sdk-lambda) ([docs](https://docs.rs/aws-sdk-lambda)) ([examples](https://github.com/awslabs/aws-sdk-rust/tree/main/examples/lambda)) |
| AWS Launch Wizard | [aws-sdk-launchwizard](https://crates.io/crates/aws-sdk-launchwizard) ([docs](https://docs.rs/aws-sdk-launchwizard)) |
| AWS License Manager | [aws-sdk-licensemanager](https://crates.io/crates/aws-sdk-licensemanager) ([docs](https://docs.rs/aws-sdk-licensemanager)) |
| AWS License Manager Linux Subscriptions | [aws-sdk-licensemanagerlinuxsubscriptions](https://crates.io/crates/aws-sdk-licensemanagerlinuxsubscriptions) ([docs](https://docs.rs/aws-sdk-licensemanagerlinuxsubscriptions)) |
| AWS License Manager User Subscriptions | [aws-sdk-licensemanagerusersubscriptions](https://crates.io/crates/aws-sdk-licensemanagerusersubscriptions) ([docs](https://docs.rs/aws-sdk-licensemanagerusersubscriptions)) |
| AWS Marketplace Agreement Service | [aws-sdk-marketplaceagreement](https://crates.io/crates/aws-sdk-marketplaceagreement) ([docs](https://docs.rs/aws-sdk-marketplaceagreement)) |
| AWS Marketplace Catalog Service | [aws-sdk-marketplacecatalog](https://crates.io/crates/aws-sdk-marketplacecatalog) ([docs](https://docs.rs/aws-sdk-marketplacecatalog)) |
| AWS Marketplace Commerce Analytics | [aws-sdk-marketplacecommerceanalytics](https://crates.io/crates/aws-sdk-marketplacecommerceanalytics) ([docs](https://docs.rs/aws-sdk-marketplacecommerceanalytics)) |
| AWS Marketplace Deployment Service | [aws-sdk-marketplacedeployment](https://crates.io/crates/aws-sdk-marketplacedeployment) ([docs](https://docs.rs/aws-sdk-marketplacedeployment)) |
| AWS Marketplace Entitlement Service | [aws-sdk-marketplaceentitlement](https://crates.io/crates/aws-sdk-marketplaceentitlement) ([docs](https://docs.rs/aws-sdk-marketplaceentitlement)) |
| AWS MediaConnect | [aws-sdk-mediaconnect](https://crates.io/crates/aws-sdk-mediaconnect) ([docs](https://docs.rs/aws-sdk-mediaconnect)) |
| AWS MediaTailor | [aws-sdk-mediatailor](https://crates.io/crates/aws-sdk-mediatailor) ([docs](https://docs.rs/aws-sdk-mediatailor)) |
| AWS Migration Hub | [aws-sdk-migrationhub](https://crates.io/crates/aws-sdk-migrationhub) ([docs](https://docs.rs/aws-sdk-migrationhub)) |
| AWS Migration Hub Config | [aws-sdk-migrationhubconfig](https://crates.io/crates/aws-sdk-migrationhubconfig) ([docs](https://docs.rs/aws-sdk-migrationhubconfig)) |
| AWS Migration Hub Orchestrator | [aws-sdk-migrationhuborchestrator](https://crates.io/crates/aws-sdk-migrationhuborchestrator) ([docs](https://docs.rs/aws-sdk-migrationhuborchestrator)) |
| AWS Migration Hub Refactor Spaces | [aws-sdk-migrationhubrefactorspaces](https://crates.io/crates/aws-sdk-migrationhubrefactorspaces) ([docs](https://docs.rs/aws-sdk-migrationhubrefactorspaces)) |
| AWS Mobile | [aws-sdk-mobile](https://crates.io/crates/aws-sdk-mobile) ([docs](https://docs.rs/aws-sdk-mobile)) |
| AWS Network Firewall | [aws-sdk-networkfirewall](https://crates.io/crates/aws-sdk-networkfirewall) ([docs](https://docs.rs/aws-sdk-networkfirewall)) |
| AWS Network Manager | [aws-sdk-networkmanager](https://crates.io/crates/aws-sdk-networkmanager) ([docs](https://docs.rs/aws-sdk-networkmanager)) |
| AWS OpsWorks | [aws-sdk-opsworks](https://crates.io/crates/aws-sdk-opsworks) ([docs](https://docs.rs/aws-sdk-opsworks)) |
| AWS OpsWorks CM | [aws-sdk-opsworkscm](https://crates.io/crates/aws-sdk-opsworkscm) ([docs](https://docs.rs/aws-sdk-opsworkscm)) |
| AWS Organizations | [aws-sdk-organizations](https://crates.io/crates/aws-sdk-organizations) ([docs](https://docs.rs/aws-sdk-organizations)) |
| AWS Outposts | [aws-sdk-outposts](https://crates.io/crates/aws-sdk-outposts) ([docs](https://docs.rs/aws-sdk-outposts)) |
| AWS Panorama | [aws-sdk-panorama](https://crates.io/crates/aws-sdk-panorama) ([docs](https://docs.rs/aws-sdk-panorama)) |
| AWS Performance Insights | [aws-sdk-pi](https://crates.io/crates/aws-sdk-pi) ([docs](https://docs.rs/aws-sdk-pi)) |
| AWS Price List Service | [aws-sdk-pricing](https://crates.io/crates/aws-sdk-pricing) ([docs](https://docs.rs/aws-sdk-pricing)) |
| AWS Private 5G | [aws-sdk-privatenetworks](https://crates.io/crates/aws-sdk-privatenetworks) ([docs](https://docs.rs/aws-sdk-privatenetworks)) |
| AWS Proton | [aws-sdk-proton](https://crates.io/crates/aws-sdk-proton) ([docs](https://docs.rs/aws-sdk-proton)) |
| AWS RDS DataService | [aws-sdk-rdsdata](https://crates.io/crates/aws-sdk-rdsdata) ([docs](https://docs.rs/aws-sdk-rdsdata)) |
| AWS Resilience Hub | [aws-sdk-resiliencehub](https://crates.io/crates/aws-sdk-resiliencehub) ([docs](https://docs.rs/aws-sdk-resiliencehub)) |
| AWS Resource Access Manager | [aws-sdk-ram](https://crates.io/crates/aws-sdk-ram) ([docs](https://docs.rs/aws-sdk-ram)) |
| AWS Resource Explorer | [aws-sdk-resourceexplorer2](https://crates.io/crates/aws-sdk-resourceexplorer2) ([docs](https://docs.rs/aws-sdk-resourceexplorer2)) |
| AWS Resource Groups | [aws-sdk-resourcegroups](https://crates.io/crates/aws-sdk-resourcegroups) ([docs](https://docs.rs/aws-sdk-resourcegroups)) |
| AWS Resource Groups Tagging API | [aws-sdk-resourcegroupstagging](https://crates.io/crates/aws-sdk-resourcegroupstagging) ([docs](https://docs.rs/aws-sdk-resourcegroupstagging)) |
| AWS RoboMaker | [aws-sdk-robomaker](https://crates.io/crates/aws-sdk-robomaker) ([docs](https://docs.rs/aws-sdk-robomaker)) |
| AWS Route53 Recovery Control Config | [aws-sdk-route53recoverycontrolconfig](https://crates.io/crates/aws-sdk-route53recoverycontrolconfig) ([docs](https://docs.rs/aws-sdk-route53recoverycontrolconfig)) |
| AWS Route53 Recovery Readiness | [aws-sdk-route53recoveryreadiness](https://crates.io/crates/aws-sdk-route53recoveryreadiness) ([docs](https://docs.rs/aws-sdk-route53recoveryreadiness)) |
| AWS S3 Control | [aws-sdk-s3control](https://crates.io/crates/aws-sdk-s3control) ([docs](https://docs.rs/aws-sdk-s3control)) |
| AWS SSO Identity Store | [aws-sdk-identitystore](https://crates.io/crates/aws-sdk-identitystore) ([docs](https://docs.rs/aws-sdk-identitystore)) |
| AWS SSO OIDC | [aws-sdk-ssooidc](https://crates.io/crates/aws-sdk-ssooidc) ([docs](https://docs.rs/aws-sdk-ssooidc)) |
| AWS Savings Plans | [aws-sdk-savingsplans](https://crates.io/crates/aws-sdk-savingsplans) ([docs](https://docs.rs/aws-sdk-savingsplans)) |
| AWS Secrets Manager | [aws-sdk-secretsmanager](https://crates.io/crates/aws-sdk-secretsmanager) ([docs](https://docs.rs/aws-sdk-secretsmanager)) |
| AWS Security Token Service | [aws-sdk-sts](https://crates.io/crates/aws-sdk-sts) ([docs](https://docs.rs/aws-sdk-sts)) |
| AWS SecurityHub | [aws-sdk-securityhub](https://crates.io/crates/aws-sdk-securityhub) ([docs](https://docs.rs/aws-sdk-securityhub)) |
| AWS Server Migration Service | [aws-sdk-sms](https://crates.io/crates/aws-sdk-sms) ([docs](https://docs.rs/aws-sdk-sms)) |
| AWS Service Catalog | [aws-sdk-servicecatalog](https://crates.io/crates/aws-sdk-servicecatalog) ([docs](https://docs.rs/aws-sdk-servicecatalog)) |
| AWS Service Catalog App Registry | [aws-sdk-servicecatalogappregistry](https://crates.io/crates/aws-sdk-servicecatalogappregistry) ([docs](https://docs.rs/aws-sdk-servicecatalogappregistry)) |
| AWS Shield | [aws-sdk-shield](https://crates.io/crates/aws-sdk-shield) ([docs](https://docs.rs/aws-sdk-shield)) |
| AWS Signer | [aws-sdk-signer](https://crates.io/crates/aws-sdk-signer) ([docs](https://docs.rs/aws-sdk-signer)) |
| AWS SimSpace Weaver | [aws-sdk-simspaceweaver](https://crates.io/crates/aws-sdk-simspaceweaver) ([docs](https://docs.rs/aws-sdk-simspaceweaver)) |
| AWS Single Sign-On | [aws-sdk-sso](https://crates.io/crates/aws-sdk-sso) ([docs](https://docs.rs/aws-sdk-sso)) |
| AWS Single Sign-On Admin | [aws-sdk-ssoadmin](https://crates.io/crates/aws-sdk-ssoadmin) ([docs](https://docs.rs/aws-sdk-ssoadmin)) |
| AWS Snow Device Management | [aws-sdk-snowdevicemanagement](https://crates.io/crates/aws-sdk-snowdevicemanagement) ([docs](https://docs.rs/aws-sdk-snowdevicemanagement)) |
| AWS Step Functions | [aws-sdk-sfn](https://crates.io/crates/aws-sdk-sfn) ([docs](https://docs.rs/aws-sdk-sfn)) |
| AWS Storage Gateway | [aws-sdk-storagegateway](https://crates.io/crates/aws-sdk-storagegateway) ([docs](https://docs.rs/aws-sdk-storagegateway)) |
| AWS Supply Chain | [aws-sdk-supplychain](https://crates.io/crates/aws-sdk-supplychain) ([docs](https://docs.rs/aws-sdk-supplychain)) |
| AWS Support | [aws-sdk-support](https://crates.io/crates/aws-sdk-support) ([docs](https://docs.rs/aws-sdk-support)) |
| AWS Support App | [aws-sdk-supportapp](https://crates.io/crates/aws-sdk-supportapp) ([docs](https://docs.rs/aws-sdk-supportapp)) |
| AWS Systems Manager Incident Manager | [aws-sdk-ssmincidents](https://crates.io/crates/aws-sdk-ssmincidents) ([docs](https://docs.rs/aws-sdk-ssmincidents)) |
| AWS Systems Manager Incident Manager Contacts | [aws-sdk-ssmcontacts](https://crates.io/crates/aws-sdk-ssmcontacts) ([docs](https://docs.rs/aws-sdk-ssmcontacts)) |
| AWS Systems Manager for SAP | [aws-sdk-ssmsap](https://crates.io/crates/aws-sdk-ssmsap) ([docs](https://docs.rs/aws-sdk-ssmsap)) |
| AWS Telco Network Builder | [aws-sdk-tnb](https://crates.io/crates/aws-sdk-tnb) ([docs](https://docs.rs/aws-sdk-tnb)) |
| AWS Transfer Family | [aws-sdk-transfer](https://crates.io/crates/aws-sdk-transfer) ([docs](https://docs.rs/aws-sdk-transfer)) |
| AWS WAF | [aws-sdk-waf](https://crates.io/crates/aws-sdk-waf) ([docs](https://docs.rs/aws-sdk-waf)) |
| AWS WAF Regional | [aws-sdk-wafregional](https://crates.io/crates/aws-sdk-wafregional) ([docs](https://docs.rs/aws-sdk-wafregional)) |
| AWS WAFV2 | [aws-sdk-wafv2](https://crates.io/crates/aws-sdk-wafv2) ([docs](https://docs.rs/aws-sdk-wafv2)) |
| AWS Well-Architected Tool | [aws-sdk-wellarchitected](https://crates.io/crates/aws-sdk-wellarchitected) ([docs](https://docs.rs/aws-sdk-wellarchitected)) |
| AWS X-Ray | [aws-sdk-xray](https://crates.io/crates/aws-sdk-xray) ([docs](https://docs.rs/aws-sdk-xray)) |
| AWS re:Post Private | [aws-sdk-repostspace](https://crates.io/crates/aws-sdk-repostspace) ([docs](https://docs.rs/aws-sdk-repostspace)) |
| AWSBillingConductor | [aws-sdk-billingconductor](https://crates.io/crates/aws-sdk-billingconductor) ([docs](https://docs.rs/aws-sdk-billingconductor)) |
| AWSKendraFrontendService | [aws-sdk-kendra](https://crates.io/crates/aws-sdk-kendra) ([docs](https://docs.rs/aws-sdk-kendra)) |
| AWSMainframeModernization | [aws-sdk-m2](https://crates.io/crates/aws-sdk-m2) ([docs](https://docs.rs/aws-sdk-m2)) |
| AWSMarketplace Metering | [aws-sdk-marketplacemetering](https://crates.io/crates/aws-sdk-marketplacemetering) ([docs](https://docs.rs/aws-sdk-marketplacemetering)) |
| AWSServerlessApplicationRepository | [aws-sdk-serverlessapplicationrepository](https://crates.io/crates/aws-sdk-serverlessapplicationrepository) ([docs](https://docs.rs/aws-sdk-serverlessapplicationrepository)) |
| Access Analyzer | [aws-sdk-accessanalyzer](https://crates.io/crates/aws-sdk-accessanalyzer) ([docs](https://docs.rs/aws-sdk-accessanalyzer)) |
| Agents for Amazon Bedrock | [aws-sdk-bedrockagent](https://crates.io/crates/aws-sdk-bedrockagent) ([docs](https://docs.rs/aws-sdk-bedrockagent)) |
| Agents for Amazon Bedrock Runtime | [aws-sdk-bedrockagentruntime](https://crates.io/crates/aws-sdk-bedrockagentruntime) ([docs](https://docs.rs/aws-sdk-bedrockagentruntime)) |
| Alexa For Business | [aws-sdk-alexaforbusiness](https://crates.io/crates/aws-sdk-alexaforbusiness) ([docs](https://docs.rs/aws-sdk-alexaforbusiness)) |
| Amazon API Gateway | [aws-sdk-apigateway](https://crates.io/crates/aws-sdk-apigateway) ([docs](https://docs.rs/aws-sdk-apigateway)) |
| Amazon AppConfig | [aws-sdk-appconfig](https://crates.io/crates/aws-sdk-appconfig) ([docs](https://docs.rs/aws-sdk-appconfig)) |
| Amazon AppIntegrations Service | [aws-sdk-appintegrations](https://crates.io/crates/aws-sdk-appintegrations) ([docs](https://docs.rs/aws-sdk-appintegrations)) |
| Amazon AppStream | [aws-sdk-appstream](https://crates.io/crates/aws-sdk-appstream) ([docs](https://docs.rs/aws-sdk-appstream)) |
| Amazon Appflow | [aws-sdk-appflow](https://crates.io/crates/aws-sdk-appflow) ([docs](https://docs.rs/aws-sdk-appflow)) |
| Amazon Athena | [aws-sdk-athena](https://crates.io/crates/aws-sdk-athena) ([docs](https://docs.rs/aws-sdk-athena)) |
| Amazon Augmented AI Runtime | [aws-sdk-sagemakera2iruntime](https://crates.io/crates/aws-sdk-sagemakera2iruntime) ([docs](https://docs.rs/aws-sdk-sagemakera2iruntime)) |
| Amazon Bedrock | [aws-sdk-bedrock](https://crates.io/crates/aws-sdk-bedrock) ([docs](https://docs.rs/aws-sdk-bedrock)) |
| Amazon Bedrock Runtime | [aws-sdk-bedrockruntime](https://crates.io/crates/aws-sdk-bedrockruntime) ([docs](https://docs.rs/aws-sdk-bedrockruntime)) |
| Amazon Chime | [aws-sdk-chime](https://crates.io/crates/aws-sdk-chime) ([docs](https://docs.rs/aws-sdk-chime)) |
| Amazon Chime SDK Identity | [aws-sdk-chimesdkidentity](https://crates.io/crates/aws-sdk-chimesdkidentity) ([docs](https://docs.rs/aws-sdk-chimesdkidentity)) |
| Amazon Chime SDK Media Pipelines | [aws-sdk-chimesdkmediapipelines](https://crates.io/crates/aws-sdk-chimesdkmediapipelines) ([docs](https://docs.rs/aws-sdk-chimesdkmediapipelines)) |
| Amazon Chime SDK Meetings | [aws-sdk-chimesdkmeetings](https://crates.io/crates/aws-sdk-chimesdkmeetings) ([docs](https://docs.rs/aws-sdk-chimesdkmeetings)) |
| Amazon Chime SDK Messaging | [aws-sdk-chimesdkmessaging](https://crates.io/crates/aws-sdk-chimesdkmessaging) ([docs](https://docs.rs/aws-sdk-chimesdkmessaging)) |
| Amazon Chime SDK Voice | [aws-sdk-chimesdkvoice](https://crates.io/crates/aws-sdk-chimesdkvoice) ([docs](https://docs.rs/aws-sdk-chimesdkvoice)) |
| Amazon CloudDirectory | [aws-sdk-clouddirectory](https://crates.io/crates/aws-sdk-clouddirectory) ([docs](https://docs.rs/aws-sdk-clouddirectory)) |
| Amazon CloudFront | [aws-sdk-cloudfront](https://crates.io/crates/aws-sdk-cloudfront) ([docs](https://docs.rs/aws-sdk-cloudfront)) |
| Amazon CloudFront KeyValueStore | [aws-sdk-cloudfrontkeyvaluestore](https://crates.io/crates/aws-sdk-cloudfrontkeyvaluestore) ([docs](https://docs.rs/aws-sdk-cloudfrontkeyvaluestore)) |
| Amazon CloudHSM | [aws-sdk-cloudhsm](https://crates.io/crates/aws-sdk-cloudhsm) ([docs](https://docs.rs/aws-sdk-cloudhsm)) |
| Amazon CloudSearch | [aws-sdk-cloudsearch](https://crates.io/crates/aws-sdk-cloudsearch) ([docs](https://docs.rs/aws-sdk-cloudsearch)) |
| Amazon CloudSearch Domain | [aws-sdk-cloudsearchdomain](https://crates.io/crates/aws-sdk-cloudsearchdomain) ([docs](https://docs.rs/aws-sdk-cloudsearchdomain)) |
| Amazon CloudWatch | [aws-sdk-cloudwatch](https://crates.io/crates/aws-sdk-cloudwatch) ([docs](https://docs.rs/aws-sdk-cloudwatch)) |
| Amazon CloudWatch Application Insights | [aws-sdk-applicationinsights](https://crates.io/crates/aws-sdk-applicationinsights) ([docs](https://docs.rs/aws-sdk-applicationinsights)) |
| Amazon CloudWatch Events | [aws-sdk-cloudwatchevents](https://crates.io/crates/aws-sdk-cloudwatchevents) ([docs](https://docs.rs/aws-sdk-cloudwatchevents)) |
| Amazon CloudWatch Evidently | [aws-sdk-evidently](https://crates.io/crates/aws-sdk-evidently) ([docs](https://docs.rs/aws-sdk-evidently)) |
| Amazon CloudWatch Internet Monitor | [aws-sdk-internetmonitor](https://crates.io/crates/aws-sdk-internetmonitor) ([docs](https://docs.rs/aws-sdk-internetmonitor)) |
| Amazon CloudWatch Logs | [aws-sdk-cloudwatchlogs](https://crates.io/crates/aws-sdk-cloudwatchlogs) ([docs](https://docs.rs/aws-sdk-cloudwatchlogs)) |
| Amazon CloudWatch Network Monitor | [aws-sdk-networkmonitor](https://crates.io/crates/aws-sdk-networkmonitor) ([docs](https://docs.rs/aws-sdk-networkmonitor)) |
| Amazon CodeCatalyst | [aws-sdk-codecatalyst](https://crates.io/crates/aws-sdk-codecatalyst) ([docs](https://docs.rs/aws-sdk-codecatalyst)) |
| Amazon CodeGuru Profiler | [aws-sdk-codeguruprofiler](https://crates.io/crates/aws-sdk-codeguruprofiler) ([docs](https://docs.rs/aws-sdk-codeguruprofiler)) |
| Amazon CodeGuru Reviewer | [aws-sdk-codegurureviewer](https://crates.io/crates/aws-sdk-codegurureviewer) ([docs](https://docs.rs/aws-sdk-codegurureviewer)) |
| Amazon CodeGuru Security | [aws-sdk-codegurusecurity](https://crates.io/crates/aws-sdk-codegurusecurity) ([docs](https://docs.rs/aws-sdk-codegurusecurity)) |
| Amazon Cognito Identity | [aws-sdk-cognitoidentity](https://crates.io/crates/aws-sdk-cognitoidentity) ([docs](https://docs.rs/aws-sdk-cognitoidentity)) |
| Amazon Cognito Identity Provider | [aws-sdk-cognitoidentityprovider](https://crates.io/crates/aws-sdk-cognitoidentityprovider) ([docs](https://docs.rs/aws-sdk-cognitoidentityprovider)) |
| Amazon Cognito Sync | [aws-sdk-cognitosync](https://crates.io/crates/aws-sdk-cognitosync) ([docs](https://docs.rs/aws-sdk-cognitosync)) |
| Amazon Comprehend | [aws-sdk-comprehend](https://crates.io/crates/aws-sdk-comprehend) ([docs](https://docs.rs/aws-sdk-comprehend)) |
| Amazon Connect Cases | [aws-sdk-connectcases](https://crates.io/crates/aws-sdk-connectcases) ([docs](https://docs.rs/aws-sdk-connectcases)) |
| Amazon Connect Contact Lens | [aws-sdk-connectcontactlens](https://crates.io/crates/aws-sdk-connectcontactlens) ([docs](https://docs.rs/aws-sdk-connectcontactlens)) |
| Amazon Connect Customer Profiles | [aws-sdk-customerprofiles](https://crates.io/crates/aws-sdk-customerprofiles) ([docs](https://docs.rs/aws-sdk-customerprofiles)) |
| Amazon Connect Participant Service | [aws-sdk-connectparticipant](https://crates.io/crates/aws-sdk-connectparticipant) ([docs](https://docs.rs/aws-sdk-connectparticipant)) |
| Amazon Connect Service | [aws-sdk-connect](https://crates.io/crates/aws-sdk-connect) ([docs](https://docs.rs/aws-sdk-connect)) |
| Amazon Connect Wisdom Service | [aws-sdk-wisdom](https://crates.io/crates/aws-sdk-wisdom) ([docs](https://docs.rs/aws-sdk-wisdom)) |
| Amazon Data Lifecycle Manager | [aws-sdk-dlm](https://crates.io/crates/aws-sdk-dlm) ([docs](https://docs.rs/aws-sdk-dlm)) |
| Amazon DataZone | [aws-sdk-datazone](https://crates.io/crates/aws-sdk-datazone) ([docs](https://docs.rs/aws-sdk-datazone)) |
| Amazon Detective | [aws-sdk-detective](https://crates.io/crates/aws-sdk-detective) ([docs](https://docs.rs/aws-sdk-detective)) |
| Amazon DevOps Guru | [aws-sdk-devopsguru](https://crates.io/crates/aws-sdk-devopsguru) ([docs](https://docs.rs/aws-sdk-devopsguru)) |
| Amazon DocumentDB Elastic Clusters | [aws-sdk-docdbelastic](https://crates.io/crates/aws-sdk-docdbelastic) ([docs](https://docs.rs/aws-sdk-docdbelastic)) |
| Amazon DocumentDB with MongoDB compatibility | [aws-sdk-docdb](https://crates.io/crates/aws-sdk-docdb) ([docs](https://docs.rs/aws-sdk-docdb)) |
| Amazon DynamoDB | [aws-sdk-dynamodb](https://crates.io/crates/aws-sdk-dynamodb) ([docs](https://docs.rs/aws-sdk-dynamodb)) |
| Amazon DynamoDB Accelerator (DAX) | [aws-sdk-dax](https://crates.io/crates/aws-sdk-dax) ([docs](https://docs.rs/aws-sdk-dax)) |
| Amazon DynamoDB Streams | [aws-sdk-dynamodbstreams](https://crates.io/crates/aws-sdk-dynamodbstreams) ([docs](https://docs.rs/aws-sdk-dynamodbstreams)) |
| Amazon EC2 Container Registry | [aws-sdk-ecr](https://crates.io/crates/aws-sdk-ecr) ([docs](https://docs.rs/aws-sdk-ecr)) |
| Amazon EC2 Container Service | [aws-sdk-ecs](https://crates.io/crates/aws-sdk-ecs) ([docs](https://docs.rs/aws-sdk-ecs)) |
| Amazon EKS Auth | [aws-sdk-eksauth](https://crates.io/crates/aws-sdk-eksauth) ([docs](https://docs.rs/aws-sdk-eksauth)) |
| Amazon EMR | [aws-sdk-emr](https://crates.io/crates/aws-sdk-emr) ([docs](https://docs.rs/aws-sdk-emr)) |
| Amazon EMR Containers | [aws-sdk-emrcontainers](https://crates.io/crates/aws-sdk-emrcontainers) ([docs](https://docs.rs/aws-sdk-emrcontainers)) |
| Amazon ElastiCache | [aws-sdk-elasticache](https://crates.io/crates/aws-sdk-elasticache) ([docs](https://docs.rs/aws-sdk-elasticache)) |
| Amazon Elastic  Inference | [aws-sdk-elasticinference](https://crates.io/crates/aws-sdk-elasticinference) ([docs](https://docs.rs/aws-sdk-elasticinference)) |
| Amazon Elastic Block Store | [aws-sdk-ebs](https://crates.io/crates/aws-sdk-ebs) ([docs](https://docs.rs/aws-sdk-ebs)) |
| Amazon Elastic Compute Cloud | [aws-sdk-ec2](https://crates.io/crates/aws-sdk-ec2) ([docs](https://docs.rs/aws-sdk-ec2)) |
| Amazon Elastic Container Registry Public | [aws-sdk-ecrpublic](https://crates.io/crates/aws-sdk-ecrpublic) ([docs](https://docs.rs/aws-sdk-ecrpublic)) |
| Amazon Elastic File System | [aws-sdk-efs](https://crates.io/crates/aws-sdk-efs) ([docs](https://docs.rs/aws-sdk-efs)) |
| Amazon Elastic Kubernetes Service | [aws-sdk-eks](https://crates.io/crates/aws-sdk-eks) ([docs](https://docs.rs/aws-sdk-eks)) |
| Amazon Elastic Transcoder | [aws-sdk-elastictranscoder](https://crates.io/crates/aws-sdk-elastictranscoder) ([docs](https://docs.rs/aws-sdk-elastictranscoder)) |
| Amazon Elasticsearch Service | [aws-sdk-elasticsearch](https://crates.io/crates/aws-sdk-elasticsearch) ([docs](https://docs.rs/aws-sdk-elasticsearch)) |
| Amazon EventBridge | [aws-sdk-eventbridge](https://crates.io/crates/aws-sdk-eventbridge) ([docs](https://docs.rs/aws-sdk-eventbridge)) |
| Amazon EventBridge Pipes | [aws-sdk-pipes](https://crates.io/crates/aws-sdk-pipes) ([docs](https://docs.rs/aws-sdk-pipes)) |
| Amazon EventBridge Scheduler | [aws-sdk-scheduler](https://crates.io/crates/aws-sdk-scheduler) ([docs](https://docs.rs/aws-sdk-scheduler)) |
| Amazon FSx | [aws-sdk-fsx](https://crates.io/crates/aws-sdk-fsx) ([docs](https://docs.rs/aws-sdk-fsx)) |
| Amazon Forecast Query Service | [aws-sdk-forecastquery](https://crates.io/crates/aws-sdk-forecastquery) ([docs](https://docs.rs/aws-sdk-forecastquery)) |
| Amazon Forecast Service | [aws-sdk-forecast](https://crates.io/crates/aws-sdk-forecast) ([docs](https://docs.rs/aws-sdk-forecast)) |
| Amazon Fraud Detector | [aws-sdk-frauddetector](https://crates.io/crates/aws-sdk-frauddetector) ([docs](https://docs.rs/aws-sdk-frauddetector)) |
| Amazon GameLift | [aws-sdk-gamelift](https://crates.io/crates/aws-sdk-gamelift) ([docs](https://docs.rs/aws-sdk-gamelift)) |
| Amazon Glacier | [aws-sdk-glacier](https://crates.io/crates/aws-sdk-glacier) ([docs](https://docs.rs/aws-sdk-glacier)) |
| Amazon GuardDuty | [aws-sdk-guardduty](https://crates.io/crates/aws-sdk-guardduty) ([docs](https://docs.rs/aws-sdk-guardduty)) |
| Amazon HealthLake | [aws-sdk-healthlake](https://crates.io/crates/aws-sdk-healthlake) ([docs](https://docs.rs/aws-sdk-healthlake)) |
| Amazon Honeycode | [aws-sdk-honeycode](https://crates.io/crates/aws-sdk-honeycode) ([docs](https://docs.rs/aws-sdk-honeycode)) |
| Amazon Import/Export Snowball | [aws-sdk-snowball](https://crates.io/crates/aws-sdk-snowball) ([docs](https://docs.rs/aws-sdk-snowball)) |
| Amazon Inspector | [aws-sdk-inspector](https://crates.io/crates/aws-sdk-inspector) ([docs](https://docs.rs/aws-sdk-inspector)) |
| Amazon Interactive Video Service | [aws-sdk-ivs](https://crates.io/crates/aws-sdk-ivs) ([docs](https://docs.rs/aws-sdk-ivs)) |
| Amazon Interactive Video Service Chat | [aws-sdk-ivschat](https://crates.io/crates/aws-sdk-ivschat) ([docs](https://docs.rs/aws-sdk-ivschat)) |
| Amazon Interactive Video Service RealTime | [aws-sdk-ivsrealtime](https://crates.io/crates/aws-sdk-ivsrealtime) ([docs](https://docs.rs/aws-sdk-ivsrealtime)) |
| Amazon Kendra Intelligent Ranking | [aws-sdk-kendraranking](https://crates.io/crates/aws-sdk-kendraranking) ([docs](https://docs.rs/aws-sdk-kendraranking)) |
| Amazon Keyspaces | [aws-sdk-keyspaces](https://crates.io/crates/aws-sdk-keyspaces) ([docs](https://docs.rs/aws-sdk-keyspaces)) |
| Amazon Kinesis | [aws-sdk-kinesis](https://crates.io/crates/aws-sdk-kinesis) ([docs](https://docs.rs/aws-sdk-kinesis)) |
| Amazon Kinesis Analytics | [aws-sdk-kinesisanalytics](https://crates.io/crates/aws-sdk-kinesisanalytics) ([docs](https://docs.rs/aws-sdk-kinesisanalytics)) |
| Amazon Kinesis Analytics | [aws-sdk-kinesisanalyticsv2](https://crates.io/crates/aws-sdk-kinesisanalyticsv2) ([docs](https://docs.rs/aws-sdk-kinesisanalyticsv2)) |
| Amazon Kinesis Firehose | [aws-sdk-firehose](https://crates.io/crates/aws-sdk-firehose) ([docs](https://docs.rs/aws-sdk-firehose)) |
| Amazon Kinesis Video Signaling Channels | [aws-sdk-kinesisvideosignaling](https://crates.io/crates/aws-sdk-kinesisvideosignaling) ([docs](https://docs.rs/aws-sdk-kinesisvideosignaling)) |
| Amazon Kinesis Video Streams | [aws-sdk-kinesisvideo](https://crates.io/crates/aws-sdk-kinesisvideo) ([docs](https://docs.rs/aws-sdk-kinesisvideo)) |
| Amazon Kinesis Video Streams Archived Media | [aws-sdk-kinesisvideoarchivedmedia](https://crates.io/crates/aws-sdk-kinesisvideoarchivedmedia) ([docs](https://docs.rs/aws-sdk-kinesisvideoarchivedmedia)) |
| Amazon Kinesis Video Streams Media | [aws-sdk-kinesisvideomedia](https://crates.io/crates/aws-sdk-kinesisvideomedia) ([docs](https://docs.rs/aws-sdk-kinesisvideomedia)) |
| Amazon Kinesis Video WebRTC Storage | [aws-sdk-kinesisvideowebrtcstorage](https://crates.io/crates/aws-sdk-kinesisvideowebrtcstorage) ([docs](https://docs.rs/aws-sdk-kinesisvideowebrtcstorage)) |
| Amazon Lex Model Building Service | [aws-sdk-lexmodelbuilding](https://crates.io/crates/aws-sdk-lexmodelbuilding) ([docs](https://docs.rs/aws-sdk-lexmodelbuilding)) |
| Amazon Lex Model Building V2 | [aws-sdk-lexmodelsv2](https://crates.io/crates/aws-sdk-lexmodelsv2) ([docs](https://docs.rs/aws-sdk-lexmodelsv2)) |
| Amazon Lex Runtime Service | [aws-sdk-lexruntime](https://crates.io/crates/aws-sdk-lexruntime) ([docs](https://docs.rs/aws-sdk-lexruntime)) |
| Amazon Lex Runtime V2 | [aws-sdk-lexruntimev2](https://crates.io/crates/aws-sdk-lexruntimev2) ([docs](https://docs.rs/aws-sdk-lexruntimev2)) |
| Amazon Lightsail | [aws-sdk-lightsail](https://crates.io/crates/aws-sdk-lightsail) ([docs](https://docs.rs/aws-sdk-lightsail)) |
| Amazon Location Service | [aws-sdk-location](https://crates.io/crates/aws-sdk-location) ([docs](https://docs.rs/aws-sdk-location)) |
| Amazon Lookout for Equipment | [aws-sdk-lookoutequipment](https://crates.io/crates/aws-sdk-lookoutequipment) ([docs](https://docs.rs/aws-sdk-lookoutequipment)) |
| Amazon Lookout for Metrics | [aws-sdk-lookoutmetrics](https://crates.io/crates/aws-sdk-lookoutmetrics) ([docs](https://docs.rs/aws-sdk-lookoutmetrics)) |
| Amazon Lookout for Vision | [aws-sdk-lookoutvision](https://crates.io/crates/aws-sdk-lookoutvision) ([docs](https://docs.rs/aws-sdk-lookoutvision)) |
| Amazon Machine Learning | [aws-sdk-machinelearning](https://crates.io/crates/aws-sdk-machinelearning) ([docs](https://docs.rs/aws-sdk-machinelearning)) |
| Amazon Macie 2 | [aws-sdk-macie2](https://crates.io/crates/aws-sdk-macie2) ([docs](https://docs.rs/aws-sdk-macie2)) |
| Amazon Managed Blockchain | [aws-sdk-managedblockchain](https://crates.io/crates/aws-sdk-managedblockchain) ([docs](https://docs.rs/aws-sdk-managedblockchain)) |
| Amazon Managed Blockchain Query | [aws-sdk-managedblockchainquery](https://crates.io/crates/aws-sdk-managedblockchainquery) ([docs](https://docs.rs/aws-sdk-managedblockchainquery)) |
| Amazon Managed Grafana | [aws-sdk-grafana](https://crates.io/crates/aws-sdk-grafana) ([docs](https://docs.rs/aws-sdk-grafana)) |
| Amazon Mechanical Turk | [aws-sdk-mturk](https://crates.io/crates/aws-sdk-mturk) ([docs](https://docs.rs/aws-sdk-mturk)) |
| Amazon MemoryDB | [aws-sdk-memorydb](https://crates.io/crates/aws-sdk-memorydb) ([docs](https://docs.rs/aws-sdk-memorydb)) |
| Amazon Neptune | [aws-sdk-neptune](https://crates.io/crates/aws-sdk-neptune) ([docs](https://docs.rs/aws-sdk-neptune)) |
| Amazon Neptune Graph | [aws-sdk-neptunegraph](https://crates.io/crates/aws-sdk-neptunegraph) ([docs](https://docs.rs/aws-sdk-neptunegraph)) |
| Amazon NeptuneData | [aws-sdk-neptunedata](https://crates.io/crates/aws-sdk-neptunedata) ([docs](https://docs.rs/aws-sdk-neptunedata)) |
| Amazon Omics | [aws-sdk-omics](https://crates.io/crates/aws-sdk-omics) ([docs](https://docs.rs/aws-sdk-omics)) |
| Amazon OpenSearch Ingestion | [aws-sdk-osis](https://crates.io/crates/aws-sdk-osis) ([docs](https://docs.rs/aws-sdk-osis)) |
| Amazon OpenSearch Service | [aws-sdk-opensearch](https://crates.io/crates/aws-sdk-opensearch) ([docs](https://docs.rs/aws-sdk-opensearch)) |
| Amazon Personalize | [aws-sdk-personalize](https://crates.io/crates/aws-sdk-personalize) ([docs](https://docs.rs/aws-sdk-personalize)) |
| Amazon Personalize Events | [aws-sdk-personalizeevents](https://crates.io/crates/aws-sdk-personalizeevents) ([docs](https://docs.rs/aws-sdk-personalizeevents)) |
| Amazon Personalize Runtime | [aws-sdk-personalizeruntime](https://crates.io/crates/aws-sdk-personalizeruntime) ([docs](https://docs.rs/aws-sdk-personalizeruntime)) |
| Amazon Pinpoint | [aws-sdk-pinpoint](https://crates.io/crates/aws-sdk-pinpoint) ([docs](https://docs.rs/aws-sdk-pinpoint)) |
| Amazon Pinpoint Email Service | [aws-sdk-pinpointemail](https://crates.io/crates/aws-sdk-pinpointemail) ([docs](https://docs.rs/aws-sdk-pinpointemail)) |
| Amazon Pinpoint SMS Voice V2 | [aws-sdk-pinpointsmsvoicev2](https://crates.io/crates/aws-sdk-pinpointsmsvoicev2) ([docs](https://docs.rs/aws-sdk-pinpointsmsvoicev2)) |
| Amazon Pinpoint SMS and Voice Service | [aws-sdk-pinpointsmsvoice](https://crates.io/crates/aws-sdk-pinpointsmsvoice) ([docs](https://docs.rs/aws-sdk-pinpointsmsvoice)) |
| Amazon Polly | [aws-sdk-polly](https://crates.io/crates/aws-sdk-polly) ([docs](https://docs.rs/aws-sdk-polly)) |
| Amazon Prometheus Service | [aws-sdk-amp](https://crates.io/crates/aws-sdk-amp) ([docs](https://docs.rs/aws-sdk-amp)) |
| Amazon Q Connect | [aws-sdk-qconnect](https://crates.io/crates/aws-sdk-qconnect) ([docs](https://docs.rs/aws-sdk-qconnect)) |
| Amazon QLDB | [aws-sdk-qldb](https://crates.io/crates/aws-sdk-qldb) ([docs](https://docs.rs/aws-sdk-qldb)) |
| Amazon QLDB Session | [aws-sdk-qldbsession](https://crates.io/crates/aws-sdk-qldbsession) ([docs](https://docs.rs/aws-sdk-qldbsession)) |
| Amazon QuickSight | [aws-sdk-quicksight](https://crates.io/crates/aws-sdk-quicksight) ([docs](https://docs.rs/aws-sdk-quicksight)) |
| Amazon Recycle Bin | [aws-sdk-rbin](https://crates.io/crates/aws-sdk-rbin) ([docs](https://docs.rs/aws-sdk-rbin)) |
| Amazon Redshift | [aws-sdk-redshift](https://crates.io/crates/aws-sdk-redshift) ([docs](https://docs.rs/aws-sdk-redshift)) |
| Amazon Rekognition | [aws-sdk-rekognition](https://crates.io/crates/aws-sdk-rekognition) ([docs](https://docs.rs/aws-sdk-rekognition)) |
| Amazon Relational Database Service | [aws-sdk-rds](https://crates.io/crates/aws-sdk-rds) ([docs](https://docs.rs/aws-sdk-rds)) |
| Amazon Route 53 | [aws-sdk-route53](https://crates.io/crates/aws-sdk-route53) ([docs](https://docs.rs/aws-sdk-route53)) |
| Amazon Route 53 Domains | [aws-sdk-route53domains](https://crates.io/crates/aws-sdk-route53domains) ([docs](https://docs.rs/aws-sdk-route53domains)) |
| Amazon Route 53 Resolver | [aws-sdk-route53resolver](https://crates.io/crates/aws-sdk-route53resolver) ([docs](https://docs.rs/aws-sdk-route53resolver)) |
| Amazon S3 on Outposts | [aws-sdk-s3outposts](https://crates.io/crates/aws-sdk-s3outposts) ([docs](https://docs.rs/aws-sdk-s3outposts)) |
| Amazon SageMaker Feature Store Runtime | [aws-sdk-sagemakerfeaturestoreruntime](https://crates.io/crates/aws-sdk-sagemakerfeaturestoreruntime) ([docs](https://docs.rs/aws-sdk-sagemakerfeaturestoreruntime)) |
| Amazon SageMaker Metrics Service | [aws-sdk-sagemakermetrics](https://crates.io/crates/aws-sdk-sagemakermetrics) ([docs](https://docs.rs/aws-sdk-sagemakermetrics)) |
| Amazon SageMaker Runtime | [aws-sdk-sagemakerruntime](https://crates.io/crates/aws-sdk-sagemakerruntime) ([docs](https://docs.rs/aws-sdk-sagemakerruntime)) |
| Amazon SageMaker Service | [aws-sdk-sagemaker](https://crates.io/crates/aws-sdk-sagemaker) ([docs](https://docs.rs/aws-sdk-sagemaker)) |
| Amazon SageMaker geospatial capabilities | [aws-sdk-sagemakergeospatial](https://crates.io/crates/aws-sdk-sagemakergeospatial) ([docs](https://docs.rs/aws-sdk-sagemakergeospatial)) |
| Amazon Sagemaker Edge Manager | [aws-sdk-sagemakeredge](https://crates.io/crates/aws-sdk-sagemakeredge) ([docs](https://docs.rs/aws-sdk-sagemakeredge)) |
| Amazon Security Lake | [aws-sdk-securitylake](https://crates.io/crates/aws-sdk-securitylake) ([docs](https://docs.rs/aws-sdk-securitylake)) |
| Amazon Simple Email Service | [aws-sdk-ses](https://crates.io/crates/aws-sdk-ses) ([docs](https://docs.rs/aws-sdk-ses)) |
| Amazon Simple Email Service | [aws-sdk-sesv2](https://crates.io/crates/aws-sdk-sesv2) ([docs](https://docs.rs/aws-sdk-sesv2)) |
| Amazon Simple Notification Service | [aws-sdk-sns](https://crates.io/crates/aws-sdk-sns) ([docs](https://docs.rs/aws-sdk-sns)) |
| Amazon Simple Queue Service | [aws-sdk-sqs](https://crates.io/crates/aws-sdk-sqs) ([docs](https://docs.rs/aws-sdk-sqs)) |
| Amazon Simple Storage Service | [aws-sdk-s3](https://crates.io/crates/aws-sdk-s3) ([docs](https://docs.rs/aws-sdk-s3)) |
| Amazon Simple Systems Manager (SSM) | [aws-sdk-ssm](https://crates.io/crates/aws-sdk-ssm) ([docs](https://docs.rs/aws-sdk-ssm)) |
| Amazon Simple Workflow Service | [aws-sdk-swf](https://crates.io/crates/aws-sdk-swf) ([docs](https://docs.rs/aws-sdk-swf)) |
| Amazon Textract | [aws-sdk-textract](https://crates.io/crates/aws-sdk-textract) ([docs](https://docs.rs/aws-sdk-textract)) |
| Amazon Timestream Query | [aws-sdk-timestreamquery](https://crates.io/crates/aws-sdk-timestreamquery) ([docs](https://docs.rs/aws-sdk-timestreamquery)) |
| Amazon Timestream Write | [aws-sdk-timestreamwrite](https://crates.io/crates/aws-sdk-timestreamwrite) ([docs](https://docs.rs/aws-sdk-timestreamwrite)) |
| Amazon Transcribe Service | [aws-sdk-transcribe](https://crates.io/crates/aws-sdk-transcribe) ([docs](https://docs.rs/aws-sdk-transcribe)) |
| Amazon Transcribe Streaming Service | [aws-sdk-transcribestreaming](https://crates.io/crates/aws-sdk-transcribestreaming) ([docs](https://docs.rs/aws-sdk-transcribestreaming)) |
| Amazon Translate | [aws-sdk-translate](https://crates.io/crates/aws-sdk-translate) ([docs](https://docs.rs/aws-sdk-translate)) |
| Amazon VPC Lattice | [aws-sdk-vpclattice](https://crates.io/crates/aws-sdk-vpclattice) ([docs](https://docs.rs/aws-sdk-vpclattice)) |
| Amazon Verified Permissions | [aws-sdk-verifiedpermissions](https://crates.io/crates/aws-sdk-verifiedpermissions) ([docs](https://docs.rs/aws-sdk-verifiedpermissions)) |
| Amazon Voice ID | [aws-sdk-voiceid](https://crates.io/crates/aws-sdk-voiceid) ([docs](https://docs.rs/aws-sdk-voiceid)) |
| Amazon WorkDocs | [aws-sdk-workdocs](https://crates.io/crates/aws-sdk-workdocs) ([docs](https://docs.rs/aws-sdk-workdocs)) |
| Amazon WorkLink | [aws-sdk-worklink](https://crates.io/crates/aws-sdk-worklink) ([docs](https://docs.rs/aws-sdk-worklink)) |
| Amazon WorkMail | [aws-sdk-workmail](https://crates.io/crates/aws-sdk-workmail) ([docs](https://docs.rs/aws-sdk-workmail)) |
| Amazon WorkMail Message Flow | [aws-sdk-workmailmessageflow](https://crates.io/crates/aws-sdk-workmailmessageflow) ([docs](https://docs.rs/aws-sdk-workmailmessageflow)) |
| Amazon WorkSpaces | [aws-sdk-workspaces](https://crates.io/crates/aws-sdk-workspaces) ([docs](https://docs.rs/aws-sdk-workspaces)) |
| Amazon WorkSpaces Thin Client | [aws-sdk-workspacesthinclient](https://crates.io/crates/aws-sdk-workspacesthinclient) ([docs](https://docs.rs/aws-sdk-workspacesthinclient)) |
| Amazon WorkSpaces Web | [aws-sdk-workspacesweb](https://crates.io/crates/aws-sdk-workspacesweb) ([docs](https://docs.rs/aws-sdk-workspacesweb)) |
| AmazonApiGatewayManagementApi | [aws-sdk-apigatewaymanagement](https://crates.io/crates/aws-sdk-apigatewaymanagement) ([docs](https://docs.rs/aws-sdk-apigatewaymanagement)) |
| AmazonApiGatewayV2 | [aws-sdk-apigatewayv2](https://crates.io/crates/aws-sdk-apigatewayv2) ([docs](https://docs.rs/aws-sdk-apigatewayv2)) |
| AmazonConnectCampaignService | [aws-sdk-connectcampaigns](https://crates.io/crates/aws-sdk-connectcampaigns) ([docs](https://docs.rs/aws-sdk-connectcampaigns)) |
| AmazonMQ | [aws-sdk-mq](https://crates.io/crates/aws-sdk-mq) ([docs](https://docs.rs/aws-sdk-mq)) |
| AmazonMWAA | [aws-sdk-mwaa](https://crates.io/crates/aws-sdk-mwaa) ([docs](https://docs.rs/aws-sdk-mwaa)) |
| AmazonNimbleStudio | [aws-sdk-nimble](https://crates.io/crates/aws-sdk-nimble) ([docs](https://docs.rs/aws-sdk-nimble)) |
| AmplifyBackend | [aws-sdk-amplifybackend](https://crates.io/crates/aws-sdk-amplifybackend) ([docs](https://docs.rs/aws-sdk-amplifybackend)) |
| AppFabric | [aws-sdk-appfabric](https://crates.io/crates/aws-sdk-appfabric) ([docs](https://docs.rs/aws-sdk-appfabric)) |
| Application Auto Scaling | [aws-sdk-applicationautoscaling](https://crates.io/crates/aws-sdk-applicationautoscaling) ([docs](https://docs.rs/aws-sdk-applicationautoscaling)) |
| Application Migration Service | [aws-sdk-mgn](https://crates.io/crates/aws-sdk-mgn) ([docs](https://docs.rs/aws-sdk-mgn)) |
| Auto Scaling | [aws-sdk-autoscaling](https://crates.io/crates/aws-sdk-autoscaling) ([docs](https://docs.rs/aws-sdk-autoscaling)) |
| Braket | [aws-sdk-braket](https://crates.io/crates/aws-sdk-braket) ([docs](https://docs.rs/aws-sdk-braket)) |
| CloudWatch Observability Access Manager | [aws-sdk-oam](https://crates.io/crates/aws-sdk-oam) ([docs](https://docs.rs/aws-sdk-oam)) |
| CloudWatch RUM | [aws-sdk-rum](https://crates.io/crates/aws-sdk-rum) ([docs](https://docs.rs/aws-sdk-rum)) |
| CodeArtifact | [aws-sdk-codeartifact](https://crates.io/crates/aws-sdk-codeartifact) ([docs](https://docs.rs/aws-sdk-codeartifact)) |
| Cost Optimization Hub | [aws-sdk-costoptimizationhub](https://crates.io/crates/aws-sdk-costoptimizationhub) ([docs](https://docs.rs/aws-sdk-costoptimizationhub)) |
| EC2 Image Builder | [aws-sdk-imagebuilder](https://crates.io/crates/aws-sdk-imagebuilder) ([docs](https://docs.rs/aws-sdk-imagebuilder)) |
| EMR Serverless | [aws-sdk-emrserverless](https://crates.io/crates/aws-sdk-emrserverless) ([docs](https://docs.rs/aws-sdk-emrserverless)) |
| Elastic Disaster Recovery Service | [aws-sdk-drs](https://crates.io/crates/aws-sdk-drs) ([docs](https://docs.rs/aws-sdk-drs)) |
| Elastic Load Balancing | [aws-sdk-elasticloadbalancing](https://crates.io/crates/aws-sdk-elasticloadbalancing) ([docs](https://docs.rs/aws-sdk-elasticloadbalancing)) |
| Elastic Load Balancing | [aws-sdk-elasticloadbalancingv2](https://crates.io/crates/aws-sdk-elasticloadbalancingv2) ([docs](https://docs.rs/aws-sdk-elasticloadbalancingv2)) |
| FinSpace Public API | [aws-sdk-finspacedata](https://crates.io/crates/aws-sdk-finspacedata) ([docs](https://docs.rs/aws-sdk-finspacedata)) |
| FinSpace User Environment Management service | [aws-sdk-finspace](https://crates.io/crates/aws-sdk-finspace) ([docs](https://docs.rs/aws-sdk-finspace)) |
| Firewall Management Service | [aws-sdk-fms](https://crates.io/crates/aws-sdk-fms) ([docs](https://docs.rs/aws-sdk-fms)) |
| IAM Roles Anywhere | [aws-sdk-rolesanywhere](https://crates.io/crates/aws-sdk-rolesanywhere) ([docs](https://docs.rs/aws-sdk-rolesanywhere)) |
| Inspector Scan | [aws-sdk-inspectorscan](https://crates.io/crates/aws-sdk-inspectorscan) ([docs](https://docs.rs/aws-sdk-inspectorscan)) |
| Inspector2 | [aws-sdk-inspector2](https://crates.io/crates/aws-sdk-inspector2) ([docs](https://docs.rs/aws-sdk-inspector2)) |
| Managed Streaming for Kafka | [aws-sdk-kafka](https://crates.io/crates/aws-sdk-kafka) ([docs](https://docs.rs/aws-sdk-kafka)) |
| Managed Streaming for Kafka Connect | [aws-sdk-kafkaconnect](https://crates.io/crates/aws-sdk-kafkaconnect) ([docs](https://docs.rs/aws-sdk-kafkaconnect)) |
| Migration Hub Strategy Recommendations | [aws-sdk-migrationhubstrategy](https://crates.io/crates/aws-sdk-migrationhubstrategy) ([docs](https://docs.rs/aws-sdk-migrationhubstrategy)) |
| OpenSearch Service Serverless | [aws-sdk-opensearchserverless](https://crates.io/crates/aws-sdk-opensearchserverless) ([docs](https://docs.rs/aws-sdk-opensearchserverless)) |
| Payment Cryptography Control Plane | [aws-sdk-paymentcryptography](https://crates.io/crates/aws-sdk-paymentcryptography) ([docs](https://docs.rs/aws-sdk-paymentcryptography)) |
| Payment Cryptography Data Plane | [aws-sdk-paymentcryptographydata](https://crates.io/crates/aws-sdk-paymentcryptographydata) ([docs](https://docs.rs/aws-sdk-paymentcryptographydata)) |
| PcaConnectorAd | [aws-sdk-pcaconnectorad](https://crates.io/crates/aws-sdk-pcaconnectorad) ([docs](https://docs.rs/aws-sdk-pcaconnectorad)) |
| QBusiness | [aws-sdk-qbusiness](https://crates.io/crates/aws-sdk-qbusiness) ([docs](https://docs.rs/aws-sdk-qbusiness)) |
| Redshift Data API Service | [aws-sdk-redshiftdata](https://crates.io/crates/aws-sdk-redshiftdata) ([docs](https://docs.rs/aws-sdk-redshiftdata)) |
| Redshift Serverless | [aws-sdk-redshiftserverless](https://crates.io/crates/aws-sdk-redshiftserverless) ([docs](https://docs.rs/aws-sdk-redshiftserverless)) |
| Route53 Recovery Cluster | [aws-sdk-route53recoverycluster](https://crates.io/crates/aws-sdk-route53recoverycluster) ([docs](https://docs.rs/aws-sdk-route53recoverycluster)) |
| Schemas | [aws-sdk-schemas](https://crates.io/crates/aws-sdk-schemas) ([docs](https://docs.rs/aws-sdk-schemas)) |
| Service Quotas | [aws-sdk-servicequotas](https://crates.io/crates/aws-sdk-servicequotas) ([docs](https://docs.rs/aws-sdk-servicequotas)) |
| Synthetics | [aws-sdk-synthetics](https://crates.io/crates/aws-sdk-synthetics) ([docs](https://docs.rs/aws-sdk-synthetics)) |
| TrustedAdvisor Public API | [aws-sdk-trustedadvisor](https://crates.io/crates/aws-sdk-trustedadvisor) ([docs](https://docs.rs/aws-sdk-trustedadvisor)) |
