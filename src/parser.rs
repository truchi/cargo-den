use super::{
    block::{Block, BlockError},
    matcher::{Find, Matcher},
};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Warning {
    UnexpectedStart(usize),
    UnexpectedEnd(usize),
    CallNoBang(usize),
    EndNoBang(usize),
    NameMismatch(usize),
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Parser<'a> {
    content:  &'a str,
    blocks:   Vec<Block<'a>>,
    warnings: Vec<Warning>,
}

impl<'a> Parser<'a> {
    pub fn new(content: &'a str) -> Self {
        Self {
            content,
            blocks: vec![],
            warnings: vec![],
        }
    }

    pub fn parse(mut self) -> Result<Self, (BlockError, Block<'a>, Option<usize>)> {
        for (i, line) in self.content.lines().enumerate() {
            match Matcher::find(line) {
                Find::Empty => {}
                Find::CallNoBang => self.warnings.push(Warning::CallNoBang(i)),
                Find::EndNoBang => self.warnings.push(Warning::EndNoBang(i)),
                Find::Call(name) => {
                    // Error if previous block is not in a valid state
                    if let Some(block) = self.blocks.last_mut() {
                        if let Err(error) = block.is_valid() {
                            return Err((error, self.blocks.pop().unwrap(), Some(i)));
                        }
                    }

                    // Append new block
                    self.blocks.push(Block::new(name, i));
                }
                Find::Attribute =>
                // Set attribute line number on last block
                    if let Some(block) = self.blocks.last_mut() {
                        block.set_attribute(i);
                    },
                Find::Item =>
                // Set item line number on last block
                    if let Some(block) = self.blocks.last_mut() {
                        block.set_item(i);
                    },
                Find::Start =>
                // Set start line number on last block,
                // or warn for execpected start tag
                    if let Some(block) = self.blocks.last_mut() {
                        if let Err(_) = block.set_start(i) {
                            self.warnings.push(Warning::UnexpectedStart(i));
                        }
                    } else {
                        self.warnings.push(Warning::UnexpectedStart(i));
                    },
                Find::End(name) =>
                // Set end line number on last block,
                // or warn for execpected end tag
                    if let Some(block) = self.blocks.last_mut() {
                        if let Err(_) = block.set_end(i) {
                            self.warnings.push(Warning::UnexpectedEnd(i));
                        } else if block.name != name {
                            self.warnings.push(Warning::NameMismatch(i));
                        }
                    } else {
                        self.warnings.push(Warning::UnexpectedEnd(i));
                    },
            }
        }

        // Error if last block is not in a valid state
        if let Some(block) = self.blocks.last_mut() {
            if let Err(error) = block.is_valid() {
                return Err((error, self.blocks.pop().unwrap(), None));
            }
        }

        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use super::{super::block::tests::block, *};
    use pretty_assertions::assert_eq;

    macro_rules! assert_parser {
        (
            $fmt:literal, $content:literal,
            [$((
                $name:literal, $call:literal, $attributes:expr,
                $items:expr, $start:expr, $output:expr, $end:expr
            ))*]
            $([$($warning:ident($i:literal))+])?
        ) => {
            let content = $content;
            let parser = Parser::new(content).parse();
            let expected = Ok(Parser {
                content:  content,
                blocks:   vec![$(
                    block($name, $call, $attributes, $items, $start, $output, $end),
                )*],
                warnings: vec![$($(Warning::$warning($i),)+)?],
            });

            assert_eq!(parser, expected, $fmt);
        };
        (
            $fmt:expr, $content:literal, $error:ident (
                $name:literal, $call:literal, $attributes:expr,
                $items:expr, $start:expr, $output:expr, $end:expr
            )
            $i:expr
        ) => {
            let content = $content;
            let parser = Parser::new(content).parse();
            let expected = Err((
                BlockError::$error,
                block($name, $call, $attributes, $items, $start, $output, $end),
                $i
            ));

            assert_eq!(parser, expected, $fmt);
        };
    }

    #[test]
    fn call() {
        assert_parser!(
            "Bare macro call",
            r#"
                // @den::macro!
            "#,
            [("macro", 1, None, None, None, None, None)]
        );

        assert_parser!(
            "Macro call with start tag",
            r#"
                // @den::macro!
                // ```@den```
            "#,
            [("macro", 1, None, None, Some(2), None, None)]
        );

        assert_parser!(
            "Macro call with output",
            r#"
                // @den::macro!
                // ```@den```
                struct Output;
                // ```@den``` end:macro!
            "#,
            [("macro", 1, None, None, Some(2), Some(3), Some(4))]
        );
    }

    #[test]
    fn attributes() {
        assert_parser!(
            "Macro with attributes",
            r#"
                // @den::macro!
                // attrs
                // attrs
                // ```@den```
            "#,
            [("macro", 1, Some(2), None, Some(4), None, None)]
        );

        assert_parser!(
            "Macro with attributes and output",
            r#"
                // @den::macro!
                // attrs
                // attrs
                // ```@den```
                struct Output;
                // ```@den``` end:macro!
            "#,
            [("macro", 1, Some(2), None, Some(4), Some(5), Some(6))]
        );
    }

    #[test]
    fn items() {
        assert_parser!(
            "Macro with items",
            r#"
                // @den::macro!
                struct Input1;
                struct Input2;
                // ```@den```
            "#,
            [("macro", 1, None, Some(2), Some(4), None, None)]
        );

        assert_parser!(
            "Macro with items and output",
            r#"
                // @den::macro!
                struct Input1;
                struct Input2;
                // ```@den```
                struct Output1;
                struct Output2;
                // ```@den``` end:macro!
            "#,
            [("macro", 1, None, Some(2), Some(4), Some(5), Some(7))]
        );
    }

    #[test]
    fn attributes_and_items() {
        assert_parser!(
            "Macro with attributes and items",
            r#"
                // @den::macro!
                // attrs
                // attrs
                fn input1() {}
                fn input2() {}
                // ```@den```
            "#,
            [("macro", 1, Some(2), Some(4), Some(6), None, None)]
        );

        assert_parser!(
            "Macro with attributes, items and output",
            r#"
                // @den::macro!
                // attrs
                // attrs
                fn input1() {}
                fn input2() {}
                // ```@den```
                fn output1() {}
                fn output2() {}
                // ```@den``` end:macro!
            "#,
            [("macro", 1, Some(2), Some(4), Some(6), Some(7), Some(9))]
        );
    }

    #[test]
    fn missing_start() {
        assert_parser!(
            "Missing start with 1 call",
            r#"
                // @den::macro!
                fn output1() {}
            "#,
            MissingStart("macro", 1, None, Some(2), None, None, None) None
        );
        assert_parser!(
            "Missing start with 2 calls",
            r#"
                // @den::macro1!
                fn output1() {}

                // @den::macro2!
            "#,
            MissingStart("macro1", 1, None, Some(2), None, None, None) Some(4)
        );
    }

    #[test]
    fn missing_end() {
        assert_parser!(
            "Missing end with 1 call",
            r#"
                // @den::macro!
                // ```@den```
                fn output1() {}
            "#,
            MissingEnd("macro", 1, None, None, Some(2), Some(3), None) None
        );
        assert_parser!(
            "Missing start with 2 calls",
            r#"
                // @den::macro1!
                // ```@den```
                fn output1() {}

                // @den::macro2!
            "#,
            MissingEnd("macro1", 1, None, None, Some(2), Some(3), None) Some(5)
        );
    }
}
