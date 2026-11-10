resource "aws_db_subnet_group" "this" {
  name       = "${var.project}-db-subnets"
  subnet_ids = aws_subnet.db[*].id
}

resource "aws_db_instance" "this" {
  identifier                  = "${var.project}-db"
  engine                      = "postgres"
  engine_version              = var.db_engine_version
  instance_class              = var.db_instance_class
  allocated_storage           = 20
  storage_type                = "gp3"
  storage_encrypted           = true
  db_name                     = var.db_name
  username                    = var.db_username
  manage_master_user_password = true
  multi_az                    = var.db_multi_az
  db_subnet_group_name        = aws_db_subnet_group.this.name
  vpc_security_group_ids      = [aws_security_group.db.id]
  publicly_accessible         = false
  backup_retention_period     = var.db_backup_retention
  skip_final_snapshot         = false
  final_snapshot_identifier   = "${var.project}-final-${random_id.snapshot.hex}"
  deletion_protection         = var.deletion_protection
  apply_immediately           = false
}

resource "random_id" "snapshot" { byte_length = 4 }
