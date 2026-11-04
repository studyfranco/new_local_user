# new_local_user — Layer 3 : OpenAI Codex CLI + Audio (`codex-audio`)

Image Docker **OpenAI Codex CLI (`codex`) + gestion de bibliothèque musicale** (`ghcr.io/studyfranco/new_local_user:codex-audio`), basée sur `ghcr.io/studyfranco/new_local_user:codex`. Conçue pour être **100 % rootless** (`1000:1000`) et **read-only ready** (`read_only: true`).

## Outils Audio & Serveur `codex app-server`
- **Outils Audio** : `beets`, `picard` (`xvfb`), `shntool`, `flac`, `wavpack`, `fdupes`, `dos2unix`, `recode`, `coreutils`
- **Agent CLI** : OpenAI Codex CLI (`codex`) + `codex app-server` (`/run/codex/app.sock` et `ws://0.0.0.0:4500`, hérités du layer 2 `codex`)
- **Tokens d'authentification headless / remote** : `OPENAI_API_KEY` ou `CODEX_API_KEY` (ou `--device-auth`).

## Exemple `docker-compose.yml` (Rootless & Read-Only)

```yaml
services:
  codex-audio:
    image: ghcr.io/studyfranco/new_local_user:codex-audio
    container_name: codex_audio
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
      - ./data/config:/home/thenewuser/.config
      - /srv/music:/srv/music
      - /mnt:/mnt
```

## Utilisation
Rejoindre la session `tmux` de Codex :
```bash
docker exec -it codex_audio tmux attach -t the_new_user
```

## Last Update
2026-11-04
