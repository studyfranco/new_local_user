# new_local_user — Layer 3 : Antigravity CLI + Dev Python & Rust (`antigravity-dev`)

Image Docker **Google Antigravity CLI (`agy`) + environnement de développement Python & Rust** (`ghcr.io/studyfranco/new_local_user:antigravity-dev`), basée sur `ghcr.io/studyfranco/new_local_user:antigravity`. Conçue pour être **100 % rootless** (`1000:1000`) et **read-only ready** (`read_only: true`).

## Outils de Développement inclus
- **Rust** : `rustc`, `cargo`, `rustfmt`, `clippy`, `rust-analyzer`, `clang`, `lld`, `lldb`, `gdb`, `cmake`, `libssl-dev`
- **Python** : `python3-dev`, `python3-venv`, `python3-pip`, `pipx`, `uv`, `ruff`, `black`, `mypy`, `pytest`, `ipython`, `py-spy`
- **Agent CLI** : Google Antigravity CLI (`agy`) avec `--remote-control` (hérité du layer 2 `antigravity`)

## Exemple `docker-compose.yml` (Rootless & Read-Only)

```yaml
services:
  antigravity-dev:
    image: ghcr.io/studyfranco/new_local_user:antigravity-dev
    container_name: antigravity_dev
    restart: unless-stopped
    user: "1000:1000"
    read_only: true
    security_opt:
      - no-new-privileges:true
    cap_drop:
      - ALL
    environment:
      - GEMINI_API_KEY=${GEMINI_API_KEY:-}
    tmpfs:
      - /tmp:exec,mode=1777
    volumes:
      - ./data/gemini:/home/thenewuser/.gemini
      - ./data/cargo:/tmp/cargo
      - /srv/projects:/srv/projects
      - /mnt:/mnt
```

## Utilisation
Rejoindre la session `tmux` d'Antigravity :
```bash
docker exec -it antigravity_dev tmux attach -t the_new_user
```

## Last Update
2026-11-04
