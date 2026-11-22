import type * as dockerBuild from "@pulumi/docker-build"

import * as gcp from "@pulumi/gcp"
import { interpolate } from "@pulumi/pulumi"

import { region } from "../config"
import { resourceName } from "../naming"
import { subnet, vpc } from "../network"
import { iamService, runService } from "../service"

interface ApiServiceArgs {
  databasePscEndpoint: gcp.compute.ForwardingRule
  image: dockerBuild.Image
  databaseSecret: gcp.secretmanager.Secret
  databaseSecretVersion: gcp.secretmanager.SecretVersion
  migrationJob: gcp.cloudrunv2.Job
}

export function createApiCloudRun({
  databasePscEndpoint,
  image,
  databaseSecret,
  databaseSecretVersion,
  migrationJob
}: ApiServiceArgs) {
  const serviceAccount = new gcp.serviceaccount.Account(
    resourceName("api-sa", 30),
    {
      accountId: resourceName("api-sa", 30),
      displayName: resourceName("api-sa", 30),
      description: "Runs the Cloud Run API service and accesses the database connection secret."
    },
    { dependsOn: [iamService] }
  )

  const databaseSecretAccess = new gcp.secretmanager.SecretIamMember(
    resourceName("api-db-access"),
    {
      secretId: databaseSecret.id,
      role: "roles/secretmanager.secretAccessor",
      member: interpolate`serviceAccount:${serviceAccount.email}`
    }
  )

  return new gcp.cloudrunv2.Service(
    resourceName("api"),
    {
      name: resourceName("api"),
      location: region,
      deletionProtection: false,
      ingress: "INGRESS_TRAFFIC_ALL",
      invokerIamDisabled: true,
      template: {
        serviceAccount: serviceAccount.email,
        scaling: {
          minInstanceCount: 0,
          maxInstanceCount: 1
        },
        vpcAccess: {
          egress: "PRIVATE_RANGES_ONLY",
          networkInterfaces: [{ network: vpc.id, subnetwork: subnet.id }]
        },
        containers: [
          {
            image: image.ref,
            ports: { containerPort: 8080 },
            startupProbe: {
              httpGet: { path: "/", port: 8080 },
              periodSeconds: 5,
              timeoutSeconds: 2,
              failureThreshold: 48
            },
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
    },
    {
      dependsOn: [runService, databaseSecretAccess, databasePscEndpoint, migrationJob]
    }
  )
}
