build-tkr:
  cd program && cargo build-bpf
  
deploy-tkr:
  solana program deploy target/deploy/tkr_token.so

run name +ARGS="":
  cargo run --bin {{name}} -- {{ARGS}}