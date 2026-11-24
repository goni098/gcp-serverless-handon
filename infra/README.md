# Infrastructure

The `serverless` Pulumi project in [`src/main.ts`](src/main.ts) deploys the Cloud Run API, event pusher, subscriber, emitter, and migration job. It also manages their Artifact Registry repository, Pub/Sub topic and push subscription, Scheduler job, Secret Manager entries, and Cloud SQL database with Private Service Connect.

The `dev` stack targets `goni-serverless-dev-20261113` in `asia-southeast3`. GitHub Actions deploys the existing `goni098-org/serverless/dev` stack.

## Set up GitHub Actions

Run the bootstrap script once from the repository root. Use a Google Cloud account that can enable APIs and manage project IAM. [Bootstrap script](scripts/bootstrap.sh)

```sh
gcloud auth login
bash infra/scripts/bootstrap.sh
```

The script creates or reuses `github-ci-sa` and its workload identity pool and OIDC provider. It grants the deploy account permission to manage the dev stack. The provider created by the script trusts `goni098/gcp-serverless-handon` on `main`. GitHub Actions uses short-lived credentials through Workload Identity Federation.

In the GitHub repository, open **Settings → Secrets and variables → Actions** and configure these values.

| Name                             | Type     | Value                                                      |
| -------------------------------- | -------- | ---------------------------------------------------------- |
| `GCP_WORKLOAD_IDENTITY_PROVIDER` | Variable | Provider resource name printed by the bootstrap script     |
| `GCP_DEPLOY_SERVICE_ACCOUNT`     | Variable | Deploy account email printed by the bootstrap script       |
| `PULUMI_DEV_STACK`               | Variable | `goni098-org/serverless/dev`                               |
| `PULUMI_ACCESS_TOKEN`            | Secret   | Pulumi Cloud token with permission to update the dev stack |

Keep `serverless:githubCiServiceAccount` in [`Pulumi.dev.yaml`](Pulumi.dev.yaml) equal to `GCP_DEPLOY_SERVICE_ACCOUNT`. Pulumi grants that account `roles/iam.serviceAccountUser` on each workload or invoker account it creates. Use `createWorkloadAccount` and its `attachmentEmail` for new workloads so the grant is registered before the resource attaches the account.

## Stack configuration

| Key                                 | Purpose                                            |
| ----------------------------------- | -------------------------------------------------- |
| `gcp:project`                       | Google Cloud project ID                            |
| `serverless:region`                 | Deployment region                                  |
| `serverless:githubCiServiceAccount` | Deploy service account email                       |
| `serverless:heliusWebhookSecret`    | Secret for authenticating incoming Helius webhooks |
| `serverless:solRpcUrl`              | Solana RPC endpoint used by the emitter            |
| `serverless:solPk`                  | Signing key used by the emitter                    |

Use `pulumi config set --secret <key>` for the webhook secret, RPC URL, and signing key. The CI workflow selects `dev`. The `prod` stack needs its own region, deploy account, and secret values before use.

## Preview and deploy locally

Install Node.js 24, pnpm, Pulumi CLI, Google Cloud CLI, and Docker with Buildx. Start in the repository root.

```sh
gcloud auth application-default login
pulumi login
cd infra
pnpm install --frozen-lockfile
pulumi preview --stack goni098-org/serverless/dev
```

To deploy from `infra`, run `pulumi up --stack goni098-org/serverless/dev`. Set `BUILDX_BUILDER` to an existing Buildx builder name to select it for a local deploy. The images target `linux/amd64`.

## Deploy through CI

The workflow runs on pushes to `main`. It checks formatting and Clippy in both Rust workspaces and runs formatting and lint checks for Pulumi. Once those checks pass, `pulumi up` builds the images and updates the stack on a native x64 Ubuntu runner. [GitHub Actions workflow](../.github/workflows/deloy-dev-stack.yml)

Pulumi runs the Cloud Run `migration` job when the migration image reference or database secret version changes. The API depends on the migration job. For a schema change, verify the latest `migration` execution and its logs in Cloud Run after the deploy.

The stack exports `apiUrl` and `heliusWebhookUrl`. From `infra`, run `pulumi stack output --stack goni098-org/serverless/dev` to read them.
