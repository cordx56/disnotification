FROM rust:1

WORKDIR /app
COPY . .

RUN cargo install --path .

ENTRYPOINT ["disnotification"]
