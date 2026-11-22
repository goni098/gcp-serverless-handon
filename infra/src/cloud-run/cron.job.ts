import type * as dockerBuild from "@pulumi/docker-build"

import * as gcp from "@pulumi/gcp"
import { interpolate } from "@pulumi/pulumi"

import { region } from "../config"
import { resourceName } from "../naming"
import { subnet, vpc } from "../network"
import { iamService, runService, schedulerService } from "../service"

interface CronJobsArgs {
  image: dockerBuild.Image
  databasePscEndpoint: gcp.compute.ForwardingRule
  databaseSecret: gcp.secretmanager.Secret
  databaseSecretVersion: gcp.secretmanager.SecretVersion
  migrationJob: gcp.cloudrunv2.Job
  cleanupTopic: gcp.pubsub.Topic
  cleanupSubscription: gcp.pubsub.Subscription
}

export function createCronJobs({
  image,
  databasePscEndpoint,
  databaseSecret,
  databaseSecretVersion,
  migrationJob,
  cleanupTopic,
  cleanupSubscription
}: CronJobsArgs) {
  const schedulerAccount = new gcp.serviceaccount.Account(
    resourceName("scheduler-sa", 30),
    {
      accountId: resourceName("scheduler-sa", 30),
      displayName: resourceName("scheduler-sa", 30),
      description:
        "Allows Cloud Scheduler to invoke the cake creation and stale cleanup Cloud Run jobs."
    },
    { dependsOn: [iamService] }
  )

  const cronCreateDataServiceAccount = new gcp.serviceaccount.Account(
    resourceName("create-sa", 30),
    {
      accountId: resourceName("create-sa", 30),
      displayName: resourceName("create-sa", 30),
      description:
        "Runs the scheduled cake and fruit creation job and accesses the database connection secret."
    },
    { dependsOn: [iamService] }
  )

  const cronQueueCleanupServiceAccount = new gcp.serviceaccount.Account(
    resourceName("prune-sa", 30),
    {
      accountId: resourceName("prune-sa", 30),
      displayName: resourceName("prune-sa", 30),
      description:
        "Runs the stale cake and fruit deletion job, accesses the database connection secret, and publishes cleanup notifications."
    },
    { dependsOn: [iamService] }
  )

  function createJob(
    name: string,
    command: "cook-cakes" | "destroy-stale-cakes",
    serviceAccount: gcp.serviceaccount.Account
  ) {
    const secretAccess = new gcp.secretmanager.SecretIamMember(resourceName(`${name}-db-access`), {
      secretId: databaseSecret.id,
      role: "roles/secretmanager.secretAccessor",
      member: interpolate`serviceAccount:${serviceAccount.email}`
    })
    const publisher =
      command === "destroy-stale-cakes"
        ? new gcp.pubsub.TopicIAMMember(resourceName("prune-publisher"), {
            topic: cleanupTopic.name,
            role: "roles/pubsub.publisher",
            member: interpolate`serviceAccount:${serviceAccount.email}`
          })
        : undefined

    return new gcp.cloudrunv2.Job(
      resourceName(name),
      {
        name: resourceName(name),
        location: region,
        deletionProtection: false,
        template: {
          taskCount: 1,
          parallelism: 1,
          template: {
            serviceAccount: serviceAccount.email,
            timeout: "600s",
            // Cake creation must not repeat on retry; stale deletion can safely repeat.
            maxRetries: command === "cook-cakes" ? 0 : 3,
            vpcAccess: {
              egress: "PRIVATE_RANGES_ONLY",
              networkInterfaces: [{ network: vpc.id, subnetwork: subnet.id }]
            },
            containers: [
              {
                image: image.ref,
                args: [command],
                envs: [
                  {
                    name: "DATABASE_URL",
                    valueSource: {
                      secretKeyRef: {
                        secret: databaseSecret.secretId,
                        version: databaseSecretVersion.version
                      }
                    }
                  },
                  ...(command === "destroy-stale-cakes"
                    ? [{ name: "CLEANUP_TOPIC", value: cleanupTopic.id }]
                    : [])
                ],
                resources: { limits: { cpu: "1", memory: "512Mi" } }
              }
            ]
          }
        }
      },
      {
        dependsOn: [
          runService,
          secretAccess,
          databasePscEndpoint,
          migrationJob,
          ...(publisher ? [publisher] : [])
        ]
      }
    )
  }

  const createDataJob = createJob("create-data", "cook-cakes", cronCreateDataServiceAccount)
  const queueCleanupJob = createJob(
    "prune-data",
    "destroy-stale-cakes",
    cronQueueCleanupServiceAccount
  )

  function createScheduler(
    name: string,
    schedule: string,
    job: gcp.cloudrunv2.Job,
    description: string
  ) {
    const invoker = new gcp.cloudrunv2.JobIamMember(resourceName(`${name}-invoker`), {
      project: job.project,
      location: job.location,
      name: job.name,
      role: "roles/run.invoker",
      member: interpolate`serviceAccount:${schedulerAccount.email}`
    })
    return new gcp.cloudscheduler.Job(
      resourceName(name),
      {
        name: resourceName(name),
        region,
        description,
        schedule,
        timeZone: "Etc/UTC",
        attemptDeadline: "300s",
        httpTarget: {
          uri: interpolate`https://run.googleapis.com/v2/projects/${job.project}/locations/${job.location}/jobs/${job.name}:run`,
          httpMethod: "POST",
          headers: { "Content-Type": "application/json" },
          body: Buffer.from("{}").toString("base64"),
          oauthToken: {
            serviceAccountEmail: schedulerAccount.email,
            scope: "https://www.googleapis.com/auth/cloud-platform"
          }
        }
      },
      {
        dependsOn: [schedulerService, invoker, cleanupSubscription]
      }
    )
  }

  const createDataScheduler = createScheduler(
    "create-hourly",
    "0 * * * *",
    createDataJob,
    "Cook cakes and fruits at the start of every hour"
  )
  const deleteDataScheduler = createScheduler(
    "prune-six-hourly",
    "0 */6 * * *",
    queueCleanupJob,
    "Delete cakes older than six hours and their fruits, then publish a notification"
  )

  return { createDataJob, queueCleanupJob, createDataScheduler, deleteDataScheduler }
}
