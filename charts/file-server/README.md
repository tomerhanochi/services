# File Server Helm Chart

This chart deploys the file-server container with a ConfigMap for non-secret
settings, an optional ingress, and a persistent volume for mounted files.
PostgreSQL and OAuth client credentials are read from pre-existing Kubernetes
Secrets; the chart does not create or store credentials.

## Required secrets

Create the PostgreSQL Secret with a connection URL in the `url` key:

```sh
kubectl create secret generic file-server-postgres \
  --from-literal=url='postgres://USER:PASSWORD@HOST:5432/DATABASE'
```

Create the OAuth Secret with the client secret in the `client-secret` key:

```sh
kubectl create secret generic file-server-oauth \
  --from-literal=client-secret='OAUTH_CLIENT_SECRET'
```

Keep these values out of Helm values files. Change the Secret names or keys with
`secrets.postgres` and `secrets.oauth` when using different names.

## Configuration

Set `fileServer.publicUrl`, `fileServer.oidc.issuer`, `fileServer.oidc.clientId`,
and `fileServer.oidc.redirectUri` for the deployment. The redirect URI must be
the public URL followed by `/auth/callback`. `fileServer.oidc.users` maps
provider subjects to local file-server user IDs; `fileServer.roles`,
`fileServer.users`, and `fileServer.machines` assign those principals access to
mounts.

Mounts are configured in `fileServer.mounts` and should use paths under `/data`,
which is backed by the chart's PVC by default. Use `persistence.existingClaim` to
reuse a claim, or set `persistence.enabled=false` for ephemeral storage.

Example values:

```yaml
fileServer:
  publicUrl: https://files.example.com
  oidc:
    issuer: https://identity.example.com
    clientId: file-server
    redirectUri: https://files.example.com/auth/callback
    users:
      "provider-subject": alice
  mounts:
    - name: documents
      path: /data/documents
  roles:
    editors:
      documents: editor
  users:
    alice:
      - editors
```

## Install

After publishing a chart version to GHCR:

```sh
helm registry login ghcr.io
helm install files oci://ghcr.io/OWNER/charts/file-server \
  --version CHART_VERSION \
  --namespace files --create-namespace \
  --set fileServer.publicUrl=https://files.example.com \
  --set fileServer.oidc.issuer=https://identity.example.com \
  --set fileServer.oidc.clientId=file-server \
  --set fileServer.oidc.redirectUri=https://files.example.com/auth/callback
```

Configure ingress TLS for the public origin. The chart's container listens on
`0.0.0.0:8080` and the Service exposes port 8080.
