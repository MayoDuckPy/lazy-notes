use crux_core::{
    capability::{CapabilityContext, Operation},
    macros::Capability,
};
use html5ever::{
    tendril::StrTendril,
    tokenizer::{BufferQueue, Tag, TagKind, Token, TokenSink, TokenSinkResult, Tokenizer},
    ATOM_LOCALNAME__61 as TOKEN_A,
    ATOM_LOCALNAME__68_31 as TOKEN_H1,
    ATOM_LOCALNAME__68_32 as TOKEN_H2,
    ATOM_LOCALNAME__68_33 as TOKEN_H3,
    ATOM_LOCALNAME__68_34 as TOKEN_H4,
    ATOM_LOCALNAME__68_35 as TOKEN_H5,
    ATOM_LOCALNAME__68_36 as TOKEN_H6,
    // ATOM_LOCALNAME__6C_69 as TOKEN_LI,
    // ATOM_LOCALNAME__75_69 as TOKEN_UL,
    ATOM_LOCALNAME__70 as TOKEN_P,
};
use serde::{Deserialize, Serialize};

/* App Logic:
    - HTTP event finshes and fires Parse event with HTML
    - Parser returns Vec of render objects
    - Parse event sends Vec over FFI
    - Shell core handler passes result along to caller
    - Caller renders render objects
*/

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub struct HtmlNode {
    pub tag: String,
    pub body: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub struct HtmlParseResult {
    pub nodes: Vec<HtmlNode>,
}

impl Operation for HtmlParseResult {
    type Output = Vec<HtmlNode>;
}

impl TokenSink for HtmlParseResult {
    type Handle = ();

    fn process_token(&mut self, token: Token, _line: u64) -> TokenSinkResult<Self::Handle> {
        match token {
            Token::TagToken(Tag {
                kind: TagKind::StartTag,
                name,
                self_closing: false,
                attrs: _,
            }) => {
                if [
                    TOKEN_H1, TOKEN_H2, TOKEN_H3, TOKEN_H4, TOKEN_H5, TOKEN_H6, TOKEN_P,
                ]
                .contains(&name)
                {
                    let name = match name {
                        TOKEN_H1 => "h1",
                        TOKEN_H2 => "h2",
                        TOKEN_H3 => "h3",
                        TOKEN_H4 => "h4",
                        TOKEN_H5 => "h5",
                        TOKEN_H6 => "h6",
                        // TOKEN_A => "a",
                        TOKEN_P => "p",
                        _ => return TokenSinkResult::Continue,
                    }
                    .to_string();

                    self.nodes.push(HtmlNode {
                        tag: name,
                        body: None,
                    });
                }
            }
            Token::CharacterTokens(string) => {
                if let Some(node) = self.nodes.last_mut() {
                    let string = string.to_string();
                    if node.body.is_some() {
                        if string.is_empty() {
                            let _ = self.nodes.pop();
                        }

                        return TokenSinkResult::Continue;
                    }

                    node.body = Some(string.to_string());
                }
            }
            _ => {}
        }

        TokenSinkResult::Continue
    }
}

#[derive(Capability)]
pub struct HtmlParser<Ev> {
    context: CapabilityContext<HtmlParseResult, Ev>,
}

impl<Ev> HtmlParser<Ev>
where
    Ev: 'static,
{
    pub fn new(context: CapabilityContext<HtmlParseResult, Ev>) -> Self {
        Self { context }
    }

    pub fn parse_html<F>(&self, html: &str, make_event: F)
    where
        F: FnOnce(HtmlParseResult) -> Ev + Send + 'static,
    {
        // Prepare input
        let mut input = BufferQueue::default();
        input.push_back(StrTendril::from_slice(html));

        // Parse
        let sink = HtmlParseResult { nodes: Vec::new() };
        let mut tokenizer = Tokenizer::new(sink, Default::default());
        let _ = tokenizer.feed(&mut input);
        tokenizer.end();

        // Send result to handler event
        let ctx = self.context.clone();
        self.context.spawn(async move {
            ctx.update_app(make_event(tokenizer.sink));
        });
    }
}

// pub fn parse_html(html: &str) -> Vec<HtmlNode> {
//     // Use html5ever types? Probably can't be serialized though
//     let sink = TocSink {
//         headings: Vec::new(),
//     };

//     // Prepare input
//     let mut input = BufferQueue::new();
//     input.push_back(StrTendril::from_slice(html));

//     // Parse
//     let mut tokenizer = Tokenizer::new(sink, Default::default());
//     let _ = tokenizer.feed(&mut input);
//     tokenizer.end();

//     tokenizer.sink.result
// }

#[cfg(test)]
mod parser_tests {}
