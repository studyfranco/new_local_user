#!/bin/bash

set -euo pipefail
tmux new-session -d -s the_new_user /home/thenewuser/run-claude.sh
tail -f /dev/null

exit 0