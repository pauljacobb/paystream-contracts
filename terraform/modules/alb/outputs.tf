output "alb_id" {
  value       = aws_lb.main.id
  description = "ALB ID"
}

output "alb_arn" {
  value       = aws_lb.main.arn
  description = "ALB ARN"
}

output "alb_dns_name" {
  value       = aws_lb.main.dns_name
  description = "ALB DNS name"
}

output "alb_sg_id" {
  value       = aws_security_group.alb.id
  description = "ALB security group ID"
}

output "target_group_arn" {
  value       = aws_lb_target_group.main.arn
  description = "Target group ARN for ECS tasks"
}

output "target_group_name" {
  value       = aws_lb_target_group.main.name
  description = "Target group name"
}
