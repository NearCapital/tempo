resource "kubernetes_manifest" "tailscale_dns_config" {
  manifest = {
    "apiVersion" = "tailscale.com/v1alpha1"
    "kind" = "DNSConfig"
    "metadata" = {
      "name" = "ts-dns"
    }
    "spec" = {
      "nameserver" = {
        "service" = {
          "clusterIP" = "10.96.0.11"
        }
      }
    }
  }
}