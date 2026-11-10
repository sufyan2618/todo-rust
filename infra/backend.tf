terraform {
  backend "s3" {
    key          = "todo/prod/terraform.tfstate"
    encrypt      = true
    use_lockfile = true
  }
}
