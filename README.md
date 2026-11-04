# new_local_user — Layer 3 : OpenAI Codex CLI + Dev Python & Rust (`codex-dev`)

Image Docker **OpenAI Codex CLI (`codex`) + environnement de développement Python & Rust** (`ghcr.io/studyfranco/new_local_user:codex-dev`), basée sur `ghcr.io/studyfranco/new_local_user:codex`. Conçue pour être **100 % rootless** (`1000:1000`) et **read-only ready** (`read_only: true`).

## Outils de Développement inclus
- **Rust** : `rustc`, `cargo`, `rustfmt`, `clippy`, `rust-analyzer`, `clang`, `lld`, `lldb`, `gdb`, `cmake`, `libssl-dev`
- **Python** : `python3-dev`, `python3-venv`, `python3-pip`, `pipx`, `uv`, `ruff`, `black`, `mypy`, `pytest`, `ipython`, `py-spy`
- **Agent CLI** : OpenAI Codex CLI (`codex`) + `codex app-server` (`/run/codex/app.sock` et `ws://0.0.0.0:4500`, hérités du layer 2 `codex`)

## Exemple `docker-compose.yml` (Rootless & Read-Only)

```yaml
services:
  codex-dev:
    image: ghcr.io/studyfranco/new_local_user:codex-dev
    container_name: codex_dev
    restart: unless-stopped
    user: "1000:1000"
    read_only: true
    security_opt:
      - no-new-privileges:true
    cap_drop:
      - ALL
    ports:
      - "4500:4500"
    environment:
      - CODEX_HOME=/home/thenewuser/.codex
      - OPENAI_API_KEY=${OPENAI_API_KEY:-}
      # - CODEX_API_KEY=${CODEX_API_KEY:-}
    tmpfs:
      - /tmp:exec,mode=1777
    volumes:
      - ./data/codex:/home/thenewuser/.codex
      - ./data/codex-run:/run/codex
      - ./data/cargo:/tmp/cargo
      - /srv/projects:/srv/projects
      - /mnt:/mnt
```

## Utilisation
Rejoindre la session `tmux` de Codex :
```bash
docker exec -it codex_dev tmux attach -t the_new_user
```

## Last Update
2026-11-04
