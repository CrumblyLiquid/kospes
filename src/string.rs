use regex::Regex;

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
        let content = local_link_re.replace_all(&content, format!("[$2](https://courses.fit.cvut.cz/{}/$1)", course_code));

        content.to_string()
    }
}
