# new_local_user — Layer 3 : Claude Code CLI + Dev Python & Rust (`claude-dev`)

Image Docker **Claude Code CLI + environnement de développement Python & Rust** (`ghcr.io/studyfranco/new_local_user:claude-dev`), basée sur `ghcr.io/studyfranco/new_local_user:claude`. Conçue pour être **100 % rootless** (`1000:1000`) et **read-only ready** (`read_only: true`).

## Outils de Développement inclus
- **Rust** : `rustc`, `cargo`, `rustfmt`, `clippy`, `rust-analyzer`, `clang`, `lld`, `lldb`, `gdb`, `cmake`, `libssl-dev`
- **Python** : `python3-dev`, `python3-venv`, `python3-pip`, `pipx`, `uv`, `ruff`, `black`, `mypy`, `pytest`, `ipython`, `py-spy`
- **Agent CLI** : Anthropic Claude Code (`claude`) avec `--remote-control` (hérité du layer 2 `claude`)

## Exemple `docker-compose.yml` (Rootless & Read-Only)

```yaml
services:
  claude-dev:
    image: ghcr.io/studyfranco/new_local_user:claude-dev
    container_name: claude_dev
    restart: unless-stopped
    user: "1000:1000"
    read_only: true
    security_opt:
      - no-new-privileges:true
    cap_drop:
      - ALL
    environment:
      - remote_name=claude_dev
      - CLAUDE_CODE_OAUTH_TOKEN=${CLAUDE_CODE_OAUTH_TOKEN:-}
      # - ANTHROPIC_API_KEY=${ANTHROPIC_API_KEY:-}
    tmpfs:
      - /tmp:exec,mode=1777
    volumes:
      - ./data/claude:/home/thenewuser/.claude
      - ./data/cargo:/tmp/cargo
      - /srv/projects:/srv/projects
      - /mnt:/mnt
```

## Utilisation
Rejoindre la session `tmux` de Claude :
```bash
docker exec -it claude_dev tmux attach -t the_new_user
```

## Last Update
2026-11-04
