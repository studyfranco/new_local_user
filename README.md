# new_local_user — Layer 3 : Antigravity CLI + Audio (`antigravity-audio`)

Image Docker **Google Antigravity CLI (`agy`) + gestion de bibliothèque musicale** (`ghcr.io/studyfranco/new_local_user:antigravity-audio`), basée sur `ghcr.io/studyfranco/new_local_user:antigravity`. Conçue pour être **100 % rootless** (`1000:1000`) et **read-only ready** (`read_only: true`).

## Outils Audio & Authentification à distance
- **Outils Audio** : `beets`, `picard` (`xvfb`), `shntool`, `flac`, `wavpack`, `fdupes`, `dos2unix`, `recode`, `coreutils`
- **Agent CLI** : Google Antigravity CLI (`agy`) avec `--remote-control` (hérité du layer 2 `antigravity`)
- **Tokens d'authentification headless / remote** : `GEMINI_API_KEY` (configure automatiquement `"modelProvider": "gemini"`) ou flux OAuth distant (`SSH_TTY`).

## Exemple `docker-compose.yml` (Rootless & Read-Only)

```yaml
services:
  antigravity-audio:
    image: ghcr.io/studyfranco/new_local_user:antigravity-audio
    container_name: antigravity_audio
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
      - ./data/config:/home/thenewuser/.config
      - /srv/music:/srv/music
      - /mnt:/mnt
```

## Utilisation
Rejoindre la session `tmux` d'Antigravity :
```bash
docker exec -it antigravity_audio tmux attach -t the_new_user
```

## Last Update
2026-11-04
