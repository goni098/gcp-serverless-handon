import * as gcp from "@pulumi/gcp"

import { resourceName } from "../naming"

export const pubsubService = new gcp.projects.Service(resourceName("api-pubsub"), {
  service: "pubsub.googleapis.com",
  disableOnDestroy: false
})
