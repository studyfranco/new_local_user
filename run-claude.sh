#!/bin/bash

cd /tmp

FLAGS=(--dangerously-skip-permissions --remote-control "${remote_name}"
       --add-dir "/srv" "/tmp" "/mnt")
claude "${FLAGS[@]}" --continue || claude "${FLAGS[@]}"