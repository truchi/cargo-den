#[derive(Copy, Clone, Eq, PartialEq, Default, Debug)]
struct Block<'a> {
    name:       &'a str,
    call:       usize,
    attributes: Option<usize>,
    items:      Option<usize>,
    start:      usize,
    end:        Option<usize>,
}

impl<'a> Block<'a> {
    fn new(name: &'a str, call: usize) -> Self {
        Self {
            name,
            call,
            attributes: None,
            items: None,
            start: call + 1,
            end: None,
        }
    }

    fn attribute(&mut self, attribute: usize) {
        self.attributes = self.attributes.or(Some(attribute));
        self.start = attribute + 1;
    }

    fn item(&mut self, item: usize) {
        self.items = self.items.or(Some(item));
        self.start = item + 1;
    }

    fn start(&mut self, start: usize) {
        self.start = start;
    }

    fn end(&mut self, end: usize) {
        self.end = Some(end);
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum State {
    Call,
    Attribute,
    Item,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum Warning {
    SoloStart(usize),
    SoloEnd(usize),
    CallNotBegin(usize),
    NoBang(usize),
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum Error {
    MissingOpen(usize),
    MissingClose(usize),
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Parser<'a> {
    content:  &'a str,
    state:    State,
    blocks:   Vec<Block<'a>>,
    warnings: Vec<Warning>,
    errors:   Vec<Error>,
}

impl<'a> Parser<'a> {
    const ATTRIBUTE: &'static str = "//";
    const CALL: &'static str = "// @den::";
    const END: &'static str = "// ```@den```";
    const START: &'static str = "// ```@den``` end: ";

    pub fn new(content: &'a str) -> Self {
        Self {
            content,
            state: State::Call,
            blocks: vec![],
            warnings: vec![],
            errors: vec![],
        }
    }

    // pub fn parse2(&mut self, content: &'a str) {
    // for line in content.lines().enumerate() {
    // match self.state {
    // State::Call => {}
    // State::Attribute => {}
    // State::Item => {}
    // }
    // }
    // }

    pub fn parse(&mut self) {
        for line in self.content.lines().enumerate() {
            self.parse_line(line);
        }
    }

    fn parse_line(&mut self, (i, line): (usize, &'a str)) {
        match self.state {
            State::Call => {
                if let Some(_) = line.find(Self::START) {
                    self.warnings.push(Warning::SoloStart(i));
                    return;
                }

                if let Some(_) = line.find(Self::END) {
                    // self.blocks.last_mut().expect("No blocks").end(i);
                    if let Some(block) = self.blocks.last_mut() {
                        // TODO matching names
                        block.end(i);
                    } else {
                        self.warnings.push(Warning::SoloEnd(i));
                    }
                    return;
                }

                let call = line.find(Self::CALL);
                if call == None {
                    return;
                }

                let call = call.unwrap();
                for c in line[0..call].chars() {
                    if !c.is_whitespace() {
                        self.warnings.push(Warning::CallNotBegin(i));
                        return;
                    }
                }

                let name_i = call + Self::CALL.len();
                if let Some(bang) = line[name_i..].find('!') {
                    let name = &line[name_i..name_i + bang];

                    self.blocks.push(Block::new(name, i));
                    self.state = State::Attribute;
                } else {
                    self.warnings.push(Warning::NoBang(i));
                }
            }
            State::Attribute =>
                if let Some(_) = line.find(Self::ATTRIBUTE) {
                    self.blocks.last_mut().expect("No blocks").attribute(i);
                } else {
                    self.state = State::Item;
                },
            State::Item =>
                if let Some(_) = line.find(Self::START) {
                    self.blocks.last_mut().expect("No blocks").start(i);
                    self.state = State::Call;
                } else {
                    self.blocks.last_mut().expect("No blocks").item(i);
                },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    macro_rules! parser {
        (
            $content:ident,
            $state:ident,[
                $(
                    $name:literal,
                    $call:literal,
                    $attributes:expr,
                    $items:expr,
                    $start:literal,
                    $end:expr
                )*
            ]
        ) => {
            Parser {
                content:  &$content,
                state:    State::$state,
                blocks:   vec![$(
                    Block {
                        name: $name,
                        call: $call,
                        attributes: $attributes,
                        items: $items,
                        start: $start,
                        end: $end,
                    }
                )*],
                warnings: vec![],
                errors:   vec![],
            }
        };
    }

    #[test]
    fn test() {
        let content = r#"
        // @den::macro!
        // ```@den```
        "#;
        let mut parser = Parser::new(content);
        parser.parse();

        assert_eq!(
            parser,
            parser!(content, Item, ["macro", 1, Some(2), None, 3, None])
        );

        // assert_eq!(parser, Parser {
        // content:  &content,
        // state:    State::Call,
        // blocks:   vec![],
        // warnings: vec![],
        // errors:   vec![],
        // });
    }
}
