FROM node as client
ADD /client /client
WORKDIR /client
RUN npm i
RUN npm run build

FROM rust as server
ADD Cargo.toml .
ADD /src /src
RUN cargo build --release

FROM ubuntu as runner
RUN apt-get update
RUN apt-get install libmariadb-dev -y
ADD .env .
COPY --from=client /client/dist/ ./client/dist/
COPY --from=server /target/release/simple-proxy ./
EXPOSE 80
ENTRYPOINT ./simple-proxy