# teleprompt — Telegram prompt/response relay CLI (spec)

## Goal
A small CLI that sends a prompt message to a configured Telegram user via a bot, then waits (polling) for a reply and emits the reply to stdout or a file.

## CLI
Binary name: `teleprompt`

### Inputs
Exactly one of:
- `--message "..."`
- stdin (the entire stdin is read as the message)

If neither is provided, the program exits with an error.

### Outputs
- Default: write the reply to stdout.
- If `--out-file <path>` is provided: write the reply to that file (overwrite).

### Flags
- `--message <STRING>`: prompt message.
- `--out-file <PATH>`: where to write the reply.
- `--config <PATH>`: config file path.
- `--print-config-path`: print the resolved config path and exit.

## Config

### Default path
- Linux: `$XDG_CONFIG_HOME/teleprompt/config.toml` (or `~/.config/teleprompt/config.toml`)
- macOS: `~/Library/Application Support/teleprompt/config.toml`
- Windows: `%APPDATA%\\teleprompt\\config.toml`

### Format
TOML.

### Fields
- `bot_token` (string, required): Telegram bot token.
- `user_id` (integer, required): Telegram user id to message (for private chats this is also the chat id).
- `timeout_minutes` (integer, optional): how long to wait for a reply. Default: `60`.

Example:
```toml
bot_token = "123456:ABCDEF..."
user_id = 123456789
timeout_minutes = 60
```

## Telegram semantics
- On startup, the tool drains existing pending updates and records the next update offset so old messages don’t count as replies.
- It sends the prompt via `sendMessage`.
- It polls using `getUpdates` (long-poll) until it finds the first **text** message from the configured `user_id` *after* startup.
- If no reply arrives before the timeout, the program exits non-zero.

## Exit codes
- `0`: reply received and emitted.
- `2`: timed out waiting for reply.
- `1`: any other error (config missing/invalid, Telegram API error, IO error, etc.).
