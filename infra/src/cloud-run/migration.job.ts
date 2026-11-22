import type * as dockerBuild from "@pulumi/docker-build"

import * as gcp from "@pulumi/gcp"
import { interpolate } from "@pulumi/pulumi"
import * as random from "@pulumi/random"

import { region } from "../config"
import { resourceName } from "../naming"
import { subnet, vpc } from "../network"
import { iamService, runService } from "../service"

interface DatabaseMigrationArgs {
  databasePscEndpoint: gcp.compute.ForwardingRule
  image: dockerBuild.Image
  databaseSecret: gcp.secretmanager.Secret
  databaseSecretVersion: gcp.secretmanager.SecretVersion
}

export function createDatabaseMigrationJob({
  databasePscEndpoint,
  image,
  databaseSecret,
  databaseSecretVersion
}: DatabaseMigrationArgs) {
  const serviceAccount = new gcp.serviceaccount.Account(
    resourceName("migration-sa", 30),
    {
      accountId: resourceName("migration-sa", 30),
      displayName: resourceName("migration-sa", 30),
      description:
        "Runs database migrations before application deployment and accesses the database connection secret."
    },
    { dependsOn: [iamService] }
  )

  const secretAccess = new gcp.secretmanager.SecretIamMember(resourceName("migrate-db-access"), {
    secretId: databaseSecret.id,
    role: "roles/secretmanager.secretAccessor",
    member: interpolate`serviceAccount:${serviceAccount.email}`
  })

  // Rotate only when inputs change. A fresh suffix also permits deploying an older image again.
  const executionId = new random.RandomId(resourceName("migrate-execution"), {
    byteLength: 8,
    keepers: {
      image: image.ref,
      databaseSecretVersion: databaseSecretVersion.name
    }
  })

  return new gcp.cloudrunv2.Job(
    resourceName("migrate"),
    {
      name: resourceName("migrate"),
      location: region,
      deletionProtection: false,
      // Unlike startExecutionToken, this waits for successful completion.
      runExecutionToken: interpolate`m${executionId.hex}`,
      template: {
        taskCount: 1,
        parallelism: 1,
        template: {
          serviceAccount: serviceAccount.email,
          timeout: "600s",
          maxRetries: 0,
          vpcAccess: {
            egress: "PRIVATE_RANGES_ONLY",
            networkInterfaces: [{ network: vpc.id, subnetwork: subnet.id }]
          },
          containers: [
            {
              image: image.ref,
              args: ["migration", "apply"],
              resources: {
                limits: { cpu: "1", memory: "512Mi" }
              },
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
      dependsOn: [runService, secretAccess, databasePscEndpoint],
      customTimeouts: { create: "15m", update: "15m" }
    }
  )
}
