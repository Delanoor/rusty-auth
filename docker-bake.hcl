group "default" {
  targets = ["auth-service"]
}

target "auth-service" {
  context = "auth-service"
  dockerfile = "auth-service/Dockerfile"
  args = {
    DATABASE_URL     = ""
    JWT_SECRET       = ""
    SQLX_OFFLINE     = ""
    POSTGRES_PASSWORD = ""
    REDIS_HOST_NAME  = ""
    REDIS_PASSWORD   = ""
    REDIS_PORT       = ""
  }

}
