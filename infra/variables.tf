variable "region" {
  type    = string
  default = "us-east-1"
  validation {
    condition     = var.region == "us-east-1"
    error_message = "The supplied AMI is in us-east-1. Use a copied AMI before deploying in another region."
  }
}
variable "project" {
  type    = string
  default = "todo"
  validation {
    condition     = can(regex("^[a-z][a-z0-9-]{1,19}$", var.project))
    error_message = "Use 2-20 lowercase letters, digits or hyphens, starting with a letter."
  }
}
variable "vpc_cidr" { default = "10.20.0.0/16" }
variable "ami_id" { default = "ami-0b56af99f64ec815b" }
variable "instance_type" { default = "t3.small" }
variable "asg_min" { default = 2 }
variable "asg_desired" { default = 2 }
variable "asg_max" { default = 4 }
variable "target_requests_per_instance" { default = 100 }
variable "db_name" { default = "tododb" }
variable "db_username" { default = "todoadmin" }
variable "db_engine_version" { default = "16" }
variable "db_instance_class" { default = "db.t4g.micro" }
variable "db_multi_az" { default = true }
variable "db_backup_retention" { default = 7 }
variable "certificate_arn" {
  type        = string
  default     = ""
  description = "Existing validated ACM certificate ARN in us-east-1. Empty enables HTTP for initial setup."
}
variable "app_origin" {
  type        = string
  default     = ""
  description = "Browser origin, e.g. https://todo.example.com; empty uses the ALB DNS URL. Configure DNS separately."
  validation {
    condition     = var.app_origin == "" || can(regex("^https?://[^/]+$", var.app_origin))
    error_message = "Provide an HTTP(S) origin without a path or trailing slash."
  }
}
variable "server_image" { default = "todo-rust-server:latest" }
variable "client_image" { default = "todo-rust-client:latest" }
variable "waf_count_mode" { default = true }
variable "waf_rate_limit" { default = 500 }
variable "deletion_protection" { default = true }
variable "kms_key_arns" {
  type        = list(string)
  default     = []
  description = "Additional customer-managed keys allowed for secret decryption, if required."
}
