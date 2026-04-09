#!/bin/bash

case "$1" in
  build_docs)
    docker compose -f docker/docker-compose-docs.yml up
    ;;
  *)
    echo "Usage: $0 {serve_docs}"
    exit 1
    ;;
esac
