# PayStream Infrastructure as Code (Terraform)

This directory contains the Terraform configuration to deploy the PayStream infrastructure on AWS.

## Architecture

The infrastructure consists of:
- **VPC**: A dedicated VPC with public and private subnets across 2 Availability Zones.
- **Application Load Balancer (ALB)**: Public load balancer providing the only entry point to the API.
- **ECS (Fargate)**: An ECS cluster running the API and background services, accessible only from the ALB.
- **RDS (PostgreSQL)**: A managed PostgreSQL database in private subnets, accessible only from ECS.
- **ElastiCache (Redis)**: A managed Redis cluster in private subnets, accessible only from ECS.

## Network Security

All network access is restricted through security groups:
- **API**: Only accessible via Application Load Balancer (0.0.0.0/0 → ALB → ECS)
- **Database**: Only accessible from ECS tasks (ECS → RDS, port 5432)
- **Cache**: Only accessible from ECS tasks (ECS → Redis, port 6379)

See [NETWORK_SECURITY.md](./NETWORK_SECURITY.md) for detailed network architecture and security group configuration.

## Directory Structure

```
terraform/
├── modules/           # Reusable modules
│   ├── alb/          # Application Load Balancer
│   ├── ecs/          # ECS cluster and tasks
│   ├── rds/          # PostgreSQL database
│   ├── redis/        # Redis cache
│   └── vpc/          # VPC and subnets
├── environments/      # Environment-specific configurations
│   ├── staging/       # Staging environment
│   └── prod/          # Production environment
└── NETWORK_SECURITY.md  # Network security documentation
```

## Prerequisites

1. [Terraform](https://www.terraform.io/downloads.html) installed.
2. AWS CLI configured with appropriate credentials.
3. An S3 bucket named `paystream-terraform-state` and a DynamoDB table named `paystream-terraform-locks` for remote state management.
4. (Production only) An AWS Certificate Manager SSL certificate for HTTPS.

## Deployment Instructions

### 1. Initialize Terraform

Navigate to the environment directory you wish to deploy:

```bash
cd terraform/environments/staging
# or
cd terraform/environments/prod
```

Initialize the backend and modules:

```bash
terraform init
```

### 2. Configure Variables

Create a `terraform.tfvars` file in the environment directory:

For **staging**:
```hcl
db_username = "paystream_admin"
db_password = "your-secure-password"
# alb_certificate_arn is optional for staging (HTTP only)
```

For **production**:
```hcl
db_username = "paystream_admin"
db_password = "your-secure-password"
alb_certificate_arn = "arn:aws:acm:us-east-1:ACCOUNT_ID:certificate/CERTIFICATE_ID"
```

### 3. Plan Deployment

Review the changes that Terraform will perform:

```bash
terraform plan
```

### 4. Apply Deployment

Deploy the infrastructure:

```bash
terraform apply
```

### 5. Access the Application

Once deployment is complete, get the ALB DNS name:

```bash
terraform output -raw alb_dns_name
```

Access the API via the ALB (staging uses HTTP, production uses HTTPS):

**Staging**:
```bash
curl http://<ALB_DNS_NAME>/health
```

**Production**:
```bash
curl https://<ALB_DNS_NAME>/health
```

### 6. Cleanup

To destroy the infrastructure:

```bash
terraform destroy
```

## State Management

The state is stored in S3 with DynamoDB for state locking to prevent concurrent modifications.

- **Bucket**: `paystream-terraform-state`
- **Lock Table**: `paystream-terraform-locks`
- **Region**: `us-east-1`
