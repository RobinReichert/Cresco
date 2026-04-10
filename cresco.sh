#!/bin/bash

case "$1" in
  serve_docs)
    docker compose -f docker/docker-compose-docs.yml up
    ;;
  setup)
    cp scripts/pre-commit .git/hooks/pre-commit
    chmod +x .git/hooks/pre-commit
    echo "Pre-commit hook installed!"
    ;;
  *)
    echo "Usage: $0 {serve_docs}"
    exit 1
    ;;
esac
