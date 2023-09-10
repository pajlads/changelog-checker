FROM rust:1-alpine AS build

WORKDIR /usr/src/changelog-checker
COPY . .

RUN cargo install --path .

FROM alpine:latest
COPY --from=build /usr/local/cargo/bin/changelog-checker /usr/local/bin/changelog-checker
RUN apk add --no-cache git

CMD ["changelog-checker", "."]
