# Test that our namespaces exist
data "kubernetes_namespace" "infra" {
  metadata {
    name = "infra"
  }
}

data "kubernetes_namespace" "apps" {
  metadata {
    name = "apps"
  }
}

# Test that Jenkins is running
data "kubernetes_deployment" "jenkins" {
  metadata {
    name      = "jenkins"
    namespace = "infra"
  }
}

# Test that our app is running
data "kubernetes_deployment" "my_app" {
  metadata {
    name      = "my-app"
    namespace = "apps"
  }
}

# Output tests
output "test_results" {
  value = {
    infra_namespace_exists = data.kubernetes_namespace.infra.metadata[0].name
    apps_namespace_exists  = data.kubernetes_namespace.apps.metadata[0].name
    jenkins_replicas      = data.kubernetes_deployment.jenkins.spec[0].replicas
    app_replicas          = data.kubernetes_deployment.my_app.spec[0].replicas
  }
}
