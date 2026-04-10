# docker/Dockerfile
FROM node:20-alpine

RUN apk add --no-cache ruby ruby-dev build-base && \
    gem install bundler

WORKDIR /app
COPY docs/config/Gemfile docs/config/_config.yml ./docs/config/
RUN bundle install --gemfile=docs/config/Gemfile

RUN npm install -g markdownlint-cli
