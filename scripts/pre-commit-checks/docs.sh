  echo 'Running markdownlint...' >&2
  npx markdownlint --config docs/local/.markdownlint.yml 'docs/**/*.md'
  echo 'Running jeykyll build...' >&2
  cd docs
  bundle exec jekyll build --destination ../../_site --config ghp/_config.yml
  cd ..
