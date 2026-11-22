import { createImage } from "./artifact-registry"
import {
  createApiCloudRun,
  createCronJobs,
  createDatabaseMigrationJob,
  createPubsubCloudRun
} from "./cloud-run"
import { createDatabase } from "./database"
import { createDatabaseSecret } from "./secret-manager"

const {
  database,
  databasePscAddress,
  databasePscEndpoint,
  databaseInstance,
  databasePassword,
  databaseUser
} = createDatabase()

const { databaseSecret, databaseSecretVersion } = createDatabaseSecret({
  database,
  databasePscAddress,
  databasePassword,
  databaseUser
})

const migrationJob = createDatabaseMigrationJob({
  databasePscEndpoint,
  image: createImage("db-migration-cli"),
  databaseSecret,
  databaseSecretVersion
})

const apiFn = createApiCloudRun({
  databasePscEndpoint,
  image: createImage("api"),
  databaseSecret,
  databaseSecretVersion,
  migrationJob
})

const pubsub = createPubsubCloudRun({
  image: createImage("pubsub"),
  databasePscEndpoint,
  databaseSecret,
  databaseSecretVersion,
  migrationJob
})

const cron = createCronJobs({
  image: createImage("cron"),
  databasePscEndpoint,
  databaseSecret,
  databaseSecretVersion,
  migrationJob,
  cleanupTopic: pubsub.topic,
  cleanupSubscription: pubsub.subscription
})

export const pubsubCloudRunName = pubsub.service.name
export const cleanupTopicName = pubsub.topic.id
export const cleanupSubscriptionName = pubsub.subscription.name

export const cronCreateDataJobName = cron.createDataJob.name
export const cronQueueCleanupJobName = cron.queueCleanupJob.name
export const cronCreateDataSchedulerJobName = cron.createDataScheduler.name
export const cronDeleteDataSchedulerJobName = cron.deleteDataScheduler.name

export const apiUrl = apiFn.uri
export const apiCloudRunName = apiFn.name
export const databaseMigrationJobName = migrationJob.name

export const databaseInstanceName = databaseInstance.name
export const databaseName = database.name
export const databaseUserName = databaseUser.name
export const databaseEndpointIp = databasePscAddress.address

export const databaseSecretId = databaseSecret.secretId
export const databaseSecretName = databaseSecret.name
