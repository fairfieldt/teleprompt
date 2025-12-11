# teleprompt

A small CLI that sends a prompt to a configured Telegram user via a bot, then waits for a reply and prints (or writes) the reply.

## Setup
Create a config file at `$HOME/.teleprompt`:

```toml
bot_token = "123456:ABCDEF..."
user_id = 123456789
timeout_minutes = 60
```

For development, you can keep a repo-local config (and avoid putting secrets in `$HOME/.teleprompt` while iterating):

```bash
cp teleprompt.dev.toml.example teleprompt.dev.toml
$EDITOR teleprompt.dev.toml
```

`teleprompt.dev.toml` is gitignored.

## Usage
Message via flag:

```bash
teleprompt --message "what should we do today?"
```

Message via stdin:

```bash
echo "what should we do today?" | teleprompt
```

Write reply to a file:

```bash
teleprompt --message "ship it?" --out-file reply.txt
```

Use a repo-local config (handy for development):

```bash
teleprompt --config ./teleprompt.dev.toml --message "ping"
```
