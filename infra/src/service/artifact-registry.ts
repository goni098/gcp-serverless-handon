import * as gcp from "@pulumi/gcp"

import { resourceName } from "../naming"

export const artifactRegistryService = new gcp.projects.Service(
  resourceName("api-artifactregistry"),
  {
    service: "artifactregistry.googleapis.com",
    disableOnDestroy: false
  }
)
