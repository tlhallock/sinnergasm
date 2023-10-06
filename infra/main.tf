
provider "aws" {
  region = "us-west-1"
}

################################################################################
# VPC

resource "aws_vpc" "this" {
  cidr_block           = "10.0.0.0/16"
  enable_dns_support   = true
  enable_dns_hostnames = true
  tags = {
    Name = "sinnergy-vpc"
  }
}

resource "aws_subnet" "this" {
  vpc_id     = aws_vpc.this.id
  cidr_block = "10.0.1.0/24"
  # map_public_ip_on_launch = true
  tags = {
    Name = "sinnergy-subnet"
  }
}

resource "aws_internet_gateway" "this" {
  vpc_id = aws_vpc.this.id
}

resource "aws_route_table" "this" {
  vpc_id = aws_vpc.this.id
  route {
    cidr_block = "0.0.0.0/0"
    gateway_id = aws_internet_gateway.this.id
  }
}

resource "aws_route_table_association" "a" {
  subnet_id      = aws_subnet.this.id
  route_table_id = aws_route_table.this.id
}

resource "aws_security_group" "service_sg" {
  vpc_id = aws_vpc.this.id
}

resource "aws_security_group_rule" "service_from_nlb" {
  security_group_id = aws_security_group.service_sg.id

  type                     = "ingress"
  from_port                = 50051
  to_port                  = 50051
  protocol                 = "tcp"
  source_security_group_id = aws_security_group.nlb_sg.id
}

resource "aws_security_group_rule" "service_egress" {
  security_group_id = aws_security_group.service_sg.id

  type        = "egress"
  from_port   = 0
  to_port     = 65535
  protocol    = "tcp"
  cidr_blocks = ["0.0.0.0/0"]
}

################################################################################
# TASK

resource "aws_ecr_repository" "this" {
  name = "sinnergy-serve"
}

resource "aws_ecs_cluster" "this" {
  name = "sinnergy-cluster"
}

# ECS Task Definition for Fargate
resource "aws_ecs_task_definition" "this" {
  family                   = "sinnergy-server-task"
  network_mode             = "awsvpc"
  requires_compatibilities = ["FARGATE"]
  cpu                      = "256"
  memory                   = "512"
  execution_role_arn       = aws_iam_role.ecs_execution_role.arn

  container_definitions = jsonencode([{
    name  = "sinnergy-server"
    image = "${aws_ecr_repository.this.repository_url}:latest"
    portMappings = [{
      containerPort = 50051
    }]
  }])
}

resource "aws_iam_role" "ecs_execution_role" {
  name = "ecs_execution_role"

  assume_role_policy = jsonencode({
    Version = "2012-10-17",
    Statement = [{
      Action = "sts:AssumeRole",
      Effect = "Allow",
      Principal = {
        Service = "ecs-tasks.amazonaws.com"
      }
    }]
  })
}

resource "aws_iam_role_policy_attachment" "a" {
  role       = aws_iam_role.ecs_execution_role.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AmazonECSTaskExecutionRolePolicy"
}


################################################################################
# SERVICE

resource "aws_ecs_service" "this" {
  name            = "sinnergy-service"
  cluster         = aws_ecs_cluster.this.id
  task_definition = aws_ecs_task_definition.this.arn
  launch_type     = "FARGATE"
  desired_count   = 1

  # load_balancer {
  #   target_group_arn = aws_lb_target_group.this.arn
  #   container_name   = "sinnergy-server"
  #   container_port   = 50051
  # }

  load_balancer {
    target_group_arn = aws_lb_target_group.this.arn
    container_name   = "sinnergy-server"
    container_port   = 50051
  }

  network_configuration {
    subnets         = [aws_subnet.this.id]
    security_groups = [aws_security_group.service_sg.id]
  }

  # Ensure that tasks are always running
  lifecycle {
    create_before_destroy = true
  }
}


################################################################################
# NLB + Elastic IP
resource "aws_eip" "this" {}

# Create a Network Load Balancer
resource "aws_lb" "this" {
  name                       = "sinnergy-nlb"
  internal                   = false
  load_balancer_type         = "network"
  enable_deletion_protection = false

  subnets = [aws_subnet.this.id]
  # enable_cross_zone_load_balancing = true
  security_groups = [aws_security_group.nlb_sg.id]

  # Attach the Elastic IP to the NLB
  subnet_mapping {
    subnet_id     = aws_subnet.this.id
    allocation_id = aws_eip.this.id
  }
}

# Create a Target Group for the NLB
resource "aws_lb_target_group" "this" {
  name     = "sinnergy-nlb-tg"
  port     = 50051
  protocol = "TCP"
  vpc_id   = aws_vpc.this.id
}

# Listener for the NLB
resource "aws_lb_listener" "this" {
  load_balancer_arn = aws_lb.this.arn
  port              = 50051
  protocol          = "TCP"

  default_action {
    type             = "forward"
    target_group_arn = aws_lb_target_group.this.arn
  }
}

resource "aws_security_group" "nlb_sg" {
  vpc_id = aws_vpc.this.id
}

resource "aws_security_group_rule" "nlb_ingress" {
  security_group_id = aws_security_group.nlb_sg.id

  type        = "ingress"
  from_port   = 50051
  to_port     = 50051
  protocol    = "tcp"
  cidr_blocks = ["0.0.0.0/0"]
}

output "nlb_ip" {
  value = aws_eip.this.public_ip
}


################################################################################
# LOAD BALANCER

# resource "aws_security_group" "alb_sg" {
#   vpc_id = aws_vpc.this.id
#   egress {
#     from_port   = 0
#     to_port     = 0
#     protocol    = "-1"
#     cidr_blocks = ["0.0.0.0/0"]
#   }
#   ingress {
#     from_port   = 80 # Change to 443 if you're using HTTPS
#     to_port     = 80
#     protocol    = "tcp"
#     cidr_blocks = ["0.0.0.0/0"]
#   }
# }


# resource "aws_lb" "this" {
#   name               = "sinnergy-alb"
#   internal           = false
#   load_balancer_type = "application"
#   security_groups    = [aws_security_group.alb_sg.id]
#   enable_deletion_protection = false

#   enable_cross_zone_load_balancing = true
#   subnets                          = [aws_subnet.this.id]
# }

# resource "aws_lb_listener" "this" {
#   load_balancer_arn = aws_lb.this.arn
#   port              = "80" # Change to 443 if you're using HTTPS

#   default_action {
#     type             = "forward"
#     target_group_arn = aws_lb_target_group.this.arn
#   }
# }

# resource "aws_lb_target_group" "this" {
#   name     = "sinnergy-tg"
#   port     = 50051
#   protocol = "HTTP" # Change if your service uses HTTPS
#   vpc_id   = aws_vpc.this.id
# }
# output "alb_endpoint" {
#   value = aws_lb.this.dns_name
# }

# resource "aws_route53_record" "this" {
#   zone_id = "YOUR_ROUTE_53_ZONE_ID"
#   name    = "service.yourdomain.com" 
#   type    = "A"

#   alias {
#     name                   = aws_lb.this.dns_name
#     zone_id                = aws_lb.this.zone_id
#     evaluate_target_health = true
#   }
# }