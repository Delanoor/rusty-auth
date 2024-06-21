provider "aws" {
  region = "us-west-1"
}

data "aws_instance" "rusty_auth" {
  instance_id = "i-0d75aa485cf100fea"
}

resource "aws_security_group" "rustyauth_alb_sg" {
  name        = "rustyauth-alb-sg"
  description = "Allow TLS inbound traffic"
  vpc_id      = "vpc-84d821e2"

  ingress {
    from_port   = 443
    to_port     = 443
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  ingress {
    from_port   = 80
    to_port     = 80
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }
}

resource "aws_security_group" "rustyauth_instance_sg" {
  name        = "rustyauth-instance-sg"
  description = "Allow traffic to EC2 instance"
  vpc_id      = "vpc-84d821e2"

  ingress {
    from_port   = 3000
    to_port     = 3000
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }
}

resource "aws_lb" "rustyauth_alb" {
  name               = "rustyauth-alb"
  internal           = false
  load_balancer_type = "application"
  security_groups    = [aws_security_group.rustyauth_alb_sg.id]
  subnets            = ["subnet-a806c2f2", "subnet-565ff730"]

  enable_deletion_protection = false
}

resource "aws_lb_listener" "https" {
  load_balancer_arn = aws_lb.rustyauth_alb.arn
  port              = "443"
  protocol          = "HTTPS"

  ssl_policy      = "ELBSecurityPolicy-2016-08"
  certificate_arn = "arn:aws:acm:us-west-1:043152383660:certificate/7d1377f9-b40c-4688-afd2-717f8da99f54"

  default_action {
    type             = "forward"
    target_group_arn = aws_lb_target_group.rustyauth_tg.arn
  }
}

resource "aws_lb_target_group" "rustyauth_tg" {
  name     = "rustyauth-tg"
  port     = 3000
  protocol = "HTTP"
  vpc_id   = "vpc-84d821e2"

  health_check {
    interval            = 30
    path                = "/"
    protocol            = "HTTP"
    timeout             = 5
    healthy_threshold   = 5
    unhealthy_threshold = 2
  }
}

resource "aws_lb_target_group_attachment" "rustyauth_tg_attachment" {
  target_group_arn = aws_lb_target_group.rustyauth_tg.arn
  target_id        = data.aws_instance.rusty_auth.id
  port             = 3000
}

resource "aws_security_group_rule" "allow_alb_to_instance" {
  type                     = "ingress"
  from_port                = 3000
  to_port                  = 3000
  protocol                 = "tcp"
  source_security_group_id = aws_security_group.rustyauth_alb_sg.id
  security_group_id        = aws_security_group.rustyauth_instance_sg.id
}

output "alb_dns_name" {
  value = aws_lb.rustyauth_alb.dns_name
}
