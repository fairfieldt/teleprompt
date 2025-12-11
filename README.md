# teleprompt

A small CLI that sends a prompt to a configured Telegram user via a bot, then waits for a reply and prints (or writes) the reply.

## Setup
Create a config file at `$HOME/.teleprompt`:

```toml
bot_token = "123456:ABCDEF..."
user_id = 123456789
timeout_minutes = 60
```

### Getting a bot token and your user id

1. Create a bot via BotFather

  - In Telegram, start chat with @BotFather.

  - /newbot → follow prompts → you get a token like:
  `1234567890:AAH-xxxxxxxxxxxxxxxxxxxxxxxxxxxx`


2. Get your own Telegram user ID

  - use a helper bot like @userinfobot or @userinfobot alternatives.

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
