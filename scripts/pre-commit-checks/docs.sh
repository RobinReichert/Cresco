  echo 'Running markdownlint...' >&2
  npx markdownlint --config docs/config/.markdownlint.yml 'docs/**/*.md'

  echo 'Running jeykyll build...' >&2
  cd docs
  bundle exec jekyll build --destination ../../_site --config config/_config.yml
  cd ..
