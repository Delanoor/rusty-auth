group "default" {
  targets = ["auth-service"]
}

target "auth-service" {
  context = "auth-service"
  dockerfile = "auth-service/Dockerfile"
  args = {
    DATABASE_URL      = "${DATABASE_URL}"
    JWT_SECRET        = "${JWT_SECRET}"
    SQLX_OFFLINE      = "${SQLX_OFFLINE}"
    POSTGRES_PASSWORD = "${POSTGRES_PASSWORD}"
    REDIS_HOST_NAME   = "${REDIS_HOST_NAME}"
    REDIS_PASSWORD    = "${REDIS_PASSWORD}"
    REDIS_PORT        = "${REDIS_PORT}"
  }
  tags = [
    "${DOCKER_USERNAME}/rusty-auth:${GITHUB_SHA}",
    "${DOCKER_USERNAME}/rusty-auth:latest"
  ]
}
