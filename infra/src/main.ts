import * as dockerBuild from "@pulumi/docker-build"
import * as gcp from "@pulumi/gcp"
import { Config, getStack, interpolate, secret } from "@pulumi/pulumi"
import { RandomId, RandomPassword } from "@pulumi/random"
import { resolve } from "node:path"

// ---------------------------------------------------------------------
// Stack configuration
// ---------------------------------------------------------------------
const config = new Config()
const region = config.require("region")
const project = new Config("gcp").require("project")
const heliusWebhookSecret = config.requireSecret("heliusWebhookSecret")
const solRpcUrl = config.requireSecret("solRpcUrl")
const solPk = config.requireSecret("solPk")

// ---------------------------------------------------------------------
// Google Cloud APIs
// ---------------------------------------------------------------------
type ProjectServices = Record<keyof typeof projectApis, gcp.projects.Service>

const projectApis = {
  run: "run.googleapis.com",
  artifactregistry: "artifactregistry.googleapis.com",
  compute: "compute.googleapis.com",
  sqladmin: "sqladmin.googleapis.com",
  secretmanager: "secretmanager.googleapis.com",
  iam: "iam.googleapis.com",
  pubsub: "pubsub.googleapis.com",
  scheduler: "cloudscheduler.googleapis.com"
}

const projectServices = Object.fromEntries(
  Object.entries(projectApis).map(([name, service]) => [
    name,
    new gcp.projects.Service(name, { service, disableOnDestroy: false })
  ])
) as ProjectServices

// ---------------------------------------------------------------------
// VPC and subnet
// ---------------------------------------------------------------------
const vpc = new gcp.compute.Network(
  "vpc",
  { name: "vpc", autoCreateSubnetworks: false },
  { dependsOn: [projectServices.compute] }
)

const subnet = new gcp.compute.Subnetwork("subnet", {
  name: "subnet",
  network: vpc.id,
  ipCidrRange: "10.0.0.0/24",
  region
})

// ---------------------------------------------------------------------
// Cloud SQL for PostgreSQL
// ---------------------------------------------------------------------
const databaseInstance = new gcp.sql.DatabaseInstance(
  "sql-instance",
  {
    name: "sql-instance",
    databaseVersion: "POSTGRES_16",
    region,
    deletionProtection: false,
    settings: {
      deletionProtectionEnabled: false,
      edition: "ENTERPRISE",
      tier: "db-f1-micro",
      availabilityType: "ZONAL",
      diskSize: 10,
      diskAutoresize: true,
      backupConfiguration: { enabled: false, pointInTimeRecoveryEnabled: false },
      ipConfiguration: {
        ipv4Enabled: false,
        pscConfigs: [{ pscEnabled: true, allowedConsumerProjects: [project] }]
      }
    }
  },
  { dependsOn: [projectServices.sqladmin] }
)

const database = new gcp.sql.Database("app-db", {
  name: "app-db",
  instance: databaseInstance.name
})

const databasePassword = new RandomPassword("db-password", { length: 32, special: false })

const databaseUser = new gcp.sql.User("app-user", {
  name: "app-user",
  instance: databaseInstance.name,
  password: databasePassword.result
})

// ---------------------------------------------------------------------
// Private Service Connect
// ---------------------------------------------------------------------
const databaseAddress = new gcp.compute.Address("db-psc-ip", {
  name: "db-psc-ip",
  region,
  addressType: "INTERNAL",
  subnetwork: subnet.id
})

const databaseEndpoint = new gcp.compute.ForwardingRule("db-psc-endpoint", {
  name: "db-psc-endpoint",
  region,
  network: vpc.id,
  ipAddress: databaseAddress.selfLink,
  loadBalancingScheme: "",
  target: databaseInstance.pscServiceAttachmentLink
})

// ---------------------------------------------------------------------
// Database connection secret
// ---------------------------------------------------------------------
const databaseSecret = new gcp.secretmanager.Secret(
  "database-url",
  { secretId: "database-url", replication: { auto: {} } },
  { dependsOn: [projectServices.secretmanager] }
)

const databaseSecretVersion = new gcp.secretmanager.SecretVersion(
  "database-url-version",
  {
    secret: databaseSecret.id,
    secretData: interpolate`postgresql://${databaseUser.name}:${databasePassword.result}@${databaseAddress.address}:5432/${database.name}`
  },
  { dependsOn: [databaseEndpoint] }
)

// ---------------------------------------------------------------------
// Artifact Registry and container images
// ---------------------------------------------------------------------
const repository = new gcp.artifactregistry.Repository(
  "images-repository",
  { location: region, repositoryId: "images-repository", format: "DOCKER" },
  { dependsOn: [projectServices.artifactregistry] }
)

const workspaceRoot = resolve(__dirname, "../..")
const sharedImageBuildConfig = {
  context: { location: workspaceRoot },
  dockerfile: { location: resolve(workspaceRoot, "docker/Dockerfile") },
  platforms: [dockerBuild.Platform.Linux_amd64],
  buildOnPreview: false
}

// Build the Rust workspace once so the four runtime images can reuse its cache.
const workspaceImage = new dockerBuild.Image("workspace-image", {
  ...sharedImageBuildConfig,
  target: "build",
  exports: [{ cacheonly: {} }],
  push: false
})

const registryAddress = `${region}-docker.pkg.dev`
const clientConfig = gcp.organizations.getClientConfigOutput()

function createImage(
  binary: "api" | "pvault-subscriber" | "pvault-emitter" | "pvault-pusher" | "migration"
) {
  return new dockerBuild.Image(
    `${binary}-image`,
    {
      ...sharedImageBuildConfig,
      target: binary === "migration" ? "migration" : "function",
      buildArgs: binary === "migration" ? undefined : { FUNCTION: binary },
      tags: [
        interpolate`${registryAddress}/${repository.project}/${repository.repositoryId}/${binary}:${getStack()}`
      ],
      registries: [
        {
          address: registryAddress,
          username: "oauth2accesstoken",
          password: secret(clientConfig.accessToken)
        }
      ],
      push: true
    },
    { dependsOn: [workspaceImage] }
  )
}

const images = {
  api: createImage("api"),
  eventSubscriber: createImage("pvault-subscriber"),
  eventEmitter: createImage("pvault-emitter"),
  eventPusher: createImage("pvault-pusher"),
  migration: createImage("migration")
}

// ---------------------------------------------------------------------
// Database migration job
// ---------------------------------------------------------------------
const migrationAccount = new gcp.serviceaccount.Account(
  "migration-sa",
  { accountId: "migration-sa", displayName: "Database migration" },
  { dependsOn: [projectServices.iam] }
)

const grantAccessDbSecretForMigration = new gcp.secretmanager.SecretIamMember(
  "grant-access-for-db-secret-for-migration",
  {
    secretId: databaseSecret.id,
    role: "roles/secretmanager.secretAccessor",
    member: interpolate`serviceAccount:${migrationAccount.email}`
  }
)

const migrationExecution = new RandomId("migration-execution", {
  byteLength: 8,
  keepers: {
    image: images.migration.ref,
    databaseSecretVersion: databaseSecretVersion.name
  }
})

const migration = new gcp.cloudrunv2.Job(
  "migration",
  {
    name: "migration",
    location: region,
    deletionProtection: false,
    // Cloud Run marks the job ready after this execution succeeds.
    runExecutionToken: interpolate`m${migrationExecution.hex}`,
    template: {
      taskCount: 1,
      parallelism: 1,
      template: {
        serviceAccount: migrationAccount.email,
        timeout: "600s",
        maxRetries: 0,
        vpcAccess: {
          egress: "PRIVATE_RANGES_ONLY",
          networkInterfaces: [{ network: vpc.id, subnetwork: subnet.id }]
        },
        containers: [
          {
            image: images.migration.ref,
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
    }
  },
  {
    dependsOn: [projectServices.run, grantAccessDbSecretForMigration, databaseEndpoint],
    customTimeouts: { create: "15m", update: "15m" }
  }
)

// ---------------------------------------------------------------------
// Pub/Sub topic and private subscriber
// ---------------------------------------------------------------------
const pvaultSubscriberAccount = new gcp.serviceaccount.Account(
  "pvault-subscriber-sa",
  {
    accountId: "pvault-subscriber-sa",
    displayName: "pvault-subscriber-sa"
  },
  { dependsOn: [projectServices.iam] }
)

// identity for pvault subscriber caller
const pvaultSubscriberInvokerAccount = new gcp.serviceaccount.Account(
  "pvault-subscriber-invoker-sa",
  {
    accountId: "pvault-subscriber-invoker-sa",
    displayName: "pvault-subscriber-invoker-sa"
  },
  { dependsOn: [projectServices.iam] }
)

const eventTopic = new gcp.pubsub.Topic(
  "pvault-events",
  { name: "pvault-events" },
  { dependsOn: [projectServices.pubsub] }
)

const grantAccessDbSecretForPvaultSubscriber = new gcp.secretmanager.SecretIamMember(
  "grant-access-db-secret-for-pvault-subscriber",
  {
    secretId: databaseSecret.id,
    role: "roles/secretmanager.secretAccessor",
    member: interpolate`serviceAccount:${pvaultSubscriberAccount.email}`
  }
)

const pvaultSubscriber = new gcp.cloudrunv2.Service(
  "pvault-subscriber",
  {
    name: "pvault-subscriber",
    location: region,
    deletionProtection: false,
    ingress: "INGRESS_TRAFFIC_INTERNAL_ONLY",
    invokerIamDisabled: false,
    template: {
      serviceAccount: pvaultSubscriberAccount.email,
      timeout: "300s",
      maxInstanceRequestConcurrency: 1,
      scaling: { minInstanceCount: 0, maxInstanceCount: 1 },
      vpcAccess: {
        egress: "PRIVATE_RANGES_ONLY",
        networkInterfaces: [{ network: vpc.id, subnetwork: subnet.id }]
      },
      containers: [
        {
          image: images.eventSubscriber.ref,
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
    dependsOn: [
      projectServices.run,
      grantAccessDbSecretForPvaultSubscriber,
      databaseEndpoint,
      migration
    ]
  }
)

const grantInvokeForPvaultSubscriberInvoker = new gcp.cloudrunv2.ServiceIamMember(
  "grant-invoke-for-pvault-subscriber-invoker",
  {
    project: pvaultSubscriber.project,
    location: pvaultSubscriber.location,
    name: pvaultSubscriber.name,
    role: "roles/run.invoker",
    member: interpolate`serviceAccount:${pvaultSubscriberInvokerAccount.email}`
  }
)

const eventChannel = new gcp.pubsub.Subscription(
  "pvault-events-channel",
  {
    name: "pvault-events-channel",
    topic: eventTopic.id,
    ackDeadlineSeconds: 300,
    messageRetentionDuration: "604800s",
    expirationPolicy: { ttl: "" },
    retryPolicy: { minimumBackoff: "10s", maximumBackoff: "600s" },
    pushConfig: {
      pushEndpoint: interpolate`${pvaultSubscriber.uri}/`,
      oidcToken: {
        serviceAccountEmail: pvaultSubscriberInvokerAccount.email,
        audience: pvaultSubscriber.uri
      }
    }
  },
  { dependsOn: [grantInvokeForPvaultSubscriberInvoker] }
)

// ---------------------------------------------------------------------
// Pvault event pusher
// ---------------------------------------------------------------------
const pvaultPusherAccount = new gcp.serviceaccount.Account(
  "pvault-pusher-sa",
  { accountId: "pvault-pusher-sa", displayName: "pvault-pusher-sa" },
  { dependsOn: [projectServices.iam] }
)

const webhookSecret = new gcp.secretmanager.Secret(
  "helius-webhook-secret",
  { secretId: "helius-webhook-secret", replication: { auto: {} } },
  { dependsOn: [projectServices.secretmanager] }
)

const webhookSecretVersion = new gcp.secretmanager.SecretVersion("helius-webhook-secret-version", {
  secret: webhookSecret.id,
  secretData: heliusWebhookSecret
})

const grantAccessWebhookSecretForPvaultPusher = new gcp.secretmanager.SecretIamMember(
  "grant-access-webhook-secret-for-pvault-pusher",
  {
    secretId: webhookSecret.id,
    role: "roles/secretmanager.secretAccessor",
    member: interpolate`serviceAccount:${pvaultPusherAccount.email}`
  }
)

const grantPublishForPvaultPusher = new gcp.pubsub.TopicIAMMember(
  "grant-publish-for-pvault-pusher",
  {
    topic: eventTopic.name,
    role: "roles/pubsub.publisher",
    member: interpolate`serviceAccount:${pvaultPusherAccount.email}`
  }
)

const pusherService = new gcp.cloudrunv2.Service(
  "pvault-event-pusher",
  {
    name: "pvault-event-pusher",
    location: region,
    deletionProtection: false,
    ingress: "INGRESS_TRAFFIC_ALL",
    invokerIamDisabled: true,
    template: {
      serviceAccount: pvaultPusherAccount.email,
      scaling: { minInstanceCount: 0, maxInstanceCount: 1 },
      containers: [
        {
          image: images.eventPusher.ref,
          ports: { containerPort: 8080 },
          startupProbe: {
            httpGet: { path: "/", port: 8080 },
            periodSeconds: 5,
            timeoutSeconds: 2,
            failureThreshold: 48
          },
          resources: { limits: { cpu: "1", memory: "512Mi" } },
          envs: [
            {
              name: "HELIUS_WEBHOOK_SECRET",
              valueSource: {
                secretKeyRef: {
                  secret: webhookSecret.secretId,
                  version: webhookSecretVersion.version
                }
              }
            },
            { name: "PVAULT_EVENT_TOPIC", value: eventTopic.id }
          ]
        }
      ]
    }
  },
  {
    dependsOn: [
      projectServices.run,
      eventChannel,
      grantAccessWebhookSecretForPvaultPusher,
      grantPublishForPvaultPusher
    ]
  }
)

// ---------------------------------------------------------------------
// Hourly pvault emitter job
// ---------------------------------------------------------------------
const pvaultEmitterAccount = new gcp.serviceaccount.Account(
  "pvault-emitter-sa",
  { accountId: "pvault-emitter-sa", displayName: "Pvault emitter" },
  { dependsOn: [projectServices.iam] }
)

const pvaultEmitterInvokerAccount = new gcp.serviceaccount.Account(
  "pvault-emitter-invoker-sa",
  { accountId: "pvault-emitter-invoker-sa", displayName: "Pvault emitter scheduler" },
  { dependsOn: [projectServices.iam] }
)

const solRpcUrlSecret = new gcp.secretmanager.Secret(
  "sol-rpc-url",
  { secretId: "sol-rpc-url", replication: { auto: {} } },
  { dependsOn: [projectServices.secretmanager] }
)

const solRpcUrlSecretVersion = new gcp.secretmanager.SecretVersion("sol-rpc-url-version", {
  secret: solRpcUrlSecret.id,
  secretData: solRpcUrl
})

const grantAccessSolRpcUrlForPvaultEmitter = new gcp.secretmanager.SecretIamMember(
  "grant-access-sol-rpc-url-for-pvault-emitter",
  {
    secretId: solRpcUrlSecret.id,
    role: "roles/secretmanager.secretAccessor",
    member: interpolate`serviceAccount:${pvaultEmitterAccount.email}`
  }
)

const solPkSecret = new gcp.secretmanager.Secret(
  "sol-pk",
  { secretId: "sol-pk", replication: { auto: {} } },
  { dependsOn: [projectServices.secretmanager] }
)

const solPkSecretVersion = new gcp.secretmanager.SecretVersion("sol-pk-version", {
  secret: solPkSecret.id,
  secretData: solPk
})

const grantAccessSolPkForPvaultEmitter = new gcp.secretmanager.SecretIamMember(
  "grant-access-sol-pk-for-pvault-emitter",
  {
    secretId: solPkSecret.id,
    role: "roles/secretmanager.secretAccessor",
    member: interpolate`serviceAccount:${pvaultEmitterAccount.email}`
  }
)

const pvaultEmitter = new gcp.cloudrunv2.Job(
  "pvault-emitter",
  {
    name: "pvault-emitter",
    location: region,
    deletionProtection: false,
    template: {
      taskCount: 1,
      parallelism: 1,
      template: {
        serviceAccount: pvaultEmitterAccount.email,
        timeout: "300s",
        maxRetries: 0,
        containers: [
          {
            image: images.eventEmitter.ref,
            resources: { limits: { cpu: "1", memory: "512Mi" } },
            envs: [
              {
                name: "SOL_RPC_URL",
                valueSource: {
                  secretKeyRef: {
                    secret: solRpcUrlSecret.secretId,
                    version: solRpcUrlSecretVersion.version
                  }
                }
              },
              {
                name: "SOL_PK",
                valueSource: {
                  secretKeyRef: {
                    secret: solPkSecret.secretId,
                    version: solPkSecretVersion.version
                  }
                }
              }
            ]
          }
        ]
      }
    }
  },
  {
    dependsOn: [
      projectServices.run,
      grantAccessSolRpcUrlForPvaultEmitter,
      grantAccessSolPkForPvaultEmitter
    ]
  }
)

const grantInvokeForPvaultEmitterInvoker = new gcp.cloudrunv2.JobIamMember(
  "grant-invoke-for-pvault-emitter-invoker",
  {
    project: pvaultEmitter.project,
    location: pvaultEmitter.location,
    name: pvaultEmitter.name,
    role: "roles/run.invoker",
    member: interpolate`serviceAccount:${pvaultEmitterInvokerAccount.email}`
  }
)

new gcp.cloudscheduler.Job(
  "pvault-emitter-hourly",
  {
    name: "pvault-emitter-hourly",
    region,
    schedule: "0 * * * *",
    timeZone: "Etc/UTC",
    retryConfig: { retryCount: 0 },
    httpTarget: {
      httpMethod: "POST",
      uri: interpolate`https://run.googleapis.com/v2/projects/${pvaultEmitter.project}/locations/${pvaultEmitter.location}/jobs/${pvaultEmitter.name}:run`,
      headers: { "Content-Type": "application/json" },
      body: Buffer.from("{}").toString("base64"),
      oauthToken: { serviceAccountEmail: pvaultEmitterInvokerAccount.email }
    }
  },
  { dependsOn: [projectServices.scheduler, grantInvokeForPvaultEmitterInvoker] }
)

// ---------------------------------------------------------------------
// Public API service
// ---------------------------------------------------------------------
const apiAccount = new gcp.serviceaccount.Account(
  "api-sa",
  { accountId: "api-sa", displayName: "api-sa" },
  { dependsOn: [projectServices.iam] }
)

const grantAccessDbSecretForApi = new gcp.secretmanager.SecretIamMember(
  "grant-access-db-secret-for-api",
  {
    secretId: databaseSecret.id,
    role: "roles/secretmanager.secretAccessor",
    member: interpolate`serviceAccount:${apiAccount.email}`
  }
)

const apiService = new gcp.cloudrunv2.Service(
  "api",
  {
    name: "api",
    location: region,
    deletionProtection: false,
    ingress: "INGRESS_TRAFFIC_ALL",
    invokerIamDisabled: true,
    template: {
      serviceAccount: apiAccount.email,
      scaling: { minInstanceCount: 0, maxInstanceCount: 1 },
      vpcAccess: {
        egress: "PRIVATE_RANGES_ONLY",
        networkInterfaces: [{ network: vpc.id, subnetwork: subnet.id }]
      },
      containers: [
        {
          image: images.api.ref,
          ports: { containerPort: 8080 },
          startupProbe: {
            httpGet: { path: "/", port: 8080 },
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
    dependsOn: [projectServices.run, migration, grantAccessDbSecretForApi]
  }
)

export const apiUrl = apiService.uri
export const heliusWebhookUrl = interpolate`${pusherService.uri}/helius/pvault`
