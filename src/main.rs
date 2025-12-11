mod config;
mod telegram;

use clap::Parser;
use std::io::{IsTerminal, Read, Write};
use std::path::PathBuf;
use std::time::{Duration, Instant};

#[derive(Parser, Debug)]
#[command(name = "teleprompt", version, about = "Telegram prompt/response relay CLI")]
struct Args {
    /// Message text to send. If omitted, the message is read from stdin.
    #[arg(long)]
    message: Option<String>,

    /// Write the reply to this file (overwrite). If omitted, reply is written to stdout.
    #[arg(long)]
    out_file: Option<PathBuf>,

    /// Config file path. If omitted, defaults to $HOME/.teleprompt
    #[arg(long)]
    config: Option<PathBuf>,
}

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("{:#}", e);
        std::process::exit(1);
    }
}

async fn run() -> anyhow::Result<()> {
    let args = Args::parse();

    let message = read_prompt_message(&args)?;

    let config_path = match &args.config {
        Some(p) => p.clone(),
        None => config::default_config_path()?,
    };
    let cfg = config::load(&config_path)?;

    let client = telegram::TelegramClient::new(cfg.bot_token);

    // Drain any old updates so only messages after this run count as replies.
    let mut offset = client.drain_updates().await?;

    client.send_message(cfg.user_id, &message).await?;
    eprintln!(
        "Waiting for reply from user_id={} (timeout={} minutes)...",
        cfg.user_id, cfg.timeout_minutes
    );

    let timeout = Duration::from_secs(cfg.timeout_minutes.saturating_mul(60));
    let start = Instant::now();

    while start.elapsed() < timeout {
        let elapsed = start.elapsed();
        if elapsed >= timeout {
            break;
        }
        let remaining = timeout - elapsed;

        let long_poll = remaining.min(Duration::from_secs(30));
        let long_poll_s = long_poll.as_secs();

        let updates = client.get_updates(offset, long_poll_s).await?;

        for update in &updates {
            offset = update.update_id + 1;

            if let Some(text) = telegram::extract_text_reply(update, cfg.user_id) {
                write_reply(&args, text)?;
                return Ok(());
            }
        }
    }

    eprintln!("Timed out waiting for reply.");
    std::process::exit(2);
}

fn read_prompt_message(args: &Args) -> anyhow::Result<String> {
    if let Some(m) = args.message.clone() {
        let m = m.trim().to_string();
        anyhow::ensure!(!m.is_empty(), "--message was provided but empty");
        return Ok(m);
    }

    if std::io::stdin().is_terminal() {
        anyhow::bail!("No --message provided and stdin is a terminal; pipe a message via stdin or pass --message.");
    }

    let mut raw = String::new();
    std::io::stdin().read_to_string(&mut raw)?;

    let raw = raw.trim_end_matches(['\r', '\n']).to_string();
    anyhow::ensure!(!raw.trim().is_empty(), "stdin was empty");
    Ok(raw)
}

fn write_reply(args: &Args, reply: &str) -> anyhow::Result<()> {
    if let Some(path) = &args.out_file {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }
        std::fs::write(path, reply)?;
        return Ok(());
    }

    let mut out = std::io::stdout().lock();
    out.write_all(reply.as_bytes())?;
    out.flush()?;
    Ok(())
}
