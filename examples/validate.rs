use std::env;
use std::time::Duration;

use meta_threads::client::{Client, Config};
use meta_threads::types::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let access_token = env::var("THREADS_ACCESS_TOKEN").expect("THREADS_ACCESS_TOKEN required");
    let client_id = env::var("THREADS_CLIENT_ID").expect("THREADS_CLIENT_ID required");
    let client_secret = env::var("THREADS_CLIENT_SECRET").expect("THREADS_CLIENT_SECRET required");
    let redirect_uri = env::var("THREADS_REDIRECT_URI").expect("THREADS_REDIRECT_URI required");

    let config = Config::new(client_id, client_secret, redirect_uri);
    let client = Client::with_token(config, &access_token).await?;

    // Get user_id from token info
    let token_info = client.get_token_info().await.expect("token info");
    let user_id = UserId::from(token_info.user_id.as_str());

    println!("=== Read-Only Operations ===\n");

    // validate_token
    run(
        "validate_token",
        client.validate_token().await.map(|_| "valid".to_string()),
    );

    // get_token_debug_info
    print_test("get_token_debug_info");
    let debug_info = client.get_token_debug_info().await;
    pass(&format!(
        "masked={}",
        debug_info
            .get("access_token")
            .map(|s| s.as_str())
            .unwrap_or("(none)")
    ));

    // get_user
    print_test("get_user");
    match client.get_user(&user_id).await {
        Ok(user) => pass(&format!("@{}", user.username)),
        Err(e) => fail(&e),
    }

    // get_user_with_fields
    print_test("get_user_with_fields");
    match client
        .get_user_with_fields(&user_id, &["id", "username"])
        .await
    {
        Ok(user) => pass(&format!("@{}", user.username)),
        Err(e) => fail(&e),
    }

    // get_me
    print_test("get_me");
    match client.get_me().await {
        Ok(user) => pass(&format!("@{}", user.username)),
        Err(e) => fail(&e),
    }

    // get_user_posts
    print_test("get_user_posts");
    let opts = PostsOptions {
        limit: Some(3),
        ..Default::default()
    };
    match client.get_user_posts(&user_id, Some(&opts)).await {
        Ok(resp) => pass(&format!("{} posts", resp.data.len())),
        Err(e) => fail(&e),
    }

    // get_post + get_post_insights on first post
    let posts = client.get_user_posts(&user_id, Some(&opts)).await.ok();
    if let Some(ref resp) = posts {
        if let Some(first) = resp.data.first() {
            print_test("get_post");
            match client.get_post(&first.id).await {
                Ok(post) => pass(&format!("{}: {}", post.id, trunc(post.text.as_deref()))),
                Err(e) => fail(&e),
            }

            print_test("get_post_insights");
            match client
                .get_post_insights(&first.id, &["views", "likes"])
                .await
            {
                Ok(ins) => pass(&format!("{} metrics", ins.data.len())),
                Err(e) => fail(&e),
            }

            print_test("get_post_insights_with_options");
            let iopts = PostInsightsOptions {
                metrics: Some(vec![PostInsightMetric::Views, PostInsightMetric::Likes]),
                ..Default::default()
            };
            match client
                .get_post_insights_with_options(&first.id, &iopts)
                .await
            {
                Ok(ins) => pass(&format!("{} metrics", ins.data.len())),
                Err(e) => fail(&e),
            }

            run(
                "get_replies",
                client
                    .get_replies(&first.id, None)
                    .await
                    .map(|r| format!("{} replies", r.data.len())),
            );
            run(
                "get_conversation",
                client
                    .get_conversation(&first.id, None)
                    .await
                    .map(|r| format!("{} msgs", r.data.len())),
            );
        }
    }

    // account insights
    print_test("get_account_insights");
    match client
        .get_account_insights(&user_id, &["views", "likes"], "lifetime")
        .await
    {
        Ok(ins) => pass(&format!("{} metrics", ins.data.len())),
        Err(e) => fail(&e),
    }

    print_test("get_account_insights_with_options");
    let aopts = AccountInsightsOptions {
        metrics: Some(vec![
            AccountInsightMetric::Views,
            AccountInsightMetric::FollowersCount,
        ]),
        period: Some(InsightPeriod::Day),
        ..Default::default()
    };
    match client
        .get_account_insights_with_options(&user_id, &aopts)
        .await
    {
        Ok(ins) => pass(&format!("{} metrics", ins.data.len())),
        Err(e) => fail(&e),
    }

    // publishing limits
    print_test("get_publishing_limits");
    match client.get_publishing_limits().await {
        Ok(lim) => pass(&format!(
            "posts={}/{}, replies={}/{}",
            lim.quota_usage,
            lim.config.quota_total,
            lim.reply_quota_usage,
            lim.reply_config.quota_total
        )),
        Err(e) => fail(&e),
    }

    // rate limit local ops
    run(
        "rate_limit_status",
        client
            .rate_limit_status()
            .await
            .map(|s| format!("limit={}, remaining={}", s.limit, s.remaining))
            .ok_or("no limiter"),
    );
    client.disable_rate_limiting().await;
    client.enable_rate_limiting().await;
    run(
        "wait_for_rate_limit",
        client.wait_for_rate_limit().await.map(|_| "ok".to_string()),
    );

    // enum helpers
    print_test("enum all()");
    assert_eq!(PostInsightMetric::all().len(), 6);
    assert_eq!(AccountInsightMetric::all().len(), 8);
    assert_eq!(InsightPeriod::all().len(), 2);
    assert_eq!(FollowerDemographicsBreakdown::all().len(), 4);
    pass("counts correct");

    // =========================================================================
    println!("\n=== Write Operations (create + delete) ===\n");
    // =========================================================================

    // --- Text post ---
    print_test("create + delete text post");
    let ts = now_str();
    let content = TextPostContent {
        text: format!("Rust integration test post {ts}"),
        reply_control: Some(ReplyControl::Everyone),
        auto_publish_text: false,
        link_attachment: None,
        poll_attachment: None,
        reply_to_id: None,
        topic_tag: None,
        allowlisted_country_codes: None,
        location_id: None,
        quoted_post_id: None,
        text_entities: None,
        text_attachment: None,
        gif_attachment: None,
        is_ghost_post: false,
        enable_reply_approvals: false,
    };
    match client.create_text_post(&content).await {
        Ok(post) => {
            pass(&format!("created {}", post.id));
            sleep(2).await;
            cleanup(&client, &post.id).await;
        }
        Err(e) => fail(&e),
    }

    // --- Text post with spoiler entities ---
    print_test("create + delete text post with spoiler");
    let content = TextPostContent {
        text: "Spoiler alert: Darth Vader is Luke's father!".into(),
        text_entities: Some(vec![TextEntity {
            entity_type: "SPOILER".into(),
            offset: 15,
            length: 11,
        }]),
        auto_publish_text: false,
        link_attachment: None,
        poll_attachment: None,
        reply_control: None,
        reply_to_id: None,
        topic_tag: None,
        allowlisted_country_codes: None,
        location_id: None,
        quoted_post_id: None,
        text_attachment: None,
        gif_attachment: None,
        is_ghost_post: false,
        enable_reply_approvals: false,
    };
    match client.create_text_post(&content).await {
        Ok(post) => {
            pass(&format!("created {} with spoiler entity", post.id));
            sleep(2).await;
            cleanup(&client, &post.id).await;
        }
        Err(e) => fail(&e),
    }

    // --- Image post ---
    print_test("create + delete image post");
    let content = ImagePostContent {
        text: Some(format!("Rust integration test image {}", now_str())),
        image_url: "https://picsur.threadsutil.cc/i/21d9256b-52f4-48c4-8093-3be5022c111a.jpg"
            .into(),
        alt_text: Some("Integration test image".into()),
        reply_control: None,
        reply_to_id: None,
        topic_tag: None,
        allowlisted_country_codes: None,
        location_id: None,
        quoted_post_id: None,
        text_entities: None,
        is_spoiler_media: false,
        enable_reply_approvals: false,
    };
    match client.create_image_post(&content).await {
        Ok(post) => {
            pass(&format!("created {}", post.id));
            sleep(2).await;
            cleanup(&client, &post.id).await;
        }
        Err(e) => fail(&e),
    }

    // --- Image post with media spoiler ---
    print_test("create + delete image post with media spoiler");
    let content = ImagePostContent {
        text: Some(format!("Rust spoiler image test {}", now_str())),
        image_url: "https://picsur.threadsutil.cc/i/21d9256b-52f4-48c4-8093-3be5022c111a.jpg"
            .into(),
        alt_text: Some("Spoiler image".into()),
        is_spoiler_media: true,
        reply_control: None,
        reply_to_id: None,
        topic_tag: None,
        allowlisted_country_codes: None,
        location_id: None,
        quoted_post_id: None,
        text_entities: None,
        enable_reply_approvals: false,
    };
    match client.create_image_post(&content).await {
        Ok(post) => {
            pass(&format!("created {}", post.id));
            sleep(2).await;
            cleanup(&client, &post.id).await;
        }
        Err(e) => fail(&e),
    }

    // --- Quote post ---
    print_test("create + delete quote post");
    // First create a post to quote
    let original = TextPostContent {
        text: format!("Original post to quote {}", now_str()),
        auto_publish_text: false,
        link_attachment: None,
        poll_attachment: None,
        reply_control: None,
        reply_to_id: None,
        topic_tag: None,
        allowlisted_country_codes: None,
        location_id: None,
        quoted_post_id: None,
        text_entities: None,
        text_attachment: None,
        gif_attachment: None,
        is_ghost_post: false,
        enable_reply_approvals: false,
    };
    match client.create_text_post(&original).await {
        Ok(orig_post) => {
            sleep(2).await;
            let quote_text = format!("Quoting a post {}", now_str());
            match client.create_quote_post(&quote_text, &orig_post.id).await {
                Ok(quote_post) => {
                    pass(&format!(
                        "quote {} -> original {}",
                        quote_post.id, orig_post.id
                    ));
                    sleep(2).await;
                    cleanup(&client, &quote_post.id).await;
                }
                Err(e) => fail(&e),
            }
            sleep(1).await;
            cleanup(&client, &orig_post.id).await;
        }
        Err(e) => fail(&e),
    }

    // --- Carousel post ---
    print_test("create + delete carousel post");
    let c1 = client
        .create_media_container(
            "IMAGE",
            "https://picsur.threadsutil.cc/i/21d9256b-52f4-48c4-8093-3be5022c111a.jpg",
            Some("Carousel image 1"),
        )
        .await;
    let c2 = client
        .create_media_container(
            "IMAGE",
            "https://picsur.threadsutil.cc/i/ae8bbb4e-a8dd-4c2b-ae9e-3103b8e26285.jpg",
            Some("Carousel image 2"),
        )
        .await;
    match (c1, c2) {
        (Ok(cid1), Ok(cid2)) => {
            // Poll until both containers are ready
            wait_container_ready(&client, &cid1).await;
            wait_container_ready(&client, &cid2).await;
            let content = CarouselPostContent {
                text: Some(format!("Rust carousel test {}", now_str())),
                children: vec![cid1, cid2],
                reply_control: None,
                reply_to_id: None,
                topic_tag: None,
                allowlisted_country_codes: None,
                location_id: None,
                quoted_post_id: None,
                text_entities: None,
                is_spoiler_media: false,
                enable_reply_approvals: false,
            };
            match client.create_carousel_post(&content).await {
                Ok(post) => {
                    pass(&format!("created {}", post.id));
                    sleep(2).await;
                    cleanup(&client, &post.id).await;
                }
                Err(e) => fail(&e),
            }
        }
        (Err(e), _) | (_, Err(e)) => fail(&e),
    }

    // --- Repost ---
    print_test("create + delete repost");
    let original = TextPostContent {
        text: format!("Post to repost {}", now_str()),
        auto_publish_text: false,
        link_attachment: None,
        poll_attachment: None,
        reply_control: None,
        reply_to_id: None,
        topic_tag: None,
        allowlisted_country_codes: None,
        location_id: None,
        quoted_post_id: None,
        text_entities: None,
        text_attachment: None,
        gif_attachment: None,
        is_ghost_post: false,
        enable_reply_approvals: false,
    };
    match client.create_text_post(&original).await {
        Ok(orig_post) => {
            sleep(2).await;
            match client.repost_post(&orig_post.id).await {
                Ok(repost_id) => {
                    pass(&format!(
                        "repost {} -> original {}",
                        repost_id, orig_post.id
                    ));
                    sleep(2).await;
                    cleanup(&client, &repost_id).await;
                }
                Err(e) => fail(&e),
            }
            sleep(1).await;
            cleanup(&client, &orig_post.id).await;
        }
        Err(e) => fail(&e),
    }

    // --- Reply to post ---
    print_test("reply_to_post + delete");
    let original = TextPostContent {
        text: format!("Post to reply to {}", now_str()),
        auto_publish_text: false,
        link_attachment: None,
        poll_attachment: None,
        reply_control: None,
        reply_to_id: None,
        topic_tag: None,
        allowlisted_country_codes: None,
        location_id: None,
        quoted_post_id: None,
        text_entities: None,
        text_attachment: None,
        gif_attachment: None,
        is_ghost_post: false,
        enable_reply_approvals: false,
    };
    match client.create_text_post(&original).await {
        Ok(orig_post) => {
            sleep(2).await;
            match client
                .reply_to_post(&orig_post.id, &format!("Reply test {}", now_str()))
                .await
            {
                Ok(reply) => {
                    pass(&format!("reply {} -> parent {}", reply.id, orig_post.id));
                    sleep(2).await;
                    cleanup(&client, &reply.id).await;
                }
                Err(e) => fail(&e),
            }
            sleep(1).await;
            cleanup(&client, &orig_post.id).await;
        }
        Err(e) => fail(&e),
    }

    println!("\n--- done ---");
    Ok(())
}

fn now_str() -> String {
    chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

async fn sleep(secs: u64) {
    tokio::time::sleep(Duration::from_secs(secs)).await;
}

async fn wait_container_ready(client: &Client, cid: &ContainerId) {
    for _ in 0..30 {
        match client.get_container_status(cid).await {
            Ok(status) if status.status == "FINISHED" || status.status == "PUBLISHED" => return,
            Ok(status) if status.status == "ERROR" || status.status == "EXPIRED" => {
                println!("    container {cid} failed: {}", status.status);
                return;
            }
            _ => sleep(1).await,
        }
    }
    println!("    container {cid} did not finish in time");
}

async fn cleanup(client: &Client, post_id: &PostId) {
    match client.delete_post(post_id).await {
        Ok(id) => println!("    deleted {id}"),
        Err(e) => println!("    delete {post_id} failed: {e}"),
    }
}

fn trunc(s: Option<&str>) -> String {
    s.unwrap_or("(no text)").chars().take(50).collect()
}

fn print_test(name: &str) {
    print!("  {name} ... ");
}

fn pass(detail: &str) {
    println!("OK ({detail})");
}

fn fail(e: &dyn std::fmt::Display) {
    println!("FAIL ({e})");
}

fn run<T: std::fmt::Display, E: std::fmt::Display>(name: &str, result: Result<T, E>) {
    print_test(name);
    match result {
        Ok(v) => pass(&v.to_string()),
        Err(e) => fail(&e),
    }
}
