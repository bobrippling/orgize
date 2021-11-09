use rowan::ast::support;

use crate::syntax::{SyntaxElement, SyntaxKind, SyntaxToken};

use super::{filter_token, Headline, HeadlinePriority, HeadlineTags, Timestamp};

impl Headline {
    /// Return level of this headline
    ///
    /// ```rust
    /// use orgize::{Org, ast::Headline};
    ///
    /// let hdl = Org::parse("* ").first_node::<Headline>().unwrap();
    /// assert_eq!(hdl.level(), Some(1));
    /// let hdl = Org::parse("****** hello").first_node::<Headline>().unwrap();
    /// assert_eq!(hdl.level(), Some(6));
    /// ```
    pub fn level(&self) -> Option<usize> {
        self.stars().map(|stars| stars.text().len())
    }

    /// Return `true` if this headline contains a COMMENT keyword
    ///      
    /// ```rust
    /// use orgize::{Org, ast::Headline};
    ///
    /// let hdl = Org::parse("* COMMENT").first_node::<Headline>().unwrap();
    /// assert!(hdl.is_commented());
    /// let hdl = Org::parse("* COMMENT hello").first_node::<Headline>().unwrap();
    /// assert!(hdl.is_commented());
    /// let hdl = Org::parse("* hello").first_node::<Headline>().unwrap();
    /// assert!(!hdl.is_commented());
    /// ```
    pub fn is_commented(&self) -> bool {
        self.title()
            .and_then(|title| title.syntax.first_token())
            .map(|title| {
                let text = title.text();
                title.kind() == SyntaxKind::TEXT
                    && text.starts_with("COMMENT")
                    && (text.len() == 7 || text[7..].starts_with(char::is_whitespace))
            })
            .unwrap_or_default()
    }

    /// Return `true` if this headline contains an archive tag
    ///
    /// ```rust
    /// use orgize::{Org, ast::Headline};
    ///
    /// let hdl = Org::parse("* hello :ARCHIVE:").first_node::<Headline>().unwrap();
    /// assert!(hdl.is_archived());
    /// let hdl = Org::parse("* hello :ARCHIVED:").first_node::<Headline>().unwrap();
    /// assert!(!hdl.is_archived());
    /// ```
    pub fn is_archived(&self) -> bool {
        self.tags()
            .map(|tags| {
                tags.syntax
                    .children_with_tokens()
                    .any(|elem| matches!(elem, SyntaxElement::Token(t) if t.text() == "ARCHIVE"))
            })
            .unwrap_or_default()
    }

    /// Returns this headline's closed timestamp, or `None` if not set.
    pub fn closed(&self) -> Option<Timestamp> {
        self.planning()
            .and_then(|planning| planning.closed())
            .and_then(|node| support::child::<Timestamp>(&node.syntax))
    }

    /// Returns this headline's scheduled timestamp, or `None` if not set.
    pub fn scheduled(&self) -> Option<Timestamp> {
        self.planning()
            .and_then(|planning| planning.scheduled())
            .and_then(|node| support::child::<Timestamp>(&node.syntax))
    }

    /// Returns this headline's deadline timestamp, or `None` if not set.
    pub fn deadline(&self) -> Option<Timestamp> {
        self.planning()
            .and_then(|planning| planning.deadline())
            .and_then(|node| support::child::<Timestamp>(&node.syntax))
    }
}

// pub enum DocumentOrHeadline {
//     Document(Document),
//     Headline(Headline),
// }

// impl From<Document> for DocumentOrHeadline {
//     fn from(value: Document) -> Self {
//         DocumentOrHeadline::Document(value)
//     }
// }

// impl From<Headline> for DocumentOrHeadline {
//     fn from(value: Headline) -> Self {
//         DocumentOrHeadline::Headline(value)
//     }
// }

// impl DocumentOrHeadline {
//     pub fn section(&self) -> Option<Section> {
//         match self {
//             DocumentOrHeadline::Document(v) => v.section(),
//             DocumentOrHeadline::Headline(v) => v.section(),
//         }
//     }
// }

// impl Org {
//     /// set the title of this headline
//     ///
//     /// ```rust
//     /// use orgize::Org;
//     ///
//     /// let mut org = Org::parse("* [#A]");
//     /// let hdl = org.document().first_headline().unwrap();
//     /// org.set_title(hdl, "world");
//     /// assert_eq!(org.to_org(), "* [#A] world");
//     /// let hdl = org.document().first_headline().unwrap();
//     /// org.set_title(hdl, "world!");
//     /// assert_eq!(org.to_org(), "* [#A] world!");
//     /// ```
//     pub fn set_title(&mut self, headline: Headline, title: &str) -> Option<HeadlineTitle> {
//         let bytes = title.as_bytes();
//         let title = match memchr(b'\n', bytes) {
//             Some(i) if i > 0 && bytes[i] == b'\r' => &title[0..i - 1],
//             Some(i) => &title[0..i],
//             _ => title,
//         };
//         let new_title = node(HEADLINE_TITLE, object_nodes(self.create_input(title)));

//         if let Some(title) = headline.title() {
//             self.green = title.syntax.replace_with(new_title.into_node().unwrap());

//             return Some(title);
//         }

//         let mut child: Vec<_> = headline
//             .syntax
//             .green()
//             .children()
//             .map(|ch| ch.to_owned())
//             .collect();

//         let index = support::child
//             .iter()
//             .enumerate()
//             .filter_map(|(idx, it)| {
//                 if it.kind() == HEADLINE_STARS.into()
//                     || it.kind() == HEADLINE_KEYWORD.into()
//                     || it.kind() == HEADLINE_PRIORITY.into()
//                 {
//                     Some(idx + 1)
//                 } else {
//                     None
//                 }
//             })
//             .last()
//             .unwrap_or_default();

//         if index == child.len() {
//             child.push(token(WHITESPACE, " "));
//             child.push(new_title);
//         } else if child[index].kind() != WHITESPACE.into() {
//             child.insert(index, token(WHITESPACE, " "));
//             child.insert(index + 1, new_title);
//         } else {
//             child.insert(index, new_title);
//         }

//         self.green = headline
//             .syntax
//             .replace_with(node(HEADLINE, child).into_node().unwrap());

//         None
//     }

//     /// set the section of this document or headline
//     ///
//     /// ```rust
//     /// use orgize::Org;
//     ///
//     /// let mut org = Org::parse("* hello");
//     ///
//     /// let hdl = org.document().first_headline().unwrap();
//     /// org.set_section(hdl, "world");
//     /// assert_eq!(org.to_org(), "* hello\nworld\n");
//     ///
//     /// let hdl = org.document().first_headline().unwrap();
//     /// org.set_section(hdl, "world!");
//     /// assert_eq!(org.to_org(), "* hello\nworld!\n");
//     ///
//     /// let doc = org.document();
//     /// org.set_section(doc, "doc");
//     /// assert_eq!(org.to_org(), "doc\n* hello\nworld!\n");
//     /// ```
//     pub fn set_section(
//         &mut self,
//         document_or_headline: impl Into<DocumentOrHeadline>,
//         section: &str,
//     ) -> Option<Section> {
//         let document_or_headline = document_or_headline.into();

//         let section = section_text(self.create_input(section)).ok()?.1.as_str();

//         let section = if section.ends_with('\n') {
//             section_node(self.create_input(section)).map(|(_, s)| s)
//         } else {
//             section_node(self.create_input(&format!("{section}\n"))).map(|(_, s)| s)
//         }
//         .ok()?;

//         if let Some(old) = document_or_headline.section() {
//             self.green = old.syntax.replace_with(section.into_node().unwrap());

//             return Some(old);
//         }

//         match document_or_headline {
//             DocumentOrHeadline::Document(document) => {
//                 let mut child: Vec<_> = document
//                     .syntax
//                     .green()
//                     .children()
//                     .map(|ch| ch.to_owned())
//                     .collect();

//                 let headline_idx = child.iter().position(|it| it.kind() == HEADLINE.into());

//                 if let Some(idx) = headline_idx {
//                     child.insert(idx, section);
//                 } else {
//                     child.push(section);
//                 }

//                 self.green = document
//                     .syntax
//                     .replace_with(GreenNode::new(DOCUMENT.into(), child));

//                 None
//             }
//             DocumentOrHeadline::Headline(headline) => {
//                 let mut child: Vec<_> = headline
//                     .syntax
//                     .green()
//                     .children()
//                     .map(|ch| ch.to_owned())
//                     .collect();

//                 let new_line_idx = support::child
//                     .iter()
//                     .position(|it| it.kind() == NEW_LINE.into());

//                 if let Some(idx) = new_line_idx {
//                     // add section *after* newline
//                     if idx < support::child.len() {
//                         support::child.insert(idx, section);
//                     } else {
//                         support::child.push(section);
//                     }
//                 } else {
//                     support::child.push(token(NEW_LINE, "\n"));
//                     support::child.push(section);
//                 }

//                 self.green = headline
//                     .syntax
//                     .replace_with(GreenNode::new(HEADLINE.into(), support::child));

//                 None
//             }
//         }
//     }

//     /// set the level of this headline
//     ///
//     /// ```rust
//     /// use orgize::Org;
//     ///
//     /// let mut org = Org::parse("** 1\n** 2");
//     ///
//     /// let hdl = org.document().last_headline().unwrap();
//     /// org.set_level(hdl, 1);
//     /// assert_eq!(org.to_org(), "** 1\n* 2");
//     ///
//     /// let hdl = org.document().last_headline().unwrap();
//     /// org.set_level(hdl, 3);
//     /// assert_eq!(org.to_org(), "** 1\n* 2");
//     /// ```
//     pub fn set_level(&mut self, headline: Headline, level: usize) {
//         if level == 0 {
//             return;
//         }

//         let min_level_in_siblings = headline
//             .syntax
//             .siblings(rowan::Direction::Next)
//             .chain(headline.syntax.siblings(rowan::Direction::Prev))
//             .filter_map(Headline::cast)
//             .filter_map(|headline| headline.level())
//             .min()
//             .unwrap_or(1);

//         if level <= min_level_in_siblings {
//             if let Some(stars) = headline.stars() {
//                 self.green = stars.replace_with(GreenToken::new(
//                     SyntaxKind::HEADLINE_STARS.into(),
//                     "*".repeat(level).as_str(),
//                 ));
//             }
//         }
//     }
// }

impl HeadlineTags {
    /// Returns an iterator of text token in this tags
    ///
    /// ```rust
    /// use orgize::{Org, ast::HeadlineTags};
    ///
    /// let tags_vec = |input: &str| {
    ///     let tags = Org::parse(input).first_node::<HeadlineTags>().unwrap();
    ///     let tags: Vec<_> = tags.iter().map(|t| t.to_string()).collect();
    ///     tags
    /// };
    ///
    /// assert_eq!(tags_vec("* :tag:"), vec!["tag".to_string()]);
    /// assert_eq!(tags_vec("* [#A] :::::a2%:"), vec!["a2%".to_string()]);
    /// assert_eq!(tags_vec("* TODO :tag:  :a2%:"), vec!["tag".to_string(), "a2%".to_string()]);
    /// assert_eq!(tags_vec("* title :tag:a2%:"), vec!["tag".to_string(), "a2%".to_string()]);
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = SyntaxToken> {
        self.syntax
            .children_with_tokens()
            .filter_map(filter_token(SyntaxKind::TEXT))
    }
}

impl HeadlinePriority {
    /// Returns priority text
    ///
    /// ```rust
    /// use orgize::{Org, ast::HeadlinePriority};
    ///
    /// let priority = Org::parse("* [#A]").first_node::<HeadlinePriority>().unwrap();
    /// assert_eq!(priority.text_string().unwrap(), "A".to_string());
    /// let priority = Org::parse("* [#破]").first_node::<HeadlinePriority>().unwrap();
    /// assert_eq!(priority.text_string().unwrap(), "破".to_string());
    /// ```
    pub fn text_string(&self) -> Option<String> {
        self.text().map(|tk| tk.to_string())
    }
}
