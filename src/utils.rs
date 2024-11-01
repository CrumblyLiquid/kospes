use regex::Regex;
use serenity::all::CreateEmbed;
use sqlx::SqlitePool;

pub trait ToMarkdown {
    /// Convert a container containing HTML tags to a container
    /// containing Discord Markdown
    fn to_md(&self, subject: &str) -> Self;
}

impl ToMarkdown for String {
    fn to_md(&self, subject: &str) -> String {
        let erase_re = Regex::new(r#"</?p>"#).expect("Invalid regex");
        let content = erase_re.replace_all(self, "");

        let bold_re = Regex::new(r#"</?(b|strong)>"#).expect("Invalid regex");
        let content = bold_re.replace_all(&content, "**");

        let italic_re = Regex::new(r#"</?(i|em)>"#).expect("Invalid regex");
        let content = italic_re.replace_all(&content, "__");

        // The order here matters
        // First we only replace links with normal URLs
        let link_re = Regex::new(r#"<a .*href="(https?://(www\.)?[-a-zA-Z0-9@:%._\+~#=]{2,256}\.[a-z]{2,4}\b([-a-zA-Z0-9@:%_\+.~#?&//=]*))">(.*)</a>"#).expect("Invalid regex");
        let content = link_re.replace_all(&content, "[$4]($1)");

        // Then we replace local links
        let local_link_re = Regex::new(r#"<a .*href="(.*)">(.*)</a>"#).expect("Invalid regex");

        let course_code = subject.replace(".21", "");

        // TODO: Escape the subject in format!
        let content = local_link_re.replace_all(
            &content,
            format!("[$2](https://courses.fit.cvut.cz/{}/$1)", course_code),
        );

        content.to_string()
    }
}

pub trait Trackable {
    async fn is_new(&self, pool: &SqlitePool) -> Option<bool>;

    async fn is_new_named(&self, pool: &SqlitePool, name: &str, id: &str) -> Option<bool> {
        let query = format!("SELECT EXISTS (SELECT * FROM {} WHERE id = $1)", name);
        let query_res = sqlx::query_as::<_, (u32,)>(&query)
            .bind(id)
            .fetch_one(pool)
            .await;

        match query_res {
            Ok(count) => {
                // News with that ID in the database doesn't exists
                if count.0 == 0 {
                    // So we should add it into the database
                    if let Err(e) = sqlx::query("INSERT INTO seen_news (id) VALUES ($1);")
                        .bind(id)
                        .execute(pool)
                        .await
                    {
                        println!("Failed to add News (id: {}) to seen_news table: {e}", id);

                        // We failed to save the information into the database
                        return None;
                    };

                    Some(true)
                // There is a row with that ID so this News isn't new
                } else {
                    Some(false)
                }
            }
            Err(e) => {
                println!("Failed to run News::is_new SQL query: {e}");
                None
            }
        }
    }
}

pub trait IntoEmbed {
    fn embed(&self, subject: &str) -> CreateEmbed;
}
