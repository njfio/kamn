const K8S_MANIFEST: &str = include_str!("../../../deploy/k8s/kamn-node.yaml");
const DEPLOY_DOC: &str = include_str!("../../../docs/ops/deployment.md");

#[test]
fn spec_c01_k8s_manifest_includes_service_api_service_and_ingress_baseline() {
    assert!(
        K8S_MANIFEST.contains("name: kamn-service-api"),
        "k8s manifest must include a dedicated service-api workload"
    );
    assert!(
        K8S_MANIFEST.contains("kind: Service"),
        "k8s manifest must include a Service resource"
    );
    assert!(
        K8S_MANIFEST.contains("kind: Ingress"),
        "k8s manifest must include an Ingress resource"
    );
    assert!(
        K8S_MANIFEST.contains("ingressClassName: nginx"),
        "k8s ingress baseline must declare ingress class"
    );
}

#[test]
fn spec_c02_k8s_manifest_includes_readiness_and_liveness_probes() {
    assert!(
        K8S_MANIFEST.contains("readinessProbe:"),
        "k8s manifest must define readiness probes"
    );
    assert!(
        K8S_MANIFEST.contains("livenessProbe:"),
        "k8s manifest must define liveness probes"
    );
    assert!(
        K8S_MANIFEST.contains("path: /healthz"),
        "k8s probes should target /healthz"
    );
    assert!(
        K8S_MANIFEST.contains("targetPort: http"),
        "k8s service should map to named container port"
    );
}

#[test]
fn spec_c03_deployment_doc_describes_service_ingress_and_probes() {
    assert!(
        DEPLOY_DOC.contains("Service (`kamn-service-api`)"),
        "deployment doc must describe service exposure contract"
    );
    assert!(
        DEPLOY_DOC.contains("Ingress (`kamn-service-api`)"),
        "deployment doc must describe ingress exposure contract"
    );
    assert!(
        DEPLOY_DOC.contains("readinessProbe"),
        "deployment doc must mention readiness probes"
    );
    assert!(
        DEPLOY_DOC.contains("livenessProbe"),
        "deployment doc must mention liveness probes"
    );
}
