locals {
  kube_host = get_env("KUBE_HOST_DEV_VIN_01")
  kube_token = get_env("KUBE_TOKEN_DEV_VIN_01")
  argocd_token = get_env("ARGOCD_DEV_VIN_01")
  argocd_url = "argocd-dev.tail388b2e.ts.net"
  onepassword_token = get_env("ONEPASSWORD_TOKEN_DEV_VIN_01")
  onepassword_connect_credentials = get_env("ONEPASSWORD_CREDS_DEV_VIN_01")
}