cli prog +ARGS="":
  cd cli && cargo run --bin {{prog}} -- {{ARGS}}

build:
  cd program && cargo build-bpf

deploy: build
  cd program && solana program deploy target/deploy/taker.so 

token +ARGS="":
  spl-token {{ARGS}}