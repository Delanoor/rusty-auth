group "default" {
  targets = ["auth-service"]
}

target "auth-service" {
<<<<<<< HEAD
  context = "./auth-service"
  dockerfile = "Dockerfile"
=======
  context = "."
  dockerfile = "Dockerfile"
  args = {
    DATABASE_URL     = ""
    JWT_SECRET       = ""
    SQLX_OFFLINE     = ""
    POSTGRES_PASSWORD = ""
    REDIS_HOST_NAME  = ""
    REDIS_PASSWORD   = ""
    REDIS_PORT       = ""
  }
>>>>>>> a61c825206d691ea1bd827a8fb9bcb8405ff0ff9

}
