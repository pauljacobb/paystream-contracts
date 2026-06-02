# Network Security Configuration - Quick Reference

## Security Requirements Met ✅

### Requirement 1: DB only accessible from API service
- **Implementation**: RDS security group allows ingress only from ECS security group on port 5432
- **Verification**: `aws ec2 describe-security-groups --group-ids <rds-sg-id>`
- **Expected**: Only one ingress rule allowing source security group (ecs-sg) on port 5432

### Requirement 2: Redis only accessible from API service
- **Implementation**: Redis security group allows ingress only from ECS security group on port 6379
- **Verification**: `aws ec2 describe-security-groups --group-ids <redis-sg-id>`
- **Expected**: Only one ingress rule allowing source security group (ecs-sg) on port 6379

### Requirement 3: API only accessible via load balancer
- **Implementation**: ECS security group allows ingress only from ALB security group on port 3000
- **Verification**: `aws ec2 describe-security-groups --group-ids <ecs-sg-id>`
- **Expected**: Only one ingress rule allowing source security group (alb-sg) on app port (3000)

### Requirement 4: All rules documented in IaC
- **Implementation**: All security groups defined in Terraform modules with inline documentation
- **Files**:
  - `terraform/modules/alb/main.tf` - ALB security group
  - `terraform/modules/ecs/main.tf` - ECS security group
  - `terraform/modules/rds/main.tf` - RDS security group
  - `terraform/modules/redis/main.tf` - Redis security group
  - `terraform/NETWORK_SECURITY.md` - Detailed documentation
  - `terraform/README.md` - Architecture overview

## Module References

| Module | Purpose | Security Group |
|--------|---------|-----------------|
| `alb` | Application Load Balancer | `alb-sg` - Allows 80/443 from 0.0.0.0/0 |
| `ecs` | ECS cluster and tasks | `ecs-sg` - Allows traffic from alb-sg only |
| `rds` | PostgreSQL database | `rds-sg` - Allows traffic from ecs-sg only |
| `redis` | Redis cache | `redis-sg` - Allows traffic from ecs-sg only |

## Terraform Outputs

After deployment, the following outputs are available:

```bash
terraform output
```

Key outputs:
- `alb_dns_name` - Public DNS name of the load balancer
- `alb_sg_id` - ALB security group ID
- `ecs_sg_id` - ECS security group ID
- `rds_sg_id` - RDS security group ID (if output in rds module)
- `redis_sg_id` - Redis security group ID (if output in redis module)

## Verification Checklist

- [ ] VPC created with public/private subnets
- [ ] ALB created in public subnets
- [ ] ECS cluster created with tasks in private subnets
- [ ] RDS database created in private subnets
- [ ] Redis cluster created in private subnets
- [ ] ALB security group allows 80/443 from 0.0.0.0/0
- [ ] ECS security group allows traffic from ALB only
- [ ] RDS security group allows traffic from ECS only
- [ ] Redis security group allows traffic from ECS only
- [ ] All services use correct ports (ALB: 80/443, ECS: 3000, RDS: 5432, Redis: 6379)
- [ ] Target group health checks passing
- [ ] ECS tasks showing as healthy

## Terraform Code Structure

### ALB Module
```hcl
# Creates:
# - ALB security group (alb-sg)
# - Application Load Balancer
# - Target group for ECS tasks
# - HTTP and HTTPS listeners
```

### ECS Module
```hcl
# Creates:
# - ECS cluster
# - ECS security group (ecs-sg)
# - Security group rule: allow from alb-sg on app port
# - IAM roles and CloudWatch logging
```

### RDS Module
```hcl
# Creates:
# - RDS security group (rds-sg)
# - Security group rule: allow from ecs-sg on port 5432
# - RDS instance in private subnets
```

### Redis Module
```hcl
# Creates:
# - Redis security group (redis-sg)
# - Security group rule: allow from ecs-sg on port 6379
# - ElastiCache cluster in private subnets
```

## Common Issues and Solutions

### Issue: "Cannot connect to API"
**Causes to check**:
1. ALB security group missing inbound rule for 80/443
2. ALB target group health checks failing
3. ECS tasks not running or unhealthy
4. Security group rule pointing to wrong ECS SG

**Solution**:
```bash
# Check ALB health
aws elbv2 describe-target-health --target-group-arn <target-group-arn>

# Check ECS tasks
aws ecs list-tasks --cluster <cluster-name>
aws ecs describe-tasks --cluster <cluster-name> --tasks <task-arn>

# Check security groups
aws ec2 describe-security-groups --group-ids <alb-sg-id> <ecs-sg-id>
```

### Issue: "ECS tasks cannot connect to database"
**Causes to check**:
1. RDS security group missing inbound rule from ECS SG
2. ECS tasks unable to reach RDS endpoint
3. Database credentials incorrect
4. Network ACLs blocking traffic

**Solution**:
```bash
# Check RDS security group
aws ec2 describe-security-group-rules --filters Name=group-id,Values=<rds-sg-id>

# Check ECS task networking
aws ecs describe-tasks --cluster <cluster-name> --tasks <task-arn>
# Verify task is in correct subnet with route to RDS

# Test from ECS task
aws ecs execute-command --cluster <cluster> --task <task> \
  --container <container> --interactive --command "/bin/sh"
# Then: nc -zv <rds-endpoint> 5432
```

### Issue: "Database accessible from internet"
**Prevention**:
- Database must be in private subnets (no route to Internet Gateway)
- Security group must restrict to ecs-sg only
- Verify no CIDR 0.0.0.0/0 rules on database security group

**Validation**:
```bash
# List database security group rules
aws ec2 describe-security-group-rules \
  --filters Name=group-id,Values=<rds-sg-id> \
  --query 'SecurityGroupRules[?CidrIpv4]'
# Should return: No results (no CIDR rules)

aws ec2 describe-security-group-rules \
  --filters Name=group-id,Values=<rds-sg-id> \
  --query 'SecurityGroupRules[?ReferencedGroupInfo]'
# Should return: One result pointing to ecs-sg only
```

## Testing Network Connectivity

### From your local machine
```bash
# Test ALB is publicly accessible
curl -v http://<alb-dns-name>/health

# Verify direct ECS access blocked
curl -v http://<ecs-private-ip>:3000/health
# Should timeout or be refused

# Verify direct database access blocked
nc -zv <rds-endpoint> 5432
# Should timeout or be refused
```

### From ECS task
```bash
# Execute a command in a running ECS task
aws ecs execute-command --cluster <cluster> --task <task> \
  --container <container> --interactive --command "/bin/sh"

# Test database connectivity
nc -zv <rds-endpoint> 5432
# Should succeed (port 5432 open)

# Test Redis connectivity
nc -zv <redis-endpoint> 6379
# Should succeed (port 6379 open)

# Test outbound to internet
curl http://example.com
# Should work (if route configured)
```

## AWS CLI Commands Reference

```bash
# List all security groups in environment
aws ec2 describe-security-groups \
  --filters Name=tag:Environment,Values=prod \
  --query 'SecurityGroups[*].[GroupId,GroupName,Tags[?Key==`Name`].Value|[0]]' \
  --output table

# Get specific security group rules
aws ec2 describe-security-group-rules \
  --filters Name=group-id,Values=<sg-id>

# Check for overly permissive rules
aws ec2 describe-security-groups \
  --filters Name=ip-permission.cidr,Values=0.0.0.0/0 \
  --query 'SecurityGroups[*].[GroupId,GroupName,IpPermissions]' \
  --output table

# Get ALB DNS name
aws elbv2 describe-load-balancers \
  --load-balancer-arns <alb-arn> \
  --query 'LoadBalancers[0].DNSName' \
  --output text

# Monitor target group health
watch -n 5 'aws elbv2 describe-target-health \
  --target-group-arn <target-group-arn> \
  --query "TargetHealthDescriptions[*].[Target.Id,TargetHealth.State]" \
  --output table'
```

## Security Audit

Run this script to verify all security constraints:

```bash
#!/bin/bash

ENVIRONMENT=${1:-prod}
REGION=${2:-us-east-1}

echo "=== Security Audit for $ENVIRONMENT ==="

# Get security group IDs
ALB_SG=$(aws ec2 describe-security-groups \
  --filters Name=tag:Name,Values="*$ENVIRONMENT*alb*" \
  --query 'SecurityGroups[0].GroupId' --output text)

ECS_SG=$(aws ec2 describe-security-groups \
  --filters Name=tag:Name,Values="*$ENVIRONMENT*ecs*" \
  --query 'SecurityGroups[0].GroupId' --output text)

RDS_SG=$(aws ec2 describe-security-groups \
  --filters Name=tag:Name,Values="*$ENVIRONMENT*rds*" \
  --query 'SecurityGroups[0].GroupId' --output text)

REDIS_SG=$(aws ec2 describe-security-groups \
  --filters Name=tag:Name,Values="*$ENVIRONMENT*redis*" \
  --query 'SecurityGroups[0].GroupId' --output text)

echo "ALB SG: $ALB_SG"
echo "ECS SG: $ECS_SG"
echo "RDS SG: $RDS_SG"
echo "Redis SG: $REDIS_SG"

echo "\n=== Checking ALB allows 80/443 ==="
aws ec2 describe-security-group-rules --filters \
  Name=group-id,Values=$ALB_SG \
  --query 'SecurityGroupRules[?FromPort==`80` || FromPort==`443`]'

echo "\n=== Checking ECS allows from ALB only ==="
aws ec2 describe-security-group-rules --filters \
  Name=group-id,Values=$ECS_SG \
  --query 'SecurityGroupRules[?IsEgress==`false`]'

echo "\n=== Checking RDS allows from ECS only ==="
aws ec2 describe-security-group-rules --filters \
  Name=group-id,Values=$RDS_SG \
  --query 'SecurityGroupRules[?IsEgress==`false`]'

echo "\n=== Checking Redis allows from ECS only ==="
aws ec2 describe-security-group-rules --filters \
  Name=group-id,Values=$REDIS_SG \
  --query 'SecurityGroupRules[?IsEgress==`false`]'
```

## References

- [NETWORK_SECURITY.md](../terraform/NETWORK_SECURITY.md) - Detailed network architecture
- [terraform/README.md](../terraform/README.md) - Terraform deployment guide
- [AWS Security Groups](https://docs.aws.amazon.com/vpc/latest/userguide/VPC_SecurityGroups.html)
- [AWS Application Load Balancer](https://docs.aws.amazon.com/elasticloadbalancing/latest/application/)
