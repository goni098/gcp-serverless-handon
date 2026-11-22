import { Config } from "@pulumi/pulumi"

const config = new Config()

export const project = new Config("gcp").require("project")
export const region = config.require("region")
