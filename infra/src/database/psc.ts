import * as gcp from "@pulumi/gcp"

import { region } from "../config"
import { resourceName } from "../naming"
import { subnet, vpc } from "../network"

export function createDatabasePsc(databaseInstance: gcp.sql.DatabaseInstance) {
  const databasePscAddress = new gcp.compute.Address(resourceName("db-psc-ip"), {
    name: resourceName("db-psc-ip"),
    region,
    addressType: "INTERNAL",
    subnetwork: subnet.id
  })

  const databasePscEndpoint = new gcp.compute.ForwardingRule(resourceName("db-psc-endpoint"), {
    name: resourceName("db-psc-endpoint"),
    region,
    network: vpc.id,
    ipAddress: databasePscAddress.selfLink,
    loadBalancingScheme: "",
    target: databaseInstance.pscServiceAttachmentLink
  })

  return { databasePscAddress, databasePscEndpoint }
}
