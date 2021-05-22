cli prog +ARGS="":
  cd cli && cargo run --bin {{prog}} -- {{ARGS}}

build:
  anchor build
  
deploy: build
  anchor deploy

token +ARGS="":
  spl-token {{ARGS}}

test:
  anchor test --skip-deploy

transact:
  node scripts/transact.js