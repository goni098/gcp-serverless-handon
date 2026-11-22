import * as gcp from "@pulumi/gcp"

import { resourceName } from "../naming"

export const runService = new gcp.projects.Service(resourceName("api-run"), {
  service: "run.googleapis.com",
  disableOnDestroy: false
})
