# Edera Protect Runtime Class Injector

This repo contains a mutating webhook to inject the `edera` runtime class into a kubernetes manifest.

### Why would I want this?

Edera Protect provides strong isolation for kubernetes workloads (see [edera.dev](https://edera.dev)
for more details). It does so by utilizing a runtime class name to specify whichworkloads should be
isolated. Some upstream projects don't have a way to specify a runtime class name so you can make
use of this webhook to inject it into the manifest upon creation. You also might not want to edit
every manifest you've ever created. Instead you can just use this mutating webhook to set a runtime
class name for manifests without ever touching their yaml.

### Quickstart

You can get started quickly by setting up self-signed certificates and then deploying the helm chart.
It deploys a mutating webhook which needs certs to communicate with the API Server. You can find an
example using self signed certs in the [examples/self-signed-certs](./examples/self-signed-certs/)
directory. Once the certificates are in place, install the mutating webhook and accompanying server

```bash
helm upgrade --install mutate oci://ghcr.io/edera-dev/charts/protect-webhook \
  --namespace edera-system \
  --create-namespace \
  --values ./examples/self-signed-certs/values.yaml
```
