import * as gcp from "@pulumi/gcp"
import { interpolate } from "@pulumi/pulumi"
import { RandomPassword } from "@pulumi/random"

import { resourceName } from "../naming"
import { secretManagerService } from "../service"

interface CreateDatabaseSecretArgs {
  database: gcp.sql.Database
  databaseUser: gcp.sql.User
  databasePassword: RandomPassword
  databasePscAddress: gcp.compute.Address
}

export function createDatabaseSecret({
  database,
  databasePscAddress,
  databasePassword,
  databaseUser
}: CreateDatabaseSecretArgs) {
  const databaseSecret = new gcp.secretmanager.Secret(
    resourceName("database-url"),
    {
      secretId: resourceName("database-url"),
      replication: { auto: {} }
    },
    { dependsOn: [secretManagerService] }
  )

  const databaseSecretVersion = new gcp.secretmanager.SecretVersion(
    resourceName("database-url-version"),
    {
      secret: databaseSecret.id,
      secretData: interpolate`postgresql://${databaseUser.name}:${databasePassword.result}@${databasePscAddress.address}:5432/${database.name}`
    }
  )

  return {
    databaseSecret,
    databaseSecretVersion
  }
}
