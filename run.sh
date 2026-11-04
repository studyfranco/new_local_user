#!/bin/bash

set -euo pipefail
cd /tmp
tmux new-session -d -s the_new_user /bin/bash
tail -f /dev/null

exit 0
