# AWS infrastructure for Daylist

This is a **new stack**, matching the app and the supplied reference. It does not adopt or modify existing console-created infrastructure. Default AMI: `ami-0b56af99f64ec815b`, region: `us-east-1`. Its availability, ownership/access, contents, and architecture must be checked in your AWS account before applying.

## Resources

- VPC across two availability zones: public ALB/NAT subnets, private application subnets, isolated RDS subnets.
- One NAT gateway per AZ for outbound image downloads, Secrets Manager, and SSM; app instances have no public IP and no SSH ingress.
- Internet-facing ALB targeting **Nginx port 80**, with **`/health`** proxied to Rust and PostgreSQL. Only status 200 passes.
- Optional ACM HTTPS listener with HTTP-to-HTTPS redirect. Existing validated certificate and DNS are supplied separately; Terraform does not own your domain.
- Launch template and Auto Scaling Group (2 instances by default), rolling updates, ALB health replacement, request-count target tracking.
- Encrypted PostgreSQL RDS, Multi-AZ by default, 7-day backups, isolated networking, deletion protection, and final snapshot on deletion.
- RDS-managed master credentials in Secrets Manager plus a generated 64-character JWT/application secret. EC2 role can read both secrets; SSM provides administration.
- Regional WAF: managed common, SQL injection, and IP reputation groups initially in count mode; IP rate limiting blocks above the configured limit.
- Separate S3-backend bootstrap with encryption, versioning, public-access blocking, destruction protection, and native state locking.

Paid resources include two NAT gateways, ALB, WAF, EC2, and Multi-AZ RDS. Review the plan before provisioning.

## AMI contract — check before applying

The supplied AMI is used as a **baked application image**. It must include:

1. A Linux/systemd environment with Docker at `/usr/bin/docker` and the Docker Compose v2 plugin.
2. The working server/client Docker images, default tags `todo-rust-server:latest` and `todo-rust-client:latest`. Override the image variables for a different tag or accessible registry. No source checkout or Rust compilation happens during boot.
3. The server image with the Secrets Manager-only loader, `/health`, and `certs/global-bundle.pem`; the client image must proxy `/health` and `/api` to `server:8080`.
4. An enabled SSM agent if Session Manager access is desired; Docker must be usable through systemd. The AMI must not include personal AWS credentials or JWT/database secret values.

If the AMI doesn't contain these images, bake a new AMI or provide pullable image references. Private registry authentication (for example ECR login/permissions) is not configured by this stack. Default public/local images need no registry credentials.

User data writes only non-secret identifiers to `/opt/todo-runtime/server/config.json`, writes the runtime Compose file, and enables `todo-rust-stack.service`. The containers fetch actual values from Secrets Manager through the EC2 role. IMDSv2 is required with hop limit **2** for Docker. Keep the baked app's Compose project name `todo-rust`; the startup service reconciles that project to its generated configuration. Disable unrelated application startup units in the baked AMI to prevent port conflicts.

## 1. Create the state bucket once

Install Terraform >=1.11 (or use the Docker commands below) and authenticate AWS on your workstation using an authorized deployment identity. Do not put access keys in Terraform files.

```bash
cd infra/bootstrap
terraform init
terraform plan -var='bucket_name=YOUR-GLOBALLY-UNIQUE-BUCKET' -out=bootstrap.tfplan
terraform apply bootstrap.tfplan
```

Bootstrap uses local state. Preserve it securely; state/plan files are ignored by Git. The bucket is in us-east-1 unless overridden. The bootstrap bucket refuses Terraform destruction.

## 2. Configure the main stack

```bash
cd ..
cp backend.hcl.example backend.hcl
cp terraform.tfvars.example terraform.tfvars
```

Set `bucket` in `backend.hcl` to the bucket from step 1. Adjust `terraform.tfvars` for your instance/image settings. Both real config files are ignored by Git. The app's `server/config.json` is generated on EC2: do not copy your laptop's config or profile into the AMI.

For HTTPS, supply a validated `certificate_arn` in us-east-1 and `app_origin = "https://todo.example.com"`, then create a DNS alias/CNAME to the ALB output. With an empty certificate ARN the stack serves HTTP for initial testing. An empty origin defaults to the ALB DNS URL. HTTPS with a certificate requires using a domain covered by that certificate, rather than the raw ALB hostname.

```bash
terraform init -backend-config=backend.hcl
terraform fmt -check -recursive
terraform validate
terraform plan -out=deployment.tfplan
terraform apply deployment.tfplan
terraform output app_url
```

Never commit `.terraform/`, actual `*.tfvars`, state, plans, or `backend.hcl`. **Commit `.terraform.lock.hcl`** for reproducible provider versions. The remote state includes the generated JWT secret because Terraform manages its value; treat access to state/backups/plans as secret access. The RDS-managed password is not fetched into Terraform. Deployment identity permissions are separate from the restricted EC2 runtime role.

The S3 backend identity needs bucket listing and state read/write, plus lock-object read/write/delete. See [HashiCorp's backend permissions](https://developer.hashicorp.com/terraform/language/backend/s3). No DynamoDB table is required.

## 3. Verify deployment

ASG instances must register healthy on port 80. Through SSM on an instance:

```bash
sudo systemctl status todo-rust-stack.service
sudo tail -100 /var/log/todo-bootstrap.log
sudo docker compose -p todo-rust -f /opt/todo-runtime/compose.yaml logs --tail=100 server
curl -i http://localhost/health
```

`200` means Nginx, Rust, and PostgreSQL are working together. Allow several minutes for first startup and IAM propagation. No private EC2 IP is required in Git or the browser config. The app URL points to the ALB.

User-data changes produce a new launch-template version and trigger an ASG rolling refresh. Database/JWT values are read once per server startup: after credential rotation, restart the app instances/containers so new connections use current values. Existing login tokens are invalidated by JWT rotation. Moving image tags alone does not trigger Terraform; deploy immutable tags and update the variables.

RDS and ALB deletion protection default to enabled. To intentionally remove a stack, change `deletion_protection = false`, apply that change, then review a destroy plan. RDS takes a final snapshot. The state bucket remains protected independently. Existing databases are not imported automatically: migrating existing RDS/VPC resources requires a deliberate import/migration plan, not applying this configuration over them.

## Validation status

Formatting and provider-schema validation are run locally without applying AWS resources. A real AWS plan/apply still needs your authenticated account and checks that the private AMI and chosen RDS engine/class are available. No claim is made that the AMI has been inspected or that infrastructure has been deployed.
