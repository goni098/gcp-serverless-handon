import { createDatabaseResources } from "./database"
import { createDatabaseInstance } from "./instance"
import { createDatabasePsc } from "./psc"

export function createDatabase() {
  const databaseInstance = createDatabaseInstance()
  const { database, databasePassword, databaseUser } = createDatabaseResources(databaseInstance)
  const { databasePscAddress, databasePscEndpoint } = createDatabasePsc(databaseInstance)

  return {
    databaseInstance,
    database,
    databaseUser,
    databasePassword,
    databasePscEndpoint,
    databasePscAddress
  }
}
