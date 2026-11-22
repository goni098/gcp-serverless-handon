import * as gcp from "@pulumi/gcp"

import { resourceName } from "../naming"

export const secretManagerService = new gcp.projects.Service(resourceName("api-secretmanager"), {
  service: "secretmanager.googleapis.com",
  disableOnDestroy: false
})
