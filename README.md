# new_local_user — Layer 3 : Goose CLI + Dev Python & Rust (`goose-dev`)

Image Docker **Goose CLI (`goose`) + environnement de développement Python & Rust** (`ghcr.io/studyfranco/new_local_user:goose-dev`), basée sur `ghcr.io/studyfranco/new_local_user:goose`. Conçue pour être **100 % rootless** (`1000:1000`) et **read-only ready** (`read_only: true`).

## Outils de Développement inclus
- **Rust** : `rustc`, `cargo`, `rustfmt`, `clippy`, `rust-analyzer`, `clang`, `lld`, `lldb`, `gdb`, `cmake`, `libssl-dev`
- **Python** : `python3-dev`, `python3-venv`, `python3-pip`, `pipx`, `uv`, `ruff`, `black`, `mypy`, `pytest`, `ipython`, `py-spy`
- **Agent CLI** : Goose CLI (`goose`) + `goose serve` (port `3284`, hérité du layer 2 `goose`)

## Exemple `docker-compose.yml` (Rootless & Read-Only)

```yaml
services:
  goose-dev:
    image: ghcr.io/studyfranco/new_local_user:goose-dev
    container_name: goose_dev
    restart: unless-stopped
    user: "1000:1000"
    read_only: true
    security_opt:
      - no-new-privileges:true
    cap_drop:
      - ALL
    ports:
      - "3284:3284"
    environment:
      - GOOSE_MODE=auto
      - GOOSE_PROVIDER=${GOOSE_PROVIDER:-openai}
      - GOOSE_MODEL=${GOOSE_MODEL:-gpt-4o}
      - GOOSE_PROVIDER__API_KEY=${GOOSE_PROVIDER__API_KEY:-}
      - GOOSE_SERVER__SECRET_KEY=${GOOSE_SERVER__SECRET_KEY:-change_me_secret_token}
    tmpfs:
      - /tmp:exec,mode=1777
    volumes:
      - ./data/goose/config:/home/thenewuser/.config
      - ./data/goose/share:/home/thenewuser/.local/share/goose
      - ./data/goose/state:/home/thenewuser/.local/state/goose
      - ./data/cargo:/tmp/cargo
      - /srv/projects:/srv/projects
      - /mnt:/mnt
```

## Utilisation
Rejoindre la session `tmux` de Goose :
```bash
docker exec -it goose_dev tmux attach -t the_new_user
```

## Last Update
2026-11-04
