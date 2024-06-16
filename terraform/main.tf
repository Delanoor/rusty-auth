locals {
    project_name = "rusty-auth"
    aws_region = "us-west-1"
    certificate_arn   = "arn:aws:acm:us-west-1:043152383660:certificate/7d1377f9-b40c-4688-afd2-717f8da99f54"
    app_port = 3000
    health_check_path = "/"

}


provider "aws" {
    shared_credentials_files = ["~/.aws/credentials"]
    shared_config_files = ["~/.aws/config"]
    profile = "default"
    region = local.aws_region
}

terraform {
    backend "s3" {
        bucket = "rusty-auth-production-terraform-state"
        key = "ecs/terraform.tfstate"
        region = "us-west-1"
        shared_credentials_files = ["~/.aws/credentials"]
        profile = "default"
    }
}

module "ecs_prod" {
    source = "git@github.com:SchoiceHabsida/terraform-modules.git//ecs_fargate"
    project_name = local.project_name
    aws_region = local.aws_region
    health_check_path = local.health_check_path
    app_port = local.app_port
    environment = "production"
    task_cpu = 512  
    task_memory = 1024  
    app_service_cpu = 256  
    app_service_memory = 512  
    auth_service_cpu = 256  
    auth_service_memory = 512  
    certificate_arn = local.certificate_arn
    task_architecture = "ARM64"
}

