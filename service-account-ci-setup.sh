#!/usr/bin/env bash
set -euo pipefail

# Configure the GitHub Actions deploy identity for the Pulumi dev stack.

PROJECT_ID=goni-serverless-dev-20261113
PROJECT_NUMBER=918697368937
REGION=asia-southeast3
GITHUB_REPO=goni098/gcp-serverless-handon
POOL_ID=github
PROVIDER_ID=github-provider
ARTIFACT_REPOSITORY=images-repository
DEPLOY_SA="github-ci-sa@${PROJECT_ID}.iam.gserviceaccount.com"
POOL_RESOURCE="projects/${PROJECT_NUMBER}/locations/global/workloadIdentityPools/${POOL_ID}"
PROVIDER_RESOURCE="${POOL_RESOURCE}/providers/${PROVIDER_ID}"

# 1. Enable the APIs used by Workload Identity Federation.
gcloud services enable iam.googleapis.com iamcredentials.googleapis.com \
  sts.googleapis.com cloudresourcemanager.googleapis.com \
  --project="$PROJECT_ID"

# 2. Create the GitHub Actions deploy service account if needed.
existing_account=$(gcloud iam service-accounts list \
  --project="$PROJECT_ID" \
  --filter="email=${DEPLOY_SA}" \
  --format='value(email)')
if [[ -z "$existing_account" ]]; then
  gcloud iam service-accounts create github-ci-sa \
    --project="$PROJECT_ID" \
    --display-name="GitHub Actions CI"
fi

# 3. Create the identity pool and its GitHub OIDC provider if needed.
existing_pool=$(gcloud iam workload-identity-pools list \
  --project="$PROJECT_ID" \
  --location=global \
  --filter="name=${POOL_RESOURCE}" \
  --format='value(name)')
if [[ -z "$existing_pool" ]]; then
  gcloud iam workload-identity-pools create "$POOL_ID" \
    --project="$PROJECT_ID" \
    --location=global \
    --display-name="GitHub Actions"
fi

existing_provider=$(gcloud iam workload-identity-pools providers list \
  --project="$PROJECT_ID" \
  --location=global \
  --workload-identity-pool="$POOL_ID" \
  --filter="name=${PROVIDER_RESOURCE}" \
  --format='value(name)')
if [[ -z "$existing_provider" ]]; then
  gcloud iam workload-identity-pools providers create-oidc "$PROVIDER_ID" \
    --project="$PROJECT_ID" \
    --location=global \
    --workload-identity-pool="$POOL_ID" \
    --issuer-uri="https://token.actions.githubusercontent.com" \
    --attribute-mapping="google.subject=assertion.sub,attribute.repository=assertion.repository,attribute.ref=assertion.ref" \
    --attribute-condition="attribute.repository == '${GITHUB_REPO}' && attribute.ref == 'refs/heads/main'"
fi

# 4. Let GitHub Actions impersonate the deploy service account.
gcloud iam service-accounts add-iam-policy-binding "$DEPLOY_SA" \
  --project="$PROJECT_ID" \
  --role=roles/iam.workloadIdentityUser \
  --member="principalSet://iam.googleapis.com/${POOL_RESOURCE}/attribute.repository/${GITHUB_REPO}" \
  --format=none

# 5. Let Pulumi manage the resources declared in the dev stack.
project_roles=(
  roles/iam.serviceAccountAdmin
  roles/serviceusage.serviceUsageAdmin
  roles/artifactregistry.admin
  roles/compute.networkAdmin
  roles/cloudsql.admin
  roles/secretmanager.admin
  roles/pubsub.admin
  roles/cloudscheduler.admin
  roles/run.admin
)

for role in "${project_roles[@]}"; do
  printf 'Granting %s in %s\n' "$role" "$PROJECT_ID"
  gcloud projects add-iam-policy-binding "$PROJECT_ID" \
    --member="serviceAccount:${DEPLOY_SA}" \
    --role="$role" \
    --condition=None \
    --format=none
done

# 6. Let the deploy service account push images to the dev repository.
gcloud artifacts repositories add-iam-policy-binding "$ARTIFACT_REPOSITORY" \
  --project="$PROJECT_ID" \
  --location="$REGION" \
  --member="serviceAccount:${DEPLOY_SA}" \
  --role=roles/artifactregistry.writer \
  --format=none


printf '\nGitHub Actions repository variables\n'
printf 'GCP_WORKLOAD_IDENTITY_PROVIDER=%s\n' "$PROVIDER_RESOURCE"
printf 'GCP_DEPLOY_SERVICE_ACCOUNT=%s\n' "$DEPLOY_SA"
