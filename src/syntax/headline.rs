use memchr::memrchr_iter;
use nom::{
    bytes::complete::take_while1,
    character::complete::{anychar, space0},
    combinator::{map, opt, verify},
    sequence::tuple,
    AsBytes, IResult, InputLength, InputTake, Slice,
};

use super::{
    combinator::{
        hash_token, l_bracket_token, line_starts_iter, node, r_bracket_token, token, trim_line_end,
        GreenElement, NodeBuilder,
    },
    drawer::property_drawer_node,
    element::element_nodes,
    input::Input,
    object::object_nodes,
    planning::planning_node,
    SyntaxKind::*,
};

#[tracing::instrument(level = "debug", skip(input), fields(input = input.s))]
pub fn headline_node(input: Input) -> IResult<Input, GreenElement, ()> {
    crate::lossless_parser!(headline_node_base, input)
}

fn headline_node_base(input: Input) -> IResult<Input, GreenElement, ()> {
    let (input, stars) = headline_stars(input)?;

    let mut b = NodeBuilder::new();

    b.token(HEADLINE_STARS, stars);

    let (input, ws) = space0(input)?;
    b.ws(ws);

    let (input, headline_keyword) = opt(headline_keyword_token)(input)?;

    if let Some((headline_keyword, ws)) = headline_keyword {
        b.push(headline_keyword);
        b.ws(ws);
    }

    let (input, headline_priority) = opt(headline_priority_node)(input)?;

    if let Some((headline_priority, ws)) = headline_priority {
        b.push(headline_priority);
        b.ws(ws);
    }

    let (input, (title_and_tags, ws_, nl)) = trim_line_end(input)?;
    let (title, tags) = opt(headline_tags_node)(title_and_tags)?;

    if !title.is_empty() {
        b.push(node(HEADLINE_TITLE, object_nodes(title)));
    }
    b.push_opt(tags);
    b.ws(ws_);
    b.nl(nl);

    if nl.is_empty() {
        return Ok((input, b.finish(HEADLINE)));
    }

    let (input, planning) = opt(planning_node)(input)?;
    b.push_opt(planning);

    let (input, property_drawer) = opt(property_drawer_node)(input)?;
    b.push_opt(property_drawer);

    let (input, section) = opt(section_node)(input)?;
    b.push_opt(section);

    let mut i = input;
    let current_level = stars.input_len();
    while !i.is_empty() {
        let next_level = i.bytes().take_while(|&c| c == b'*').count();

        if next_level <= current_level {
            break;
        }

        let (input, headline) = headline_node(i)?;
        b.push(headline);
        i = input;
    }

    Ok((i, b.finish(HEADLINE)))
}

#[tracing::instrument(level = "debug", skip(input), fields(input = input.s))]
pub fn section_node(input: Input) -> IResult<Input, GreenElement, ()> {
    let (input, section) = section_text(input)?;
    Ok((input, node(SECTION, element_nodes(section)?)))
}

pub fn section_text(input: Input) -> IResult<Input, Input, ()> {
    if input.is_empty() {
        return Err(nom::Err::Error(()));
    }

    for (input, section) in line_starts_iter(input.as_str()).map(|i| input.take_split(i)) {
        if headline_stars(input).is_ok() {
            if section.is_empty() {
                return Err(nom::Err::Error(()));
            }

            return Ok((input, section));
        }
    }

    Ok(input.take_split(input.input_len()))
}

#[tracing::instrument(level = "debug", skip(input), fields(input = input.s))]
fn headline_stars(input: Input) -> IResult<Input, Input, ()> {
    let bytes = input.as_bytes();
    let level = bytes.iter().take_while(|&&c| c == b'*').count();

    if level == 0 {
        Err(nom::Err::Error(()))
    } else if input.input_len() == level {
        Ok(input.take_split(level))
    } else if bytes[level] == b'\n' || bytes[level] == b'\r' || bytes[level] == b' ' {
        Ok(input.take_split(level))
    } else {
        Err(nom::Err::Error(()))
    }
}

#[tracing::instrument(level = "debug", skip(input), fields(input = input.s))]
fn headline_tags_node(input: Input) -> IResult<Input, GreenElement, ()> {
    if !input.s.ends_with(':') {
        return Err(nom::Err::Error(()));
    };

    let bytes = input.as_bytes();

    // we're going to skip to first colon, so we start from the
    // second last character
    let mut i = input.input_len() - 1;
    let mut can_not_be_ws = true;
    let mut children = vec![token(COLON, ":")];

    for ii in memrchr_iter(b':', bytes).skip(1) {
        let item = &bytes[ii + 1..i];

        if item.is_empty() {
            children.push(token(COLON, ":"));
            can_not_be_ws = false;
            i = ii;
        } else if item
            .iter()
            .all(|&c| c.is_ascii_alphanumeric() || c == b'_' || c == b'@' || c == b'#' || c == b'%')
        {
            children.push(input.slice(ii + 1..i).text_token());
            children.push(token(COLON, ":"));
            can_not_be_ws = false;
            i = ii;
        } else if item.iter().all(|&c| c == b' ' || c == b'\t') && !can_not_be_ws {
            children.push(input.slice(ii + 1..i).ws_token());
            children.push(token(COLON, ":"));
            can_not_be_ws = true;
            i = ii;
        } else {
            break;
        }
    }

    if children.len() == 1 {
        return Err(nom::Err::Error(()));
    }

    if i != 0 && bytes[i - 1] != b' ' && bytes[i - 1] != b'\t' {
        return Err(nom::Err::Error(()));
    }

    // we parse headline tag from right to left,
    // so we need to reverse the result after it finishes
    children.reverse();

    Ok((input.slice(0..i), node(HEADLINE_TAGS, children)))
}

fn headline_keyword_token(input: Input) -> IResult<Input, (GreenElement, Input), ()> {
    let (input, word) = verify(
        take_while1(|c: char| !c.is_ascii_whitespace()),
        |input: &Input| {
            let Input { c, s } = input;
            c.todo_keywords.0.iter().any(|k| k == s) || c.todo_keywords.1.iter().any(|k| k == s)
        },
    )(input)?;

    let (input, ws) = space0(input)?;

    Ok((input, (word.token(HEADLINE_KEYWORD), ws)))
}

fn headline_priority_node(input: Input) -> IResult<Input, (GreenElement, Input), ()> {
    let (input, node) = map(
        tuple((l_bracket_token, hash_token, anychar, r_bracket_token)),
        |(l_bracket, hash, char, r_bracket)| {
            node(
                HEADLINE_PRIORITY,
                [l_bracket, hash, token(TEXT, &char.to_string()), r_bracket],
            )
        },
    )(input)?;

    let (input, ws) = space0(input)?;

    Ok((input, (node, ws)))
}

#[test]
fn parse() {
    use crate::{ast::Headline, tests::to_ast};

    let to_headline = to_ast::<Headline>(headline_node);

    let hdl = to_headline("* foo");

    insta::assert_debug_snapshot!(
        hdl.syntax,
        @r###"
    HEADLINE@0..5
      HEADLINE_STARS@0..1 "*"
      WHITESPACE@1..2 " "
      HEADLINE_TITLE@2..5
        TEXT@2..5 "foo"
    "###
    );

    let hdl = to_headline("* foo\n\n** bar");
    insta::assert_debug_snapshot!(
        hdl.syntax,
        @r###"
    HEADLINE@0..13
      HEADLINE_STARS@0..1 "*"
      WHITESPACE@1..2 " "
      HEADLINE_TITLE@2..5
        TEXT@2..5 "foo"
      NEW_LINE@5..6 "\n"
      SECTION@6..7
        PARAGRAPH@6..7
          BLANK_LINE@6..7 "\n"
      HEADLINE@7..13
        HEADLINE_STARS@7..9 "**"
        WHITESPACE@9..10 " "
        HEADLINE_TITLE@10..13
          TEXT@10..13 "bar"
    "###
    );

    let hdl = to_headline("* TODO foo\nbar\n** baz\n");
    assert_eq!(hdl.level(), Some(1));
    assert_eq!(hdl.keyword().as_ref().map(|x| x.text()), Some("TODO"));
    insta::assert_debug_snapshot!(
        hdl.syntax,
        @r###"
    HEADLINE@0..22
      HEADLINE_STARS@0..1 "*"
      WHITESPACE@1..2 " "
      HEADLINE_KEYWORD@2..6 "TODO"
      WHITESPACE@6..7 " "
      HEADLINE_TITLE@7..10
        TEXT@7..10 "foo"
      NEW_LINE@10..11 "\n"
      SECTION@11..15
        PARAGRAPH@11..15
          TEXT@11..15 "bar\n"
      HEADLINE@15..22
        HEADLINE_STARS@15..17 "**"
        WHITESPACE@17..18 " "
        HEADLINE_TITLE@18..21
          TEXT@18..21 "baz"
        NEW_LINE@21..22 "\n"
    "###
    );

    let hdl = to_headline("** [#A] foo\n* baz");
    assert_eq!(hdl.level(), Some(2));
    assert_eq!(
        hdl.priority().unwrap().text_string().unwrap(),
        "A".to_string()
    );
    insta::assert_debug_snapshot!(
        hdl.syntax,
        @r###"
    HEADLINE@0..12
      HEADLINE_STARS@0..2 "**"
      WHITESPACE@2..3 " "
      HEADLINE_PRIORITY@3..7
        L_BRACKET@3..4 "["
        HASH@4..5 "#"
        TEXT@5..6 "A"
        R_BRACKET@6..7 "]"
      WHITESPACE@7..8 " "
      HEADLINE_TITLE@8..11
        TEXT@8..11 "foo"
      NEW_LINE@11..12 "\n"
    "###
    );
}

#[test]
fn issue_15_16() {
    use crate::{ast::Headline, tests::to_ast};

    let to_headline = to_ast::<Headline>(headline_node);

    let tags = to_headline("* a ::").tags().unwrap();
    assert_eq!(tags.iter().count(), 0);

    // let tags = to_headline("* a :(:").tags().unwrap();
    // assert_eq!(tags.iter().count(), 0);

    let tags = to_headline("* a \t:_:").tags().unwrap();
    assert_eq!(
        vec!["_".to_string()],
        tags.iter().map(|x| x.to_string()).collect::<Vec<_>>(),
    );

    let tags = to_headline("* a \t :@:").tags().unwrap();
    assert_eq!(
        vec!["@".to_string()],
        tags.iter().map(|x| x.to_string()).collect::<Vec<_>>(),
    );

    let tags = to_headline("* a :#:").tags().unwrap();
    assert_eq!(
        vec!["#".to_string()],
        tags.iter().map(|x| x.to_string()).collect::<Vec<_>>(),
    );

    let tags = to_headline("* a\t :%:").tags().unwrap();
    assert_eq!(
        vec!["%".to_string()],
        tags.iter().map(|x| x.to_string()).collect::<Vec<_>>(),
    );

    // let tags = to_headline("* a :余:").tags().unwrap();
    // assert_eq!(
    //     vec!["余".to_string()],
    //     tags.iter().map(|x| x.to_string()).collect::<Vec<_>>(),
    // );
}