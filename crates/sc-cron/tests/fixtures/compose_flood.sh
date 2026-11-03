#!/bin/sh
# 65537 bytes: one past MAX_HOOK_OUTPUT_BYTES (64 KiB).
head -c 65537 /dev/zero | tr '\0' 'a'
