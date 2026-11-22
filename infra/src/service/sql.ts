import * as gcp from "@pulumi/gcp"

import { resourceName } from "../naming"

export const sqlService = new gcp.projects.Service(resourceName("api-sqladmin"), {
  service: "sqladmin.googleapis.com",
  disableOnDestroy: false
})
