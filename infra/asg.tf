resource "aws_autoscaling_group" "this" {
  name                      = "${var.project}-asg"
  min_size                  = var.asg_min
  max_size                  = var.asg_max
  desired_capacity          = var.asg_desired
  vpc_zone_identifier       = aws_subnet.app[*].id
  target_group_arns         = [aws_lb_target_group.this.arn]
  health_check_type         = "ELB"
  health_check_grace_period = 600

  launch_template {
    id      = aws_launch_template.this.id
    version = tostring(aws_launch_template.this.latest_version)
  }

  instance_refresh {
    strategy = "Rolling"
    preferences { min_healthy_percentage = 50 }
  }

  tag {
    key                 = "Name"
    value               = "${var.project}-app"
    propagate_at_launch = true
  }

  depends_on = [aws_iam_role_policy.secret, aws_iam_role_policy_attachment.ssm,
    aws_secretsmanager_secret_version.app, aws_route_table_association.app,
  aws_lb_listener.http_forward, aws_lb_listener.http_redirect, aws_lb_listener.https]

  lifecycle { ignore_changes = [desired_capacity] }
}

resource "aws_autoscaling_policy" "requests" {
  name                   = "${var.project}-req-per-target"
  autoscaling_group_name = aws_autoscaling_group.this.name
  policy_type            = "TargetTrackingScaling"

  target_tracking_configuration {
    predefined_metric_specification {
      predefined_metric_type = "ALBRequestCountPerTarget"
      resource_label         = "${aws_lb.this.arn_suffix}/${aws_lb_target_group.this.arn_suffix}"
    }
    target_value = var.target_requests_per_instance
  }
}
