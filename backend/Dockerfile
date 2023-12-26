FROM rust:1.74.1
WORKDIR /usr/src/app
COPY . .
ENV DATABASE_URL=postgresql://postgres:example@db:5432/postgres
RUN cargo install sqlx-cli
RUN cargo install --path .
CMD sqlx db create && sqlx migrate run && backend

