variable "project_name" {
  type    = string
  default = "paystream"
}

variable "db_username" {
  type = string
}

variable "db_password" {
  type      = string
  sensitive = true
}

variable "alb_certificate_arn" {
  type        = string
  default     = ""
  description = "ARN of SSL certificate for HTTPS (optional for staging)"
}
