# Network Security Architecture

This document describes the network security configuration and restrictions implemented through AWS Security Groups and the Application Load Balancer.

## Overview

The infrastructure uses a multi-tier security model with the following components:

- **Load Balancer (ALB)**: Single public entry point for all external traffic
- **API Servers (ECS)**: Backend services accessible only from the ALB
- **Database (RDS)**: PostgreSQL database accessible only from ECS tasks
- **Cache (Redis)**: In-memory cache accessible only from ECS tasks

## Security Group Rules

### Load Balancer Security Group (`alb-sg`)

**Purpose**: Controls external access to the API

| Direction | Port | Protocol | Source | Description |
|-----------|------|----------|--------|-------------|
| Inbound | 80 | TCP | 0.0.0.0/0 | HTTP from internet |
| Inbound | 443 | TCP | 0.0.0.0/0 | HTTPS from internet |
| Outbound | All | All | 0.0.0.0/0 | All outbound (to communicate with ECS) |

**Key Point**: This is the ONLY security group that accepts traffic from the internet.

### ECS Security Group (`ecs-sg`)

**Purpose**: Restricts API access to traffic from the load balancer only

| Direction | Port | Protocol | Source | Description |
|-----------|------|----------|--------|-------------|
| Inbound | 3000 | TCP | alb-sg | Traffic from Load Balancer only |
| Outbound | All | All | 0.0.0.0/0 | All outbound (to DB, Redis, and external services) |

**Key Point**: ECS tasks cannot be accessed directly - ALL external traffic must go through the ALB.

### RDS Security Group (`rds-sg`)

**Purpose**: Restricts database access to API services only

| Direction | Port | Protocol | Source | Description |
|-----------|------|----------|--------|-------------|
| Inbound | 5432 | TCP | ecs-sg | PostgreSQL from ECS tasks only |
| Outbound | All | All | 0.0.0.0/0 | All outbound traffic |

**Key Point**: Database is NOT accessible from the internet, development machines, or CI/CD pipelines - only from running ECS tasks.

### Redis Security Group (`redis-sg`)

**Purpose**: Restricts cache access to API services only

| Direction | Port | Protocol | Source | Description |
|-----------|------|----------|--------|-------------|
| Inbound | 6379 | TCP | ecs-sg | Redis from ECS tasks only |
| Outbound | All | All | 0.0.0.0/0 | All outbound traffic |

**Key Point**: Cache is NOT accessible from the internet or external services - only from running ECS tasks.

## Network Flow Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                        INTERNET                             │
└──────────────┬──────────────────────────────────────────────┘
               │
               │ HTTP/HTTPS (80, 443)
               │ [0.0.0.0/0 allowed]
               ▼
        ┌──────────────┐
        │   ALB        │ alb-sg
        │ (Public)     │ ✓ Inbound: 80, 443
        └──────────────┘
               │
               │ App Port 3000
               │ [ecs-sg only allowed]
               ▼
        ┌──────────────┐
        │  ECS Tasks   │ ecs-sg
        │   (API)      │ ✓ Inbound: from ALB only
        │  (Private)   │ ✓ Outbound: to DB, Redis
        └────┬─────────┘
             │
    ┌────────┴────────┐
    │                 │
    │ Port 5432       │ Port 6379
    │ [ecs-sg only]   │ [ecs-sg only]
    ▼                 ▼
 ┌──────────┐    ┌──────────┐
 │   RDS    │    │  Redis   │
 │ (Private)│    │ (Private)│
 │rds-sg    │    │redis-sg  │
 │          │    │          │
 │PostgreSQL│    │  Cache   │
 └──────────┘    └──────────┘
```

## Data Flow Examples

### User API Request
1. Client makes HTTP request to `api.example.com`
2. DNS resolves to ALB public IP
3. ALB accepts connection (alb-sg allows 0.0.0.0/0 on port 80)
4. ALB forwards request to ECS task (ecs-sg allows traffic from alb-sg)
5. ECS task processes request
6. Result sent back through ALB to client

### Database Query from API
1. ECS task needs to query database
2. Initiates connection to RDS endpoint on port 5432
3. RDS security group checks source: ecs-sg
4. Connection allowed (rds-sg allows traffic from ecs-sg on port 5432)
5. Query executed and result returned

### Cache Access from API
1. ECS task needs to read/write cache
2. Initiates connection to Redis endpoint on port 6379
3. Redis security group checks source: ecs-sg
4. Connection allowed (redis-sg allows traffic from ecs-sg on port 6379)
5. Cache operation completes

## Security Guarantees

✅ **Database cannot be accessed**:
- Not from the internet (requires ecs-sg source)
- Not from development machines (no direct connection path)
- Not from CI/CD pipelines (only ECS tasks can connect)
- Only via the running API service

✅ **Cache cannot be accessed**:
- Not from the internet (requires ecs-sg source)
- Not from other services outside ECS
- Only via the running API service

✅ **API can only be accessed via Load Balancer**:
- No direct access to ECS tasks (ecs-sg only allows alb-sg)
- All traffic routed through ALB (enables WAF, SSL/TLS termination, etc.)
- ECS tasks never exposed on public internet

## Implementation Details

### Terraform Modules

**ALB Module** (`modules/alb/`):
- Creates Application Load Balancer in public subnets
- Creates ALB security group (allows 80/443 from 0.0.0.0/0)
- Creates target group pointing to ECS tasks
- Handles HTTP→HTTPS redirect in production
- Exports `alb_sg_id` for ECS module to reference

**ECS Module** (`modules/ecs/`):
- Requires `alb_sg_id` parameter
- Creates security group with ingress rule restricted to ALB
- Uses security group rules (not inline rules) for better auditability
- Exports `ecs_sg_id` for RDS/Redis modules to reference

**RDS Module** (`modules/rds/`):
- Requires `ecs_sg_id` parameter
- Creates security group with ingress rule restricted to ECS
- Database remains in private subnets

**Redis Module** (`modules/redis/`):
- Requires `ecs_sg_id` parameter
- Creates security group with ingress rule restricted to ECS
- Cache remains in private subnets

### Environment Configuration

**prod/main.tf**:
```hcl
module "alb" {
  source = "../../modules/alb"
  # ...
}

module "ecs" {
  source = "../../modules/ecs"
  alb_sg_id = module.alb.alb_sg_id  # Dependency chain ensures correct ordering
  # ...
}

module "rds" {
  source = "../../modules/rds"
  ecs_sg_id = module.ecs.ecs_sg_id  # Dependency chain ensures correct ordering
  # ...
}

module "redis" {
  source = "../../modules/redis"
  ecs_sg_id = module.ecs.ecs_sg_id  # Dependency chain ensures correct ordering
  # ...
}
```

The module dependencies ensure security groups are created in the correct order:
1. ALB created first
2. ECS created with reference to ALB SG
3. RDS/Redis created with reference to ECS SG

## Deployment Considerations

### Required Variables

For production deployment, provide:

```hcl
# terraform.tfvars
db_username           = "admin"
db_password           = "secure-password-here"
alb_certificate_arn   = "arn:aws:acm:region:account:certificate/xyz"
```

### Production vs Staging

**Production**:
- ALB enables HTTPS (requires certificate ARN)
- HTTP requests automatically redirected to HTTPS
- ALB deletion protection enabled

**Staging**:
- HTTP only (no certificate required)
- ALB deletion protection disabled

## Troubleshooting

### "Cannot connect to database"
1. Verify ECS task security group is being used for RDS ingress rule
2. Check RDS security group allows inbound on 5432
3. Verify ECS task has connectivity to RDS endpoint
4. Check database credentials are correct

### "Cannot connect to cache"
1. Verify ECS task security group is being used for Redis ingress rule
2. Check Redis security group allows inbound on 6379
3. Verify ECS task has connectivity to Redis endpoint

### "ECS tasks not receiving traffic"
1. Verify ALB security group has inbound rule on 80/443
2. Verify ECS security group has inbound rule from ALB SG on app port
3. Check ALB target group health checks passing
4. Verify ECS task is listening on correct port

### "Cannot access API"
1. Verify ALB security group has inbound rule on 80/443 from 0.0.0.0/0
2. Check ALB has public IP/DNS name
3. Verify ALB is in public subnets
4. Verify application is running in ECS tasks

## Modifying Security Rules

### Adding a New Service

To add a new service (e.g., another cache, message queue):

1. Create security group for the service
2. Add ingress rule from `ecs-sg` (if ECS needs access)
3. Create the service in the appropriate subnets
4. Update ECS module to add outbound rule if needed
5. Update documentation

### Allowing External Database Access

**NOT RECOMMENDED** - Use AWS RDS Proxy for controlled access:

1. Deploy RDS Proxy in front of database
2. Create separate security group for RDS Proxy
3. Allow limited CIDR blocks or security groups
4. Document the external access and rotate credentials regularly

### Allowing CI/CD Pipeline Access

**Recommended approach**: Use Systems Manager Session Manager instead of direct database access:

1. Grant CI/CD role permission to assume role with EC2/ECS access
2. Use Session Manager to connect to running ECS task
3. Task can connect to database, bypassing direct access requirement
4. No need to open database to external networks

## Compliance and Auditing

Security group rules are fully defined in Terraform and version controlled. To audit:

```bash
# View all security groups
aws ec2 describe-security-groups --filters Name=tag:Environment,Values=prod

# View specific security group rules
aws ec2 describe-security-group-rules --filters Name=group-id,Values=sg-xxxxx

# Check for overly permissive rules
aws ec2 describe-security-groups \
  --filters Name=ip-permission.cidr,Values=0.0.0.0/0 \
  --query 'SecurityGroups[*].[GroupId,GroupName]'
```

## Security Best Practices

✅ **Do**:
- Keep database and cache in private subnets
- Use ALB as single entry point
- Restrict security groups to specific sources
- Regularly audit security group rules
- Use TLS/SSL for data in transit
- Rotate database credentials regularly

❌ **Don't**:
- Allow 0.0.0.0/0 on database or cache ports
- Expose ECS tasks directly to internet
- Use overly permissive security group rules
- Disable ALB security groups
- Store credentials in code or version control

## References

- [AWS Security Groups Documentation](https://docs.aws.amazon.com/vpc/latest/userguide/VPC_SecurityGroups.html)
- [AWS Application Load Balancer](https://docs.aws.amazon.com/elasticloadbalancing/latest/application/)
- [AWS RDS Security Groups](https://docs.aws.amazon.com/AmazonRDS/latest/UserGuide/Overview.DBSecurityGroup.html)
- [AWS ElastiCache Security Groups](https://docs.aws.amazon.com/AmazonElastiCache/latest/red-ug/SecurityGroups.html)
