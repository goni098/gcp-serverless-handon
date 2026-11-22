import * as gcp from "@pulumi/gcp"

import { resourceName } from "../naming"
import { computeService } from "../service"

export const vpc = new gcp.compute.Network(
  resourceName("vpc"),
  {
    name: resourceName("vpc"),
    autoCreateSubnetworks: false
  },
  { dependsOn: [computeService] }
)
