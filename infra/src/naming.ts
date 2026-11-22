import { getProject, getStack } from "@pulumi/pulumi"

// Keep Pulumi logical names and configurable GCP resource IDs consistent.
export function resourceName(purpose: string, maxLength = 63): string {
  const name = `${getProject()}-${getStack()}-${purpose}`
  if (!/^[a-z][a-z0-9-]*[a-z0-9]$/.test(name) || name.length > maxLength) {
    throw new Error(
      `Invalid resource name "${name}": use lowercase letters, digits and hyphens, start with a letter, and keep it within ${maxLength} characters.`
    )
  }
  return name
}
