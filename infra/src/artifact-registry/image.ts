import * as dockerBuild from "@pulumi/docker-build"
import * as gcp from "@pulumi/gcp"
import { getStack, interpolate, secret } from "@pulumi/pulumi"
import { resolve } from "node:path"

import { region } from "../config"
import { resourceName } from "../naming"
import { repository } from "./repository"

export type BinaryTarget = "api" | "cron" | "pubsub" | "db-migration-cli"

const workspaceRoot = resolve("..")
const dockerfile = resolve(workspaceRoot, "docker/Dockerfile")

// Populate the shared compilation cache before packaging runtime images in parallel.
const workspaceBuild = new dockerBuild.Image(resourceName("workspace-build"), {
  context: { location: workspaceRoot },
  dockerfile: { location: dockerfile },
  target: "build",
  platforms: [dockerBuild.Platform.Linux_amd64],
  exports: [{ cacheonly: {} }],
  push: false,
  buildOnPreview: false
})

export function createImage(bin: BinaryTarget) {
  const clientConfig = gcp.organizations.getClientConfigOutput()
  const registryAddress = `${region}-docker.pkg.dev`

  const image = new dockerBuild.Image(
    resourceName(`${bin}-image`),
    {
      context: { location: workspaceRoot },
      dockerfile: { location: dockerfile },
      target: bin === "db-migration-cli" ? "migration" : "function",
      buildArgs: { FUNCTION: bin },
      platforms: [dockerBuild.Platform.Linux_amd64],
      tags: [
        interpolate`${registryAddress}/${repository.project}/${repository.repositoryId}/${bin}:${getStack()}`
      ],
      registries: [
        {
          address: registryAddress,
          username: "oauth2accesstoken",
          password: secret(clientConfig.accessToken)
        }
      ],
      push: true,
      buildOnPreview: false
    },
    { dependsOn: [workspaceBuild] }
  )

  return image
}
