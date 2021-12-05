#!/usr/bin/env bash
set -e

[ -z "$1" ] && echo "Give fully qualified image name as parameter"
[ -z "$DOCKERHUB_USERNAME" ] && echo "Dockerhub username?" && read -s DOCKERHUB_USERNAME
[ -z "$DOCKERHUB_PASSWORD" ] && echo "Dockerhub password?" && read -s DOCKERHUB_PASSWORD

REPO=$1
README=$(pwd)/README.md
TOKEN=$(curl -s -H "Content-Type: application/json" -X POST -d '{"username": "'"${DOCKERHUB_USERNAME}"'", "password": "'${DOCKERHUB_PASSWORD}'"}' https://hub.docker.com/v2/users/login/ | jq -r .token)
RESPONSE=$(curl -s --write-out %{response_code} --output /dev/null -H "Authorization: JWT ${TOKEN}" -X PATCH --data-urlencode full_description@${README} https://hub.docker.com/v2/repositories/${REPO}/)


if [[ ${RESPONSE} -eq 200 ]]; then
    exit 0
else
	echo "Error with repo ${REPO}: ${RESPONSE}"
    exit 1
fi