group "default" {
  targets = ["auth-service"]
}

target "auth-service" {
  context = "./auth-service"
  dockerfile = "Dockerfile"

}
