data "aws_iam_policy_document" "assume" {
  statement {
    actions = ["sts:AssumeRole"]
    principals {
      type        = "Service"
      identifiers = ["ec2.amazonaws.com"]
    }
  }
}

resource "aws_iam_role" "ec2" {
  name               = "${var.project}-ec2-role"
  assume_role_policy = data.aws_iam_policy_document.assume.json
}

resource "aws_iam_role_policy_attachment" "ssm" {
  role       = aws_iam_role.ec2.name
  policy_arn = "arn:aws:iam::aws:policy/AmazonSSMManagedInstanceCore"
}

data "aws_iam_policy_document" "secret" {
  statement {
    actions   = ["secretsmanager:GetSecretValue"]
    resources = [aws_db_instance.this.master_user_secret[0].secret_arn, aws_secretsmanager_secret.app.arn]
  }
}

resource "aws_iam_role_policy" "secret" {
  name   = "read-db-secret"
  role   = aws_iam_role.ec2.id
  policy = data.aws_iam_policy_document.secret.json
}

resource "aws_iam_instance_profile" "ec2" {
  name = "${var.project}-ec2-profile"
  role = aws_iam_role.ec2.name
}

data "aws_iam_policy_document" "kms" {
  count = length(var.kms_key_arns) > 0 ? 1 : 0
  statement {
    actions   = ["kms:Decrypt"]
    resources = var.kms_key_arns
  }
}
resource "aws_iam_role_policy" "kms" {
  count  = length(var.kms_key_arns) > 0 ? 1 : 0
  role   = aws_iam_role.ec2.id
  policy = data.aws_iam_policy_document.kms[0].json
}
