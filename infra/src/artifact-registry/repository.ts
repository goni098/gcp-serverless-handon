import * as gcp from "@pulumi/gcp"

import { region } from "../config"
import { resourceName } from "../naming"
import { artifactRegistryService } from "../service"

export const repository = new gcp.artifactregistry.Repository(
  resourceName("images"),
  {
    location: region,
    repositoryId: resourceName("images"),
    format: "DOCKER"
  },
  { dependsOn: [artifactRegistryService] }
)
