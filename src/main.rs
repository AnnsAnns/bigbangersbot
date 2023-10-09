use std::env;

use serenity::{async_trait, builder::CreateEmbedAuthor};

use serenity::model::gateway::Ready;
use serenity::prelude::*;
extern crate dotenv;

use dotenv::dotenv;
struct Handler {
    channel_id: u64,
    server_id: u64,
    reaction_threshold: u64,
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
                .get_guild(self.server_id)
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

                    if star_reaction.count < self.reaction_threshold {
                        continue;
                    }

                    let author_name = &message
                        .author
                        .nick_in(&ctx.http, self.server_id)
                        .await
                        .unwrap_or(message.author.name.clone());

                    let msg_url = &message.link_ensured(&ctx.http).await;
                    let channel = ctx.http.get_channel(self.channel_id).await.unwrap().id();
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

            // Sleep for 5 minutes
            tokio::time::sleep(std::time::Duration::from_secs(60 * 5)).await;
        }
    }
}

#[tokio::main]
async fn main() {
    // Load the environment variables from the .env file.
    dotenv().ok();

    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let channel_id = env::var("CHANNEL_ID").expect("Expected a channel id in the environment");
    let server_id = env::var("SERVER_ID").expect("Expected a server id in the environment");
    let reaction_threshold =
        env::var("REACTION_THRESHOLD").expect("Expected a reaction threshold in the environment");

    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::DIRECT_MESSAGES;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler {
            channel_id: channel_id.parse::<u64>().unwrap(),
            server_id: server_id.parse::<u64>().unwrap(),
            reaction_threshold: reaction_threshold.parse::<u64>().unwrap(),
        })
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
