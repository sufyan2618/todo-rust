output "alb_dns_name" { value = aws_lb.this.dns_name }
output "rds_endpoint" { value = aws_db_instance.this.address }
output "db_secret_arn" { value = aws_db_instance.this.master_user_secret[0].secret_arn }
output "asg_name" { value = aws_autoscaling_group.this.name }

output "app_url" { value = local.app_origin }
output "app_secret_arn" { value = aws_secretsmanager_secret.app.arn }
output "vpc_id" { value = aws_vpc.this.id }
output "app_subnet_ids" { value = aws_subnet.app[*].id }
