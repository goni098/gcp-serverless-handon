import type * as dockerBuild from "@pulumi/docker-build"

import * as gcp from "@pulumi/gcp"
import { interpolate } from "@pulumi/pulumi"

import { project, region } from "../config"
import { resourceName } from "../naming"
import { subnet, vpc } from "../network"
import { iamService, pubsubService, runService } from "../service"

interface PubsubServiceArgs {
  image: dockerBuild.Image
  databasePscEndpoint: gcp.compute.ForwardingRule
  databaseSecret: gcp.secretmanager.Secret
  databaseSecretVersion: gcp.secretmanager.SecretVersion
  migrationJob: gcp.cloudrunv2.Job
}

export function createPubsubCloudRun({
  image,
  databasePscEndpoint,
  databaseSecret,
  databaseSecretVersion,
  migrationJob
}: PubsubServiceArgs) {
  const serviceAccount = new gcp.serviceaccount.Account(
    resourceName("cleanup-sa", 30),
    {
      accountId: resourceName("cleanup-sa", 30),
      displayName: resourceName("cleanup-sa", 30),
      description:
        "Runs the Cloud Run cleanup subscriber and accesses the database connection secret."
    },
    { dependsOn: [iamService] }
  )

  const pushAccount = new gcp.serviceaccount.Account(
    resourceName("push-sa", 30),
    {
      accountId: resourceName("push-sa", 30),
      displayName: resourceName("push-sa", 30),
      description: "Authenticates Pub/Sub push requests to the Cloud Run cleanup subscriber."
    },
    { dependsOn: [iamService] }
  )

  const topic = new gcp.pubsub.Topic(
    resourceName("cleanup-topic"),
    { name: resourceName("cleanup-topic") },
    { dependsOn: [pubsubService] }
  )

  const secretAccess = new gcp.secretmanager.SecretIamMember(resourceName("cleanup-db-access"), {
    secretId: databaseSecret.id,
    role: "roles/secretmanager.secretAccessor",
    member: interpolate`serviceAccount:${serviceAccount.email}`
  })

  const service = new gcp.cloudrunv2.Service(
    resourceName("cleanup"),
    {
      name: resourceName("cleanup"),
      location: region,
      deletionProtection: false,
      invokerIamDisabled: false,
      template: {
        serviceAccount: serviceAccount.email,
        timeout: "300s",
        maxInstanceRequestConcurrency: 1,
        scaling: { minInstanceCount: 0, maxInstanceCount: 1 },
        vpcAccess: {
          egress: "PRIVATE_RANGES_ONLY",
          networkInterfaces: [{ network: vpc.id, subnetwork: subnet.id }]
        },
        containers: [
          {
            image: image.ref,
            ports: { containerPort: 8080 },
            startupProbe: {
              tcpSocket: { port: 8080 },
              periodSeconds: 5,
              timeoutSeconds: 2,
              failureThreshold: 48
            },
            resources: { limits: { cpu: "1", memory: "512Mi" } },
            envs: [
              {
                name: "DATABASE_URL",
                valueSource: {
                  secretKeyRef: {
                    secret: databaseSecret.secretId,
                    version: databaseSecretVersion.version
                  }
                }
              }
            ]
          }
        ]
      }
    },
    {
      dependsOn: [runService, secretAccess, databasePscEndpoint, migrationJob]
    }
  )

  const invoker = new gcp.cloudrunv2.ServiceIamMember(resourceName("cleanup-push-invoker"), {
    project: service.project,
    location: service.location,
    name: service.name,
    role: "roles/run.invoker",
    member: interpolate`serviceAccount:${pushAccount.email}`
  })
  const pubsubIdentity = new gcp.projects.ServiceIdentity(
    resourceName("pubsub-identity"),
    { project, service: "pubsub.googleapis.com" },
    { dependsOn: [pubsubService] }
  )
  const tokenCreator = new gcp.serviceaccount.IAMMember(resourceName("push-token-creator"), {
    serviceAccountId: pushAccount.name,
    role: "roles/iam.serviceAccountTokenCreator",
    member: pubsubIdentity.member
  })

  const subscription = new gcp.pubsub.Subscription(
    resourceName("cleanup-push"),
    {
      name: resourceName("cleanup-push"),
      topic: topic.id,
      ackDeadlineSeconds: 300,
      messageRetentionDuration: "604800s",
      expirationPolicy: { ttl: "" },
      retryPolicy: { minimumBackoff: "10s", maximumBackoff: "600s" },
      pushConfig: {
        pushEndpoint: interpolate`${service.uri}/`,
        oidcToken: { serviceAccountEmail: pushAccount.email, audience: service.uri }
      }
    },
    { dependsOn: [invoker, tokenCreator] }
  )

  return { service, topic, subscription }
}
