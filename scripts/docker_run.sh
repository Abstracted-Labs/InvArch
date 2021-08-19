#!/usr/bin/env bash
# This script meant to be run on Unix/Linux based systems
set -e

echo "*** Start Substrate InvArch node ***"

cd $(dirname ${BASH_SOURCE[0]})/..

docker-compose down --remove-orphans
docker-compose run --rm --service-ports dev $@
