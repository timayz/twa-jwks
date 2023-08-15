dev: up exemple

clippy:
	cargo clippy --fix --all-features -- -D warnings
	cargo clippy --all-features -- -D warnings

test:
	cargo test

fmt:
	cargo fmt -- --emit files

deny:
	cargo deny check

udeps:
	cargo udeps -p twa-actix-web -p twa-axum -p twa-jwks

udeps.leptos:
	echo "udeps.leptos"

advisory.clean:
	rm -rf ~/.cargo/advisory-db

pants: advisory.clean
	cargo pants

audit: advisory.clean
	cargo audit

outdated:
	cargo outdated

up:
	docker compose up -d --remove-orphans

stop:
	docker compose stop

down:
	docker compose down -v --remove-orphans

exemple:
	$(MAKE) _exemple -j2

_exemple: exemple.actix-web exemple.axum

exemple.actix-web:
	cargo run --bin twa-actix-web

exemple.axum:
	cargo run --bin twa-axum

token:
	curl -X POST http://127.0.0.1:6550/oauth/token \
		-H 'Content-Type: application/json' \
		-d '{"client_id": "john.doe"}'

hello.actix-web:
	curl http://127.0.0.1:3000/hello \
		-H 'Authorization: Bearer $(token)'

hello.axum:
	curl http://127.0.0.1:3001/hello \
		-H 'Authorization: Bearer $(token)'
