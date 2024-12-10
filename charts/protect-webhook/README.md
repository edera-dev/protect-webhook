# protect-webhook

![Version: 0.1.0](https://img.shields.io/badge/Version-0.1.0-informational?style=flat-square) ![Type: application](https://img.shields.io/badge/Type-application-informational?style=flat-square) ![AppVersion: 0.1.0](https://img.shields.io/badge/AppVersion-0.1.0-informational?style=flat-square)

A Helm chart for the Edera Protect Mutating Webhook

## Values

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| affinity | object | `{}` | Webhook server affinity |
| fullnameOverride | string | `""` |  |
| image.pullPolicy | string | `"IfNotPresent"` | This sets the pull policy for images. |
| image.repository | string | `"ghcr.io/edera-dev/protect-webhook"` |  |
| image.tag | string | `"latest"` | Overrides the image tag whose default is the chart appVersion. |
| imagePullSecrets | list | `[]` | This is for the secretes for pulling an image from a private repository |
| livenessProbe | object | `{"tcpSocket":{"port":8443}}` | Webhook server liveness probe |
| logLevel | string | `"info"` | Webhook server log level |
| nameOverride | string | `""` | This is to override the chart name. |
| nodeSelector | object | `{}` | Webhook server node selector |
| podAnnotations | object | `{}` | Webhook server pod annotations |
| podLabels | object | `{}` | Webhook server pod labels |
| podSecurityContext | object | `{}` | Webhook server pod security context |
| readinessProbe | object | `{"tcpSocket":{"port":8443}}` | Webhook server readiness probe |
| replicaCount | int | `1` | Webhook server replica count |
| resources | object | `{}` | Webhook server resources |
| securityContext | object | `{}` | Webhook server security context |
| service | object | `{"port":443,"type":"ClusterIP"}` | Webhook server service definition |
| tolerations | list | `[]` | Webhook server tolerations |
| volumeMounts | list | `[]` | Webhook server additional volumes mounts |
| volumes | list | `[]` | Webhook server additional volumes |
| webhook | object | `{"serviceNamespace":"edera-system"}` | Mutating webhook configuration |
| webhook.serviceNamespace | string | `"edera-system"` | Mutating webhook configuration for webhook server service namespace |

----------------------------------------------
Autogenerated from chart metadata using [helm-docs v1.14.2](https://github.com/norwoodj/helm-docs/releases/v1.14.2)