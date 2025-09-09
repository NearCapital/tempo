variable "cluster_name" {
  type        = string
  description = "Name of the Kubernetes cluster"
}

variable "tailscale_operator_version" {
  type = string
}

variable "coredns_version" {
  type        = string
  description = "CoreDNS Helm chart version"
}

variable "onepassword_connect_version" {
  type = string
  description = "OnePassword operator Helm chart version"
}

variable "onepassword_secret_token" {
  type = string
  sensitive = true
  description = "OnePassword operator secret token"
}

variable "onepassword_credentials" {
  type = string
  sensitive = true
  description = "OnePassword Connect credentials"
}