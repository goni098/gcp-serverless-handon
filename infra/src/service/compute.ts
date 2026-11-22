import * as gcp from "@pulumi/gcp"

import { resourceName } from "../naming"

export const computeService = new gcp.projects.Service(resourceName("api-compute"), {
  service: "compute.googleapis.com",
  disableOnDestroy: false
})
