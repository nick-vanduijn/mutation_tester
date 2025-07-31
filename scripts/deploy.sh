#!/bin/bash

set -e

NAMESPACE="mutation-tester"
ENVIRONMENT=${1:-development}
IMAGE_TAG=${2:-latest}

echo "🚀 Deploying mutation tester backend to $ENVIRONMENT environment"
echo "📦 Using image tag: $IMAGE_TAG"

if ! command -v kubectl &> /dev/null; then
    echo "❌ kubectl is not installed. Please install kubectl first."
    exit 1
fi

if ! command -v kustomize &> /dev/null; then
    echo "❌ kustomize is not installed. Please install kustomize first."
    exit 1
fi

if [[ "$ENVIRONMENT" != "development" && "$ENVIRONMENT" != "production" ]]; then
    echo "❌ Invalid environment. Use 'development' or 'production'"
    exit 1
fi

echo "🔍 Checking cluster connectivity..."
if ! kubectl cluster-info &> /dev/null; then
    echo "❌ Cannot connect to Kubernetes cluster. Please check your kubeconfig."
    exit 1
fi

echo "🏗️  Ensuring namespace exists..."
kubectl create namespace $NAMESPACE --dry-run=client -o yaml | kubectl apply -f -

echo "🏷️  Updating image tag to $IMAGE_TAG..."
cd k8s/overlays/$ENVIRONMENT
kustomize edit set image ghcr.io/mutation-tester/mutation-tester-backend:$IMAGE_TAG

echo "⚙️  Applying Kubernetes configuration..."
kustomize build . | kubectl apply -f -

echo "⏳ Waiting for deployment to complete..."
if [[ "$ENVIRONMENT" == "development" ]]; then
    DEPLOYMENT_NAME="dev-mutation-tester-backend"
else
    DEPLOYMENT_NAME="prod-mutation-tester-backend"
fi

kubectl rollout status deployment/$DEPLOYMENT_NAME -n $NAMESPACE --timeout=300s

echo "✅ Verifying deployment..."
kubectl get pods -n $NAMESPACE -l app.kubernetes.io/name=mutation-tester
kubectl get services -n $NAMESPACE
kubectl get ingress -n $NAMESPACE

echo "🏥 Running health checks..."
if [[ "$ENVIRONMENT" == "development" ]]; then
    SERVICE_NAME="dev-mutation-tester-service"
else
    SERVICE_NAME="prod-mutation-tester-service"
fi

echo "🔌 Setting up port forward for health check..."
kubectl port-forward service/$SERVICE_NAME 8080:80 -n $NAMESPACE &
PF_PID=$!
sleep 10

# Test health endpoint
if curl -f http://localhost:8080/health > /dev/null 2>&1; then
    echo "✅ Health check passed"
else
    echo "❌ Health check failed"
fi

# Cleanup port forward
kill $PF_PID 2>/dev/null || true

echo "🎉 Deployment completed successfully!"
echo ""
echo "📊 Access your services:"
echo "   • Application: Check your ingress configuration"
echo "   • Grafana: kubectl port-forward service/grafana-service 3000:3000 -n $NAMESPACE"
echo "   • Prometheus: kubectl port-forward service/prometheus-service 9090:9090 -n $NAMESPACE"
echo "   • Jaeger: kubectl port-forward service/jaeger-service 16686:16686 -n $NAMESPACE"
echo ""
echo "🔧 Useful commands:"
echo "   • View logs: kubectl logs -f deployment/$DEPLOYMENT_NAME -n $NAMESPACE"
echo "   • Get pods: kubectl get pods -n $NAMESPACE"
echo "   • Describe deployment: kubectl describe deployment/$DEPLOYMENT_NAME -n $NAMESPACE"
