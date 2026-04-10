#!/bin/bash

case "$1" in
  serve_docs)
    docker compose -f docker/docker-compose-docs.yml up
    ;;
  setup)
    if cmp -s scripts/pre-commit .git/hooks/pre-commit; then
      echo "Pre-commit hook already up to date"
    else
      cp scripts/pre-commit .git/hooks/pre-commit
      chmod +x .git/hooks/pre-commit
      echo "Pre-commit hook installed/updated!"
    fi
    ;;
  *)
    echo "Usage: $0 {serve_docs}"
    exit 1
    ;;
esac
