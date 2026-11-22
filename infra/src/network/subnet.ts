import * as gcp from "@pulumi/gcp"

import { region } from "../config"
import { resourceName } from "../naming"
import { vpc } from "./vpc"

export const subnet = new gcp.compute.Subnetwork(resourceName(`subnet-${region}`), {
  name: resourceName(`subnet-${region}`),
  network: vpc.id,
  ipCidrRange: "10.0.0.0/24",
  region
})
