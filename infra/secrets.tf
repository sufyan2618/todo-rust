resource "random_password" "jwt" {
  length  = 64
  special = false
}
resource "aws_secretsmanager_secret" "app" {
  name_prefix             = "${var.project}/app-"
  recovery_window_in_days = 30
}
resource "aws_secretsmanager_secret_version" "app" {
  secret_id = aws_secretsmanager_secret.app.id
  secret_string = jsonencode({
    JWT_SECRET = random_password.jwt.result
    RDS_HOST   = aws_db_instance.this.address
    RDS_PORT   = aws_db_instance.this.port
    DB_NAME    = var.db_name
  })
}
