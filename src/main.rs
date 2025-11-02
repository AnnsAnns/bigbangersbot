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

    /// Basically the main logic of the bot
    /// Triggered when a reaction is added to a message
    async fn reaction_add(&self, ctx: Context, add_reaction: Reaction) {
        // Only process whitelisted channels
        if !self.is_channel_whitelisted(add_reaction.channel_id) {
            return;
        }

        let message_id = add_reaction.message_id.get();

        // Checks for cases where the reaction is to non-existent messages
        let message = match add_reaction.message(&ctx.http).await {
            Ok(message) => message,
            Err(_) => return,
        };

        // Check if the bot has already approved this message on discord
        // or if the message doesn't meet the star threshold
        if !self.meets_star_threshold(&message) || self.has_approved_reaction(&ctx, &message).await
        {
            return;
        }

        // This is a temporary local in-memory storage to keep track of approved messages
        // to avoid cases where discord cache doesn't properly show the bot's own reactions
        let mut approved_messages = self.approved_messages.lock().await;
        // Check if we've already processed this message
        if approved_messages.contains(&message_id) {
            return;
        }

        let discord_server_id = add_reaction.guild_id;

        // Create and send the starboard message
        self.create_starboard_message(&ctx, &message, discord_server_id)
            .await;

        let approved_emoji =
            serenity::model::channel::ReactionType::Unicode(APPROVED_EMOJI.to_string());
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

impl Handler {
    /// Check if the channel is whitelisted
    /// If channel whitelisting is disabled, all channels are considered whitelisted
    fn is_channel_whitelisted(&self, channel_id: serenity::model::id::ChannelId) -> bool {
        if self.config.enable_channel_whitelist.unwrap_or(false) {
            self.config
                .channels
                .iter()
                .any(|channel| channel.id == channel_id.get())
        } else {
            true
        }
    }

    /// Check if the bot has already approved the message
    /// by reacting with the approved emoji
    /// If the bot has approved the message, return true
    async fn has_approved_reaction(
        &self,
        ctx: &Context,
        message: &serenity::model::channel::Message,
    ) -> bool {
        let approved_emoji =
            serenity::model::channel::ReactionType::Unicode(APPROVED_EMOJI.to_string());

        let own_id = match &ctx.http.get_current_user().await {
            Ok(user) => user.id,
            Err(_) => return false,
        };

        let approved_reactions = message
            .reaction_users(&ctx.http, approved_emoji.clone(), None, None)
            .await;

        if approved_reactions.is_err() {
            return false;
        }

        approved_reactions
            .unwrap()
            .iter()
            .any(|user| user.id == own_id)
    }

    /// Check if the message meets the star threshold
    /// by counting the number of star reactions
    /// If the number of star reactions is greater than or equal to the threshold, return true
    fn meets_star_threshold(&self, message: &serenity::model::channel::Message) -> bool {
        let reaction_emoji =
            serenity::model::channel::ReactionType::Unicode(REACTION_EMOJI.to_string());

        let star_reactions = message
            .reactions
            .iter()
            .find(|reaction| reaction.reaction_type == reaction_emoji);

        if star_reactions.is_none() {
            return false;
        }

        let star_reaction = star_reactions.unwrap().clone();

        star_reaction.count >= self.config.threshold
    }

    /// Create and send the starboard message
    /// with the original message content, author, and link
    /// to the original message
    /// Also (tries) to include most images/embeds
    async fn create_starboard_message(
        &self,
        ctx: &Context,
        message: &serenity::model::channel::Message,
        discord_server_id: Option<serenity::model::id::GuildId>,
    ) {
        let author_name = {
            let mut display_name = message.author.display_name().to_string();

            if let Some(id) = discord_server_id {
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

        // There are like 50 different places an image can be in a message
        // ... for some reason
        let image_str: Option<&str> = {
            if let Some(image) = image {
                Some(image.url.as_str())
            } else if let Some(embed) = embed {
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
