# Kubernetes cluster setup

## Bootstrapping

1. Manually create an Omni installation media image in the Omni UI (select the `Generic Image (amd64)` option). Host it somewhere publicly accessible. Use the OVH BYOI option to reinstall the server with the downloaded image. Select `\EFI\BOOT\BOOTX64.EFI` as the boot path.

2. Create the cluster in Omni with the following config patches:

```
cluster:
  coreDNS:
    disabled: true

machine:
  kubelet:
    clusterDNS:
    - 10.96.0.10
```

3. Create a service account using `just 