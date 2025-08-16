#!/bin/bash
printf "Tokens:\n"
cat /app/websockify_config
printf "\n\n"
python -m websockify 80 --target-config /app/websockify_config