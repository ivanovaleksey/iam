#!/usr/bin/env bash
set -ex

export NAMESPACE=$(if [[ ${TRAVIS_TAG} ]]; then echo 'production'; else echo 'staging'; fi)
export DOCKER_IMAGE_TAG=$(if [[ ${TRAVIS_TAG} ]]; then echo ${TRAVIS_TAG}; else echo $(git rev-parse --short HEAD); fi)

echo "Deploy ${NAMESPACE}"

mkdir -p ${HOME}/.local/bin
export PATH=${HOME}/.local/bin:${PATH}

curl -Lo kubectl https://storage.googleapis.com/kubernetes-release/release/$(curl -s https://storage.googleapis.com/kubernetes-release/release/stable.txt)/bin/linux/amd64/kubectl && chmod +x kubectl && mv kubectl ${HOME}/.local/bin
curl -Lo skaffold https://storage.googleapis.com/skaffold/releases/latest/skaffold-linux-amd64 && chmod +x skaffold && mv skaffold ${HOME}/.local/bin

kubectl config set-cluster media --embed-certs --server ${KUBE_SERVER} --certificate-authority k8s/ca.crt
kubectl config set-credentials travis --token ${KUBE_TOKEN}
kubectl config set-context media --cluster media --user travis --namespace=${NAMESPACE}
kubectl config use-context media

CONFIGMAP_FILE="https://raw.githubusercontent.com/netology-group/environment/master/cluster/k8s/apps/iam/ns/${NAMESPACE}/iam-configmap.yaml"
curl -fsSL "${CONFIGMAP_FILE}?token=${GITHUB_TOKEN}" | kubectl apply -f -

echo ${DOCKER_PASSWORD} | docker login -u ${DOCKER_USERNAME} --password-stdin

kubectl delete -f k8s/iam-migrations.yaml || true
skaffold run
