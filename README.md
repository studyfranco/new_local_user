# new_local_user — Layer 2 : Goose CLI (`goose`)

Image Docker pour **Goose CLI (`goose`)** (`ghcr.io/studyfranco/new_local_user:goose`), basée sur `ghcr.io/studyfranco/new_local_user:master`. Conçue pour être **100 % rootless** (`1000:1000`) et **read-only ready** (`read_only: true`).

## Fonctionnalités & Serveur ACP distant (`goose serve`)
- **Agent CLI** : Goose CLI (`goose`) installé dans `/usr/local/bin/goose`
- **Mode autonome** : `GOOSE_MODE=auto` activé par défaut, lancé par `/usr/local/bin/run.sh` (hérité de `master`) via `/usr/local/bin/start-cli.sh` (`run-goose.sh`) dans une session `tmux` (`the_new_user`) avec reprise automatique (`goose session --resume || goose session`).
- **Connexion Web UI / Remote ACP (`goose serve`)** :
  - Lorsque `GOOSE_SERVER__SECRET_KEY` est défini, `goose serve --host 0.0.0.0 --port 3284` démarre en arrière-plan et expose le protocole **ACP (Agent Client Protocol)** sur `ws://goose_cli:3284/acp?token=<GOOSE_SERVER__SECRET_KEY>` pour être piloté depuis le conteneur de la branche `web` (`acp-ui`) ou Goose Desktop.
- **Tokens Provider Headless** : `GOOSE_PROVIDER`, `GOOSE_MODEL` et `GOOSE_PROVIDER__API_KEY`.

## Exemple `docker-compose.yml` (Rootless & Read-Only)

```yaml
services:
  goose:
    image: ghcr.io/studyfranco/new_local_user:goose
    container_name: goose_cli
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
      - ./data/goose/config:/home/thenewuser/.config/goose
      - ./data/goose/share:/home/thenewuser/.local/share/goose
      - ./data/goose/state:/home/thenewuser/.local/state/goose
      - /srv:/srv
      - /mnt:/mnt
```

## Utilisation
Rejoindre la session `tmux` de Goose :
```bash
docker exec -it goose_cli tmux attach -t the_new_user
```

## Last Update
2026-11-04
