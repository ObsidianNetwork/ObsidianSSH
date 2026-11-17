//! Editable tags shared by the host and macro forms.

#[derive(Debug, Clone, Default)]
pub(crate) struct TagInput {
    pub(crate) tags: Vec<String>,
    pub(crate) draft: String,
}

#[derive(Debug, Clone)]
pub(crate) enum TagInputMessage {
    Changed(String),
    Commit,
    Add(String),
    Remove(String),
}

impl From<Vec<String>> for TagInput {
    fn from(tags: Vec<String>) -> Self {
        Self { tags, draft: String::new() }
    }
}

impl TagInput {
    pub(crate) fn update(&mut self, message: TagInputMessage) {
        match message {
            TagInputMessage::Changed(value) => {
                // Commas remain useful for pasting lists; ordinary typing
                // stays a draft until Enter or the Add button.
                if let Some((complete, draft)) = value.rsplit_once(',') {
                    self.add(complete);
                    self.draft = draft.trim_start().to_string();
                } else {
                    self.draft = value;
                }
            }
            TagInputMessage::Commit => {
                let draft = std::mem::take(&mut self.draft);
                self.add(&draft);
            }
            TagInputMessage::Add(tag) => self.add(&tag),
            TagInputMessage::Remove(tag) => self.tags.retain(|t| t != &tag),
        }
    }

    fn add(&mut self, value: &str) {
        for tag in crate::util::parse_tags(value) {
            if !self.tags.iter().any(|t| t.eq_ignore_ascii_case(&tag)) {
                self.tags.push(tag);
            }
        }
    }

    /// Save includes the unfinished tag without changing the input mid-edit.
    pub(crate) fn values(&self) -> Vec<String> {
        let mut complete = self.clone();
        complete.update(TagInputMessage::Commit);
        complete.tags
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn committing_trims_and_deduplicates_existing_tags() {
        let mut input = TagInput::from(vec!["Prod".into()]);
        input.update(TagInputMessage::Changed(" prod, web ,WEB, ".into()));
        input.update(TagInputMessage::Commit);
        assert_eq!(input.tags, ["Prod", "web"]);
        assert!(input.draft.is_empty());
    }

    #[test]
    fn saving_keeps_unfinished_tag_without_changing_the_draft() {
        let mut input = TagInput::from(vec!["prod".into()]);
        input.update(TagInputMessage::Changed(" web ".into()));
        assert_eq!(input.values(), ["prod", "web"]);
        assert_eq!(input.draft, " web ");
        assert_eq!(input.tags, ["prod"]);
    }

    #[test]
    fn suggestions_and_removal_preserve_unfinished_text() {
        let mut input = TagInput::from(vec!["prod".into()]);
        input.update(TagInputMessage::Changed("database".into()));
        input.update(TagInputMessage::Add("web".into()));
        input.update(TagInputMessage::Remove("prod".into()));
        assert_eq!(input.values(), ["web", "database"]);
        assert_eq!(input.draft, "database");
    }
}
