resource "kubernetes_service_account" "jenkins" {
  metadata {
    name      = "jenkins"
    namespace = "infra"
  }
}

resource "kubernetes_cluster_role_binding" "jenkins_admin" {
  metadata {
    name = "jenkins-admin"
  }
  role_ref {
    api_group = "rbac.authorization.k8s.io"
    kind      = "ClusterRole"
    name      = "cluster-admin"
  }
  subject {
    kind      = "ServiceAccount"
    name      = kubernetes_service_account.jenkins.metadata[0].name
    namespace = "infra"
  }
}

