use serde::{Serialize, Deserialize};
use serenity::{async_trait, builder::CreateEmbedAuthor};

use serenity::model::gateway::Ready;
use serenity::prelude::*;

struct Handler {
    config: Config,
}


#[derive(Serialize, Deserialize, Debug)]
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
}

const REACTION_EMOJI: &str = "⭐";
const APPROVED_EMOJI: &str = "🌠";

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        let approved_emoji =
            serenity::model::channel::ReactionType::Unicode(APPROVED_EMOJI.to_string());
        let reaction_emoji =
            serenity::model::channel::ReactionType::Unicode(REACTION_EMOJI.to_string());
        let own_id = &ctx.http.get_current_user().await.unwrap().id;

        loop {
            let channels = ctx
                .http
                .get_guild(self.config.discord_server)
                .await
                .unwrap()
                .channels(&ctx.http)
                .await
                .unwrap();

            for channel in channels {
                if channel.1.kind != serenity::model::channel::ChannelType::Text {
                    continue;
                }
                let messages_retriever = channel
                    .1
                    .messages(&ctx.http, |retriever| retriever.limit(100))
                    .await;

                let messages = match messages_retriever {
                    Ok(messages) => messages,
                    Err(_) => continue,
                };

                for message in messages {
                    if message.author.id == *own_id {
                        continue;
                    }

                    let approved_reactions = message
                        .reaction_users(&ctx.http, approved_emoji.clone(), None, None)
                        .await;

                    if approved_reactions.is_err()
                        || approved_reactions
                            .unwrap()
                            .iter()
                            .find(|user| user.id == *own_id)
                            .is_some()
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

                    if star_reaction.count < self.config.threshold {
                        continue;
                    }

                    let author_name = &message
                        .author
                        .nick_in(&ctx.http, self.config.discord_server)
                        .await
                        .unwrap_or(message.author.name.clone());

                    let msg_url = &message.link_ensured(&ctx.http).await;
                    let channel = ctx.http.get_channel(self.config.discord_channel).await.unwrap().id();
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

                                let content = if message.content.is_empty() && embed.is_some() {
                                    embed.unwrap().description.clone().unwrap_or_default()
                                } else {
                                    message.content.clone()
                                };

                                e.description(format!(
                                    "{}\n\n👉 [Original Message]({})",
                                    content, msg_url
                                ));
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
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    // Parse channels from config
    let config: Config = serde_json::from_str(&std::fs::read_to_string("config.json").unwrap())
        .expect("Failed to parse config.json");

    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::DIRECT_MESSAGES;

    let mut client = Client::builder(&config.discord_token, intents)
        .event_handler(Handler {
            config,
        })
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
