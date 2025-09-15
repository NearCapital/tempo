include "kubernetes" {
  path = find_in_parent_folders("kubernetes.hcl")
}

include "cluster" {
  path = find_in_parent_folders("cluster.hcl")
  expose = true
}

include "root" {
  path = find_in_parent_folders("root.hcl")
}

terraform {
  source = find_in_parent_folders("modules/kubernetes/tailscale_manifests")
}