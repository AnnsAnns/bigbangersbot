use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use serenity::all::{ActivityData, CreateEmbed, CreateEmbedFooter, CreateMessage, Reaction};
use serenity::{async_trait, builder::CreateEmbedAuthor};

use serenity::model::gateway::Ready;
use serenity::prelude::*;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::Mutex;

struct Handler {
    config: Config,
    approved_messages: Arc<Mutex<HashSet<u64>>>,
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
    priority: Option<Priority>,
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
    enable_channel_whitelist: Option<bool>,
}

const REACTION_EMOJI: &str = "⭐";
const APPROVED_EMOJI: &str = "🌠";

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, _ready: Ready) {
        println!("Logged in as {}", _ready.user.name);

        // Set the bot's activity
        ctx.set_activity(Some(ActivityData::playing(format!("on v{} ⭐", VERSION))));
    }

    async fn reaction_add(&self, ctx: Context, add_reaction: Reaction) {
        let mut approved_messages = self.approved_messages.lock().await;

        if self.config.enable_channel_whitelist.is_some()
            && self.config.enable_channel_whitelist.unwrap()
        {
            let channel_id = add_reaction.channel_id;

            if !self
                .config
                .channels
                .iter()
                .any(|channel| channel.id.to_string() == channel_id.to_string())
            {
                return;
            }
        }

        let message_id = add_reaction.message_id.get();
        
        // Check if we've already processed this message
        if approved_messages.contains(&message_id) {
            return;
        }

        let approved_emoji =
            serenity::model::channel::ReactionType::Unicode(APPROVED_EMOJI.to_string());
        let reaction_emoji =
            serenity::model::channel::ReactionType::Unicode(REACTION_EMOJI.to_string());

        let own_id = match &ctx.http.get_current_user().await {
            Ok(user) => user.id,
            Err(_) => return,
        };

        let message = match add_reaction.message(&ctx.http).await {
            Ok(message) => message,
            Err(_) => return,
        };

        let approved_reactions = message
            .reaction_users(&ctx.http, approved_emoji.clone(), None, None)
            .await;

        if approved_reactions.is_err()
            || approved_reactions
                .unwrap()
                .iter()
                .any(|user| user.id == own_id)
        {
            return;
        }

        let star_reactions = message
            .reactions
            .iter()
            .find(|reaction| reaction.reaction_type == reaction_emoji);

        if star_reactions.is_none() {
            return;
        }

        let star_reaction = star_reactions.unwrap().clone();

        if star_reaction.count < self.config.threshold {
            return;
        }

        let discord_server_id = add_reaction.guild_id;

        let author_name = {
            let mut display_name = message.author.display_name().to_string();

            if discord_server_id.is_some() {
                let id = discord_server_id.unwrap();
                let member = message.author.clone();

                if let Some(member_name) = member.nick_in(&ctx.http, id).await {
                    display_name = member_name;
                }
            }

            format!("{} ({})", display_name, message.author.name)
        };

        let msg_url = &message.link();
        let channel = ctx
            .http
            .get_channel(self.config.discord_channel.into())
            .await
            .unwrap()
            .id();
        let image = message.attachments.first();
        let embed = message.embeds.first();

        let mut content = if message.content.is_empty() {
            match embed {
                Some(embed) => embed.title.clone().unwrap_or("".to_string()),
                None => "".to_string(),
            }
        } else {
            message.content.clone()
        };

        let image_str: Option<&str> = {
            if let Some(image) = image {
                Some(image.url.as_str())
            } else if embed.is_some() {
                let embed = embed.unwrap();
                if embed.thumbnail.is_some() {
                    Some(embed.thumbnail.as_ref().unwrap().url.as_str())
                } else if embed.image.is_some() {
                    Some(embed.image.as_ref().unwrap().url.as_str())
                } else if embed.video.is_some() {
                    Some(embed.video.as_ref().unwrap().url.as_str())
                } else {
                    None
                }
            } else {
                None
            }
        };

        if message.referenced_message.is_some() {
            let referenced_message = message.referenced_message.clone().unwrap();
            content = format!(
                "> ⤴️ {} said: {}\n\n{}",
                referenced_message.author.name, referenced_message.content, content,
            );
        }

        let embed_author = CreateEmbedAuthor::new(author_name)
            .icon_url(message.author.avatar_url().unwrap_or_default())
            .url("https://www.youtube.com/watch?v=qWNQUvIk954")
            .to_owned();

        let mut message_embed = CreateEmbed::new().author(embed_author);

        if let Some(image_url) = image_str {
            message_embed = message_embed.image(image_url);
        }

        message_embed = message_embed
            .description(format!("{}\n\n👉 [Original Message]({})", content, msg_url))
            .footer(CreateEmbedFooter::new("⭐".to_string()))
            .timestamp(message.timestamp);

        let send_message = CreateMessage::new().embed(message_embed);

        channel.send_message(&ctx.http, send_message).await.unwrap();

        message
            .react(&ctx.http, approved_emoji.clone())
            .await
            .unwrap();

        // Mark this message as approved
        approved_messages.insert(message_id);

        if self.config.reply {
            let reply = self.config.replies.choose(&mut rand::thread_rng());

            if reply.is_none() {
                return;
            }

            message.reply(&ctx.http, reply.unwrap()).await.unwrap();
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
    let config: Config = serde_json::from_str(&config_str).expect("Failed to parse config.json");

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::GUILD_MESSAGE_REACTIONS;

    let mut client = Client::builder(&config.discord_token, intents)
        .event_handler(Handler {
            config,
            approved_messages: Arc::new(Mutex::new(HashSet::new())),
        })
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
