# File Server Helm Chart

This chart deploys the file-server container with a ConfigMap for non-secret
settings and an optional Gateway API HTTPRoute. Each file
mount uses a pre-existing PersistentVolumeClaim; the chart does not manage PVs
or PVCs. PostgreSQL and OAuth client credentials come from pre-existing
Kubernetes Secrets; the chart does not create or store credentials.

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

Keep credentials out of Helm values files. Set the references under
`fileServer.postgresql.secretKeyRef` and `fileServer.oidc.secretKeyRef` to use
different Secret names or keys.

## Configuration

Set `fileServer.publicUrl`, `fileServer.oidc.issuer`, and
`fileServer.oidc.clientId` for the deployment. The chart derives the OAuth
redirect URI from `publicUrl` by appending `/auth/callback`, which is the
file-server's fixed callback route. Register that resulting URI with the OAuth
provider. The chart does not currently configure local users, roles, or an
OIDC subject-to-user mapping. OIDC access will need the planned role-claims
support.

Configure each mount with a file-server `name` and an existing
`persistentVolumeClaimName`. The chart mounts that claim at `/data/<name>` and
sets the corresponding file-server mount path automatically.

Example values:

```yaml
fileServer:
  publicUrl: https://files.example.com
  postgresql:
    secretKeyRef:
      name: file-server-postgres
      key: url
  oidc:
    issuer: https://identity.example.com
    clientId: file-server
    secretKeyRef:
      name: file-server-oauth
      key: client-secret
  mounts:
    - name: documents
      persistentVolumeClaimName: documents-pvc
```

An optional Gateway API route can be enabled with values like:

```yaml
httpRoute:
  enabled: true
  parentRefs:
    - name: public-gateway
      namespace: gateway-system
  hostnames:
    - files.example.com
```

The cluster must have the Gateway API `HTTPRoute` CRD and a compatible Gateway
controller installed.

## Install

After publishing a chart version to GHCR:

```sh
helm registry login ghcr.io
helm install files oci://ghcr.io/OWNER/services/charts/file-server \
  --version CHART_VERSION \
  --namespace files --create-namespace \
  --set fileServer.publicUrl=https://files.example.com \
  --set fileServer.oidc.issuer=https://identity.example.com \
  --set fileServer.oidc.clientId=file-server
```

Configure Gateway TLS for the public origin. The chart's container
listens on `0.0.0.0:8080` and the Service exposes port 8080.
