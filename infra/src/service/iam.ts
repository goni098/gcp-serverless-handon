import * as gcp from "@pulumi/gcp"

import { resourceName } from "../naming"

export const iamService = new gcp.projects.Service(resourceName("api-iam"), {
  service: "iam.googleapis.com",
  disableOnDestroy: false
})
