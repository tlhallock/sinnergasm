
# TODO: do I need the ecs instance to have a public IP?


provider "aws" {
  region = "us-west-1"
}

# variable "image_tag" {
#   description = "The image tag to use"
#   type        = string
# }

locals {
  image_tag = trimspace(file("${path.module}/../container/version.txt"))
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
  map_public_ip_on_launch = true
  tags = {
    Name = "sinnergy-public-subnet"
  }

}

# resource "aws_subnet" "public" {
#   vpc_id     = aws_vpc.this.id
#   cidr_block = "10.0.2.0/24"
#   tags = {
#     Name = "sinnergy-private-subnet"
#   }
# }

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
  name   = "sinnergy-service-sg"
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

resource "aws_ecr_repository_policy" "this" {
  repository = aws_ecr_repository.this.name

  policy = jsonencode({
    Version = "2008-10-17",
    Statement = [
      {
        Effect = "Allow",
        Action = [
          "ecr:GetDownloadUrlForLayer",
          "ecr:BatchGetImage",
          "ecr:BatchCheckLayerAvailability"
        ],
        Principal = {
          Service = "ecs-tasks.amazonaws.com"
        }
      }
    ]
  })
}

resource "aws_ecs_cluster" "this" {
  name = "sinnergy-cluster"
}

resource "aws_cloudwatch_log_group" "this" {
  name              = "sinnergy-logs"
  retention_in_days = 30
}


resource "aws_ecs_task_definition" "this" {
  family                   = "sinnergy-server-task"
  network_mode             = "awsvpc"
  requires_compatibilities = ["FARGATE"]
  cpu                      = "2048"
  memory                   = "4096"
  execution_role_arn       = aws_iam_role.ecs_execution_role.arn

  container_definitions = jsonencode([{
    name         = "sinnergy-server"
    image        = "${aws_ecr_repository.this.repository_url}:${local.image_tag}"
    portMappings = [{ containerPort = 50051 }]
    logConfiguration = {
      logDriver = "awslogs",
      options = {
        "awslogs-group"         = aws_cloudwatch_log_group.this.name,
        "awslogs-region"        = "us-west-1",
        "awslogs-stream-prefix" = "ecs"
      }
    }
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

resource "aws_iam_role_policy_attachment" "b" {
  role       = aws_iam_role.ecs_execution_role.name
  policy_arn = "arn:aws:iam::aws:policy/AmazonEC2ContainerRegistryReadOnly"
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
    assign_public_ip = true
  }

  # Ensure that tasks are always running
  lifecycle {
    create_before_destroy = true
  }
}


################################################################################
# NLB
# resource "aws_eip" "this" {}

resource "aws_lb" "this" {
  name                       = "sinnergy-nlb"
  internal                   = false
  load_balancer_type         = "network"
  enable_deletion_protection = false

  # subnets = [aws_subnet.this.id]
  # enable_cross_zone_load_balancing = true
  security_groups = [aws_security_group.nlb_sg.id]

  subnet_mapping {
    subnet_id     = aws_subnet.this.id
    # allocation_id = aws_eip.this.id
  }
}

resource "aws_lb_target_group" "this" {
  name        = "sinnergy-nlb-tg"
  port        = 50051
  protocol    = "TCP"
  vpc_id      = aws_vpc.this.id
  target_type = "ip"
}

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
  name   = "sinnergy-nlb-sg"
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

resource "aws_security_group_rule" "nlb_egress" {
  security_group_id = aws_security_group.nlb_sg.id

  type        = "egress"
  from_port   = 50051
  to_port     = 50051
  protocol    = "tcp"
  source_security_group_id = aws_security_group.service_sg.id
}

# output "nlb_ip" {
#   value = aws_eip.this.public_ip
# }

output "sinnergy_nlb_dns_name" {
  description = "The DNS name of the Sinnergy NLB"
  value       = aws_lb.this.dns_name
}


resource "aws_security_group" "vpce_sg" {
  name   = "sinnergy-vpce-sg"
  vpc_id = aws_vpc.this.id
}

resource "aws_security_group_rule" "vpce_ingress" {
  security_group_id = aws_security_group.vpce_sg.id

  type                     = "ingress"
  from_port                = 0
  to_port                  = 0
  protocol                 = "-1"
  source_security_group_id = aws_security_group.service_sg.id
}

resource "aws_security_group_rule" "vpce_egress" {
  security_group_id = aws_security_group.vpce_sg.id

  type        = "egress"
  from_port   = 0
  to_port     = 65535
  protocol    = "-1"
  cidr_blocks = ["0.0.0.0/0"]
}


resource "aws_vpc_endpoint" "ecr_api" {
  vpc_id            = aws_vpc.this.id
  service_name      = "com.amazonaws.us-west-1.ecr.api"
  vpc_endpoint_type = "Interface"

  subnet_ids         = [aws_subnet.this.id]
  security_group_ids = [aws_security_group.vpce_sg.id]

  private_dns_enabled = true
}

resource "aws_vpc_endpoint" "ecr_docker" {
  vpc_id            = aws_vpc.this.id
  service_name      = "com.amazonaws.us-west-1.ecr.dkr"
  vpc_endpoint_type = "Interface"

  subnet_ids         = [aws_subnet.this.id]
  security_group_ids = [aws_security_group.vpce_sg.id]

  private_dns_enabled = true
}
