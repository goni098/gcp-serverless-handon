import * as gcp from "@pulumi/gcp"

import { project, region } from "../config"
import { resourceName } from "../naming"
import { sqlService } from "../service"

export function createDatabaseInstance() {
  const databaseInstance = new gcp.sql.DatabaseInstance(
    resourceName("sql-instance"),
    {
      name: resourceName("sql-instance"),
      databaseVersion: "POSTGRES_16",
      region,
      deletionProtection: false,
      settings: {
        edition: "ENTERPRISE",
        tier: "db-f1-micro",
        availabilityType: "ZONAL",
        diskSize: 10,
        diskAutoresize: true,
        backupConfiguration: {
          enabled: false,
          pointInTimeRecoveryEnabled: false
        },
        ipConfiguration: {
          ipv4Enabled: false,
          pscConfigs: [{ pscEnabled: true, allowedConsumerProjects: [project] }]
        }
      }
    },
    { dependsOn: [sqlService] }
  )

  return databaseInstance
}
