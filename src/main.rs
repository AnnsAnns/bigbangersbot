use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use serenity::{async_trait, builder::CreateEmbedAuthor};

use serenity::model::gateway::Ready;
use serenity::prelude::*;

struct Handler {
    config: Config,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, PartialOrd)]
enum Priority {
    Low,
    Medium,
    High,
}

#[derive(Serialize, Deserialize, Debug)]
struct PriorityChannel {
    id: u64,
    priority: Priority,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Config {
    channels: Vec<PriorityChannel>,
    discord_token: String,
    discord_channel: u64,
    discord_server: u64,
    threshold: u64,
    reply: bool,
    replies: Vec<String>,
}

const REACTION_EMOJI: &str = "⭐";
const APPROVED_EMOJI: &str = "🌠";

async fn queue(handler: &Handler, ctx: Context, channel_id: u64) {
    let approved_emoji =
        serenity::model::channel::ReactionType::Unicode(APPROVED_EMOJI.to_string());
    let reaction_emoji =
        serenity::model::channel::ReactionType::Unicode(REACTION_EMOJI.to_string());

    let own_id = match &ctx.http.get_current_user().await {
        Ok(user) => user.id,
        Err(_) => return,
    };

    let messages = match ctx.http.get_messages(channel_id, "").await {
        Ok(messages) => messages,
        Err(_) => return,
    };

    for message in messages {
        if message.author.id == own_id {
            continue;
        }

        let approved_reactions = message
            .reaction_users(&ctx.http, approved_emoji.clone(), None, None)
            .await;

        if approved_reactions.is_err()
            || approved_reactions
                .unwrap()
                .iter()
                .any(|user| user.id == own_id)
        {
            continue;
        }

        let star_reactions = message
            .reactions
            .iter()
            .find(|reaction| reaction.reaction_type == reaction_emoji);

        if star_reactions.is_none() {
            continue;
        }

        let star_reaction = star_reactions.unwrap().clone();

        if star_reaction.count < handler.config.threshold {
            continue;
        }

        let author_name = &message
            .author
            .nick_in(&ctx.http, handler.config.discord_server)
            .await
            .unwrap_or(message.author.name.clone());

        let msg_url = &message.link_ensured(&ctx.http).await;
        let channel = ctx
            .http
            .get_channel(handler.config.discord_channel)
            .await
            .unwrap()
            .id();
        let image = message.attachments.first();
        let embed = message.embeds.first();
        channel
            .send_message(&ctx.http, |m| {
                m.embed(|e| {
                    e.set_author(
                        CreateEmbedAuthor::default()
                            .icon_url(message.author.avatar_url().unwrap_or_default())
                            .name(author_name)
                            .url("https://www.youtube.com/watch?v=qWNQUvIk954")
                            .to_owned(),
                    );
                    if image.is_some() {
                        e.image(image.unwrap().url.as_str());
                    } else if embed.is_some() {
                        let embed = embed.unwrap();
                        if embed.thumbnail.is_some() {
                            e.image(embed.thumbnail.as_ref().unwrap().url.as_str());
                        } else if embed.image.is_some() {
                            e.image(embed.image.as_ref().unwrap().url.as_str());
                        } else if embed.video.is_some() {
                            e.image(embed.video.as_ref().unwrap().url.as_str());
                        }
                    }

                    let mut content = if message.content.is_empty() {
                        match embed {
                            Some(embed) => embed.title.clone().unwrap_or("".to_string()),
                            None => "".to_string(),
                        }
                    } else {
                        message.content.clone()
                    };

                    if message.referenced_message.is_some() {
                        let referenced_message = message.referenced_message.clone().unwrap();
                        content = format!(
                            "> ⤴️ {} said: {}\n\n{}",
                            referenced_message.author.name,
                            referenced_message.content,
                            content,
                        );
                    }

                    e.description(format!("{}\n\n👉 [Original Message]({})", content, msg_url));
                    e.footer(|f| f.text(format!("⭐ {} ", star_reaction.count)));
                    e.timestamp(message.timestamp);
                    e
                })
            })
            .await
            .unwrap();

        message
            .react(&ctx.http, approved_emoji.clone())
            .await
            .unwrap();
        
        if handler.config.reply {
            let reply = handler.config.replies.choose(&mut rand::thread_rng());

            if reply.is_none() {
                return;
            }

            message.reply(&ctx.http, reply.unwrap()).await.unwrap();
        }
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, _ready: Ready) {
        let mut current_priority = Priority::High;

        println!("Logged in as {}", _ready.user.name);

        loop {
            for channel in &self.config.channels {
                if current_priority > channel.priority {
                    continue;
                }
                queue(self, ctx.clone(), channel.id).await;
            }

            current_priority = match current_priority {
                Priority::High => Priority::Medium,
                Priority::Medium => Priority::Low,
                Priority::Low => Priority::High,
            };
        }
    }
}

#[tokio::main]
async fn main() {
    // Parse channels from config
    let config_str = match std::fs::read_to_string("config.json") {
        Ok(config_str) => config_str,
        Err(_) => {
            panic!("Failed to read config.json. Make sure it exists (See config.example.json)");
        }
    };
    let config: Config = serde_json::from_str(&config_str)
        .expect("Failed to parse config.json");

    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::DIRECT_MESSAGES;

    let mut client = Client::builder(&config.discord_token, intents)
        .event_handler(Handler { config })
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
