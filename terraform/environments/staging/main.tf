module "vpc" {
  source = "../../modules/vpc"

  project_name = var.project_name
  environment  = "staging"
  vpc_cidr     = "10.1.0.0/16"
  public_subnets  = ["10.1.1.0/24", "10.1.2.0/24"]
  private_subnets = ["10.1.10.0/24", "10.1.11.0/24"]
}

# Load Balancer provides the only public access to the API
module "alb" {
  source = "../../modules/alb"

  project_name       = var.project_name
  environment        = "staging"
  vpc_id             = module.vpc.vpc_id
  public_subnet_ids  = module.vpc.public_subnet_ids
  app_port           = 3000
  certificate_arn    = var.alb_certificate_arn
}

# ECS tasks only accept traffic from Load Balancer
module "ecs" {
  source = "../../modules/ecs"

  project_name       = var.project_name
  environment        = "staging"
  vpc_id             = module.vpc.vpc_id
  public_subnet_ids  = module.vpc.public_subnet_ids
  private_subnet_ids = module.vpc.private_subnet_ids
  alb_sg_id          = module.alb.alb_sg_id
}

# RDS only accepts traffic from ECS tasks
module "rds" {
  source = "../../modules/rds"

  project_name       = var.project_name
  environment        = "staging"
  vpc_id             = module.vpc.vpc_id
  private_subnet_ids = module.vpc.private_subnet_ids
  ecs_sg_id          = module.ecs.ecs_sg_id
  username           = var.db_username
  password           = var.db_password
}

# Redis only accepts traffic from ECS tasks
module "redis" {
  source = "../../modules/redis"

  project_name       = var.project_name
  environment        = "staging"
  vpc_id             = module.vpc.vpc_id
  private_subnet_ids = module.vpc.private_subnet_ids
  ecs_sg_id          = module.ecs.ecs_sg_id
}
