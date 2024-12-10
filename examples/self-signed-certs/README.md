# Using Cert Manager to generate self-signed certs

Start by installing cert-manager

```bash
helm repo add jetstack https://charts.jetstack.io
helm repo update
helm install cert-manager jetstack/cert-manager \
    --namespace cert-manager \
    --create-namespace \
    --set installCRDs=true
```

Next apply the manifest in the `edera-system` namespace

**NOTE:** you may need to create the `edera-system` namespace with `kubectl create namespace edera-system`

```bash
kubectl apply -f ./self-signed-certs.yaml --namespace edera-system
```

This should configure all the certificates and secrets you need to configure the mutating webhook.
