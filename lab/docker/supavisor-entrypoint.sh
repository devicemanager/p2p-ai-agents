#!/bin/sh
# Entrypoint for supavisor with binary VAULT_ENC_KEY support

# Read the raw binary key from the file and export as VAULT_ENC_KEY
export VAULT_ENC_KEY="$(cat /run/secrets/vault_enc_key)"

# Execute the original command
exec "$@"
