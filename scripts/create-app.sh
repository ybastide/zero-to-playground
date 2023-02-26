#!/usr/bin/env bash
set -x
set -eo pipefail

doctl apps create --spec .do/app.yaml
