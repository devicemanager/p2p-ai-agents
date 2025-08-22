# encrypt_with_vault_key.exs
# Usage: VAULT_ENC_KEY=<hex_key> elixir encrypt_with_vault_key.exs "your secret value"

Mix.install([:cloak, :jason])


# Get the key from the environment (hex encoded)
key_hex = System.get_env("VAULT_ENC_KEY")
if is_nil(key_hex) or byte_size(key_hex) != 64 do
  IO.puts("Error: VAULT_ENC_KEY must be a 64-character hex string (32 bytes).")
  System.halt(1)
end
key = Base.decode16!(key_hex, case: :lower)

# Get the value to encrypt from the command line
[value] = System.argv()


# Encrypt using Cloak.Ciphers.AES.GCM directly
opts = [key: key, iv_length: 12, tag: "AES.GCM.V1"]
case Cloak.Ciphers.AES.GCM.encrypt(value, opts) do
  {:ok, ciphertext} ->
    IO.puts("Encrypted value (base64):")
    IO.puts(Base.encode64(ciphertext))
  {:error, err} ->
    IO.puts("Encryption failed: #{inspect(err)}")
    System.halt(1)
end
