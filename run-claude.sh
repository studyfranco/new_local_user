#!/bin/bash

cd /tmp

FLAGS=(--dangerously-skip-permissions --remote-control "the_new_user"
       --add-dir "/srv" "/tmp" "/mnt")
claude "${FLAGS[@]}" --continue || claude "${FLAGS[@]}"