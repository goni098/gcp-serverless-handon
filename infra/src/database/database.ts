import * as gcp from "@pulumi/gcp"
import * as random from "@pulumi/random"

import { resourceName } from "../naming"

export function createDatabaseResources(databaseInstance: gcp.sql.DatabaseInstance) {
  const database = new gcp.sql.Database(resourceName("app-db"), {
    name: resourceName("app-db"),
    instance: databaseInstance.name
  })

  const databasePassword = new random.RandomPassword(resourceName("db-password"), {
    length: 32,
    special: false
  })

  const databaseUser = new gcp.sql.User(resourceName("app-user"), {
    name: resourceName("app-user"),
    instance: databaseInstance.name,
    password: databasePassword.result
  })

  return { database, databasePassword, databaseUser }
}
