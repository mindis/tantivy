use super::{Token, Analyzer, TokenStream};
use std::str;
use schema::FACET_SEP_BYTE;


/// The `FacetTokenizer` process a `Facet` binary representation
/// and emits a token for all of its parent.
///
/// For instance,  `/america/north_america/canada`
/// will emit the three following tokens
///     - `/america/north_america/canada`
///     - `/america/north_america`
///     - `/america`
#[derive(Clone)]
pub struct FacetTokenizer;

pub struct FacetTokenStream<'a> {
    text: &'a str,
    pos: usize,
    token: Token,
}

impl<'a> Analyzer<'a> for FacetTokenizer {
    type TokenStreamImpl = FacetTokenStream<'a>;

    fn token_stream(&mut self, text: &'a str) -> Self::TokenStreamImpl {
        FacetTokenStream {
            text: text,
            pos: 0,
            token: Token::default(),
        }
    }
}


impl<'a> TokenStream for FacetTokenStream<'a> {
    fn advance(&mut self) -> bool {
        let bytes: &[u8] = self.text.as_bytes();
        if self.pos == bytes.len() {
            false
        } else {
            let next_sep_pos = bytes[self.pos + 1..]
                .iter()
                .cloned()
                .position(|b| b == FACET_SEP_BYTE)
                .map(|pos| pos + self.pos + 1)
                .unwrap_or(bytes.len());
            let facet_prefix = unsafe { str::from_utf8_unchecked(&bytes[self.pos..next_sep_pos]) };
            self.pos = next_sep_pos;
            self.token.term.push_str(facet_prefix);
            true
        }
    }

    fn token(&self) -> &Token {
        &self.token
    }

    fn token_mut(&mut self) -> &mut Token {
        &mut self.token
    }
}

#[cfg(test)]
mod tests {

    use analyzer::{TokenStream, Token, Analyzer};
    use super::FacetTokenizer;
    use schema::Facet;

    #[test]
    fn test_facet_tokenizer() {
        let facet = Facet::from_path(vec!["top", "a", "b"]);
        let mut tokens = vec![];
        {
            let mut add_token = |token: &Token| {
                let facet = Facet::from_encoded(token.term.as_bytes().to_owned());
                tokens.push(format!("{}", facet));
            };
            FacetTokenizer
                .token_stream(unsafe { ::std::str::from_utf8_unchecked(facet.encoded_bytes()) })
                .process(&mut add_token);
        }
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0], "/top");
        assert_eq!(tokens[1], "/top/a");
        assert_eq!(tokens[2], "/top/a/b");
    }
}