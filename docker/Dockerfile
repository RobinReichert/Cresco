# docker/Dockerfile
FROM node:20-alpine

RUN apk add --no-cache ruby ruby-dev build-base curl
RUN gem install bundler
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
RUN rustup target add riscv32imc-unknown-none-elf
WORKDIR /app
COPY docs/config/Gemfile docs/config/_config.yml ./docs/config/
RUN bundle install --gemfile=docs/config/Gemfile

RUN npm install -g markdownlint-cli
