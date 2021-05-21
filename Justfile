cli prog +ARGS="":
  cd cli && cargo run --bin {{prog}} -- {{ARGS}}

build:
  cd program && cargo build-bpf && anchor idl parse -f src/lib.rs -o target/idl/basic_0.json
  

deploy: build
  cd program && solana program deploy target/deploy/taker.so 

token +ARGS="":
  spl-token {{ARGS}}

test:
  anchor test --skip-deploy