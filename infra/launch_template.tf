data "aws_ami" "app" {
  filter {
    name   = "image-id"
    values = [var.ami_id]
  }
}
resource "aws_launch_template" "this" {
  name_prefix            = "${var.project}-lt-"
  image_id               = var.ami_id
  instance_type          = var.instance_type
  vpc_security_group_ids = [aws_security_group.app.id]
  update_default_version = true

  iam_instance_profile {
    name = aws_iam_instance_profile.ec2.name
  }

  metadata_options {
    http_endpoint               = "enabled"
    http_tokens                 = "required"
    http_put_response_hop_limit = 2
  }

  block_device_mappings {
    device_name = data.aws_ami.app.root_device_name
    ebs {
      volume_type = "gp3"
      encrypted   = true
    }
  }

  user_data = base64encode(templatefile("${path.module}/user_data.sh.tpl", {
    config_base64 = base64encode(jsonencode({
      aws_region    = var.region
      rds_secret_id = aws_db_instance.this.master_user_secret[0].secret_arn
      app_secret_id = aws_secretsmanager_secret.app.arn
      port          = 8080
      client_origin = local.app_origin
      rds_ca_bundle = "certs/global-bundle.pem"
    }))
    server_image = var.server_image
    client_image = var.client_image
  }))

  tag_specifications {
    resource_type = "instance"
    tags          = { Name = "${var.project}-app" }
  }
}
