import * as cdk from 'aws-cdk-lib';
import * as ses from 'aws-cdk-lib/aws-ses';
import * as sesActions from 'aws-cdk-lib/aws-ses-actions';
import * as sns from 'aws-cdk-lib/aws-sns';
import * as snsSubscriptions from 'aws-cdk-lib/aws-sns-subscriptions';
import * as s3 from 'aws-cdk-lib/aws-s3';
import * as iam from 'aws-cdk-lib/aws-iam';
import * as route53 from 'aws-cdk-lib/aws-route53';
import { Construct } from 'constructs';

export interface MarkMailInfrastructureStackProps extends cdk.StackProps {
  // カスタムドメインを使用する場合
  domainName?: string;
  // 通知用メールアドレス
  notificationEmail?: string;
}

export class MarkMailInfrastructureStack extends cdk.Stack {
  public readonly sesConfigurationSet: ses.ConfigurationSet;
  public readonly bounceSnsTopic: sns.Topic;
  public readonly complaintSnsTopic: sns.Topic;

  constructor(scope: Construct, id: string, props?: MarkMailInfrastructureStackProps) {
    super(scope, id, props);

    // 環境変数から設定を取得
    const domainName = props?.domainName || process.env.SES_DOMAIN;
    const notificationEmail =
      props?.notificationEmail || process.env.NOTIFICATION_EMAIL || 'admin@example.com';

    // S3バケット（メールのバックアップ用）- オプション
    const emailBucket = new s3.Bucket(this, 'EmailStorageBucket', {
      bucketName: `markmail-emails-${this.account}-${this.region}`,
      versioned: true,
      lifecycleRules: [
        {
          id: 'delete-old-emails',
          expiration: cdk.Duration.days(90),
        },
      ],
      removalPolicy: cdk.RemovalPolicy.RETAIN,
      encryption: s3.BucketEncryption.S3_MANAGED,
    });

    // SNSトピック（バウンス通知用）
    this.bounceSnsTopic = new sns.Topic(this, 'BounceNotificationTopic', {
      topicName: 'markmail-bounce-notifications',
      displayName: 'MarkMail Bounce Notifications',
    });

    // SNSトピック（苦情通知用）
    this.complaintSnsTopic = new sns.Topic(this, 'ComplaintNotificationTopic', {
      topicName: 'markmail-complaint-notifications',
      displayName: 'MarkMail Complaint Notifications',
    });

    // メール通知を追加
    if (notificationEmail) {
      this.bounceSnsTopic.addSubscription(
        new snsSubscriptions.EmailSubscription(notificationEmail)
      );
      this.complaintSnsTopic.addSubscription(
        new snsSubscriptions.EmailSubscription(notificationEmail)
      );
    }

    // SES Configuration Set
    this.sesConfigurationSet = new ses.ConfigurationSet(this, 'ConfigurationSet', {
      configurationSetName: 'markmail-configuration-set',
      // 送信イベントを有効化
      sendingEnabled: true,
    });

    // ドメイン検証（カスタムドメインを使用する場合）
    if (domainName) {
      const emailIdentity = new ses.EmailIdentity(this, 'EmailIdentity', {
        identity: ses.Identity.domain(domainName),
        // DKIM設定
        dkimSigning: true,
      });

      // アウトプット（DNSレコード設定用）
      new cdk.CfnOutput(this, 'DkimRecords', {
        value: 'Check AWS Console for DKIM CNAME records to add to your DNS',
        description: 'DKIM records for domain verification',
      });
    }

    // IAMユーザー（プログラマティックアクセス用）
    const sesUser = new iam.User(this, 'SESUser', {
      userName: 'markmail-ses-user',
    });

    // SES送信ポリシー
    const sesSendPolicy = new iam.PolicyStatement({
      effect: iam.Effect.ALLOW,
      actions: [
        'ses:SendEmail',
        'ses:SendRawEmail',
        'ses:SendTemplatedEmail',
        'ses:SendBulkTemplatedEmail',
        'ses:GetSendQuota',
        'ses:GetSendStatistics',
      ],
      resources: ['*', `arn:aws:ses:${this.region}:${this.account}:identity/*`],
    });

    // バウンス・苦情処理用ポリシー
    const sesEventPolicy = new iam.PolicyStatement({
      effect: iam.Effect.ALLOW,
      actions: [
        'ses:PutConfigurationSetEventDestination',
        'ses:GetConfigurationSetEventDestinations',
        'ses:DeleteConfigurationSetEventDestination',
      ],
      resources: [
        `arn:aws:ses:${this.region}:${this.account}:configuration-set/${this.sesConfigurationSet.configurationSetName}`,
      ],
    });

    // SNS読み取りポリシー
    const snsReadPolicy = new iam.PolicyStatement({
      effect: iam.Effect.ALLOW,
      actions: ['sns:Subscribe', 'sns:Unsubscribe', 'sns:ListSubscriptionsByTopic'],
      resources: [this.bounceSnsTopic.topicArn, this.complaintSnsTopic.topicArn],
    });

    // ポリシーをユーザーに付与
    sesUser.addToPolicy(sesSendPolicy);
    sesUser.addToPolicy(sesEventPolicy);
    sesUser.addToPolicy(snsReadPolicy);

    // アクセスキーの作成
    const accessKey = new iam.AccessKey(this, 'SESUserAccessKey', {
      user: sesUser,
    });

    // 受信ルール（オプション - バウンスメールの処理用）
    const receiptRuleSet = new ses.ReceiptRuleSet(this, 'ReceiptRuleSet', {
      receiptRuleSetName: 'markmail-receipt-rules',
    });

    // バウンスメール処理ルール
    new ses.ReceiptRule(this, 'BounceProcessingRule', {
      ruleSet: receiptRuleSet,
      recipients: ['bounce@' + (domainName || 'example.com')],
      actions: [
        // バウンスメールをS3に保存
        new sesActions.S3({
          bucket: emailBucket,
          objectKeyPrefix: 'bounces/',
        }),
        // SNSトピックに通知
        new sesActions.Sns({
          topic: this.bounceSnsTopic,
        }),
      ],
    });

    // Outputs
    new cdk.CfnOutput(this, 'ConfigurationSetName', {
      value: this.sesConfigurationSet.configurationSetName!,
      description: 'SES Configuration Set Name',
      exportName: 'MarkMailConfigurationSetName',
    });

    new cdk.CfnOutput(this, 'BounceTopicArn', {
      value: this.bounceSnsTopic.topicArn,
      description: 'Bounce notification SNS topic ARN',
      exportName: 'MarkMailBounceTopicArn',
    });

    new cdk.CfnOutput(this, 'ComplaintTopicArn', {
      value: this.complaintSnsTopic.topicArn,
      description: 'Complaint notification SNS topic ARN',
      exportName: 'MarkMailComplaintTopicArn',
    });

    new cdk.CfnOutput(this, 'SESUserAccessKeyId', {
      value: accessKey.accessKeyId,
      description: 'Access Key ID for SES user',
    });

    new cdk.CfnOutput(this, 'SESUserSecretAccessKey', {
      value: accessKey.secretAccessKey.unsafeUnwrap(),
      description: 'Secret Access Key for SES user (KEEP THIS SECRET!)',
    });

    new cdk.CfnOutput(this, 'EmailBucketName', {
      value: emailBucket.bucketName,
      description: 'S3 bucket for email storage',
      exportName: 'MarkMailEmailBucketName',
    });

    // デプロイメント後の設定手順
    new cdk.CfnOutput(this, 'NextSteps', {
      value: `
Next steps:
1. Verify your domain or email addresses in AWS SES console
2. Request production access (move out of sandbox) if needed
3. Update your .env file with the access keys and configuration set name
4. Configure DNS records if using custom domain
      `.trim(),
      description: 'Post-deployment configuration steps',
    });
  }
}
