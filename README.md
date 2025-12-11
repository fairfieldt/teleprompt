# teleprompt

A small CLI that sends a prompt to a configured Telegram user via a bot, then waits for a reply and prints (or writes) the reply.

I made this so that coding agents like claude, codex, azad, opencode, etc can reach out to me when I'm away from keyboard. There are various more complicated remote uis, most targetting a specific tool. 

I wanted something that works in pretty much any environment. Add this (or something like this, I'm sure it can be tuned) to your system/custom prompt:

```md
<afk-mode-rules>
There is a mode called AFK (away from keyboard). in this mode I cannot answer your questions in the normal way.
I will enable AFK mode by telling you to enable it: AFK ON
I will disable AFK mode by telling you to enable it. AFK OFF


Any time you are about to ask a blocking question (including a pause-chat that contains a question), if AFK is ON, use teleprompt instead.

Use the `teleprompt` cli tool to send the question to me. You will send me an instant message and I will be able to reply. You should wait for the reply and then continue on with my answer.


Example:

    teleprompt --message "I need to create a git branch. should I call it feat/foo or feat/bar?"
    #... teleprompt sends the message and then blocks until I respond. then the response is written to stdout, you can read it and continue!

AFK mode starts off.
</afk-mode-rules>
AFK OFF
```

## Setup
Create a config file at the default config path (you can print it with `teleprompt --print-config-path`).

Typical defaults:
- Linux: `$XDG_CONFIG_HOME/teleprompt/config.toml` (or `~/.config/teleprompt/config.toml`)
- macOS: `~/Library/Application Support/teleprompt/config.toml`
- Windows: `%APPDATA%\\teleprompt\\config.toml`

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
