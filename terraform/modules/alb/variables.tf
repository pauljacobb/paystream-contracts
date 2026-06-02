variable "project_name" {
  type        = string
  description = "Project name"
}

variable "environment" {
  type        = string
  description = "Environment (prod, staging, etc)"
}

variable "vpc_id" {
  type        = string
  description = "VPC ID"
}

variable "public_subnet_ids" {
  type        = list(string)
  description = "List of public subnet IDs for ALB"
}

variable "app_port" {
  type        = number
  default     = 3000
  description = "Application port that ECS tasks listen on"
}

variable "certificate_arn" {
  type        = string
  default     = ""
  description = "ARN of SSL certificate for HTTPS (required for prod)"
}
