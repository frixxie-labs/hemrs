FROM rust:latest AS build-stage
WORKDIR /usr/src/app
ENV SQLX_OFFLINE=1
COPY Cargo.toml Cargo.lock ./
COPY backend ./backend
COPY .sqlx ./.sqlx
RUN cargo install --path backend

FROM rust:slim
COPY --from=build-stage /usr/local/cargo/bin/backend /usr/local/bin/backend
CMD ["backend"]
