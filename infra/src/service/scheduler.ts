import * as gcp from "@pulumi/gcp"

import { resourceName } from "../naming"

export const schedulerService = new gcp.projects.Service(resourceName("api-scheduler"), {
  service: "cloudscheduler.googleapis.com",
  disableOnDestroy: false
})
