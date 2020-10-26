#[derive(Copy, Clone, Eq, PartialEq, Default, Debug)]
pub struct Block<'a> {
    pub name:   &'a str,
    call:       usize,
    attributes: Option<usize>,
    items:      Option<usize>,
    start:      Option<usize>,
    output:     Option<usize>,
    end:        Option<usize>,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum BlockError {
    MissingStart,
    MissingEnd,
}

impl<'a> Block<'a> {
    pub fn new(name: &'a str, call: usize) -> Self {
        Self {
            name,
            call,
            attributes: None,
            items: None,
            start: None,
            output: None,
            end: None,
        }
    }

    pub fn is_valid(&self) -> Result<(), BlockError> {
        if self.items.is_some() && self.start.is_none() {
            Err(BlockError::MissingStart)
        } else if self.output.is_some() && self.end.is_none() {
            Err(BlockError::MissingEnd)
        } else {
            Ok(())
        }
    }

    pub fn set_attribute(&mut self, i: usize) {
        if self.attributes.is_none() && self.items.is_none() && self.start.is_none() {
            self.attributes = Some(i);
        }
    }

    pub fn set_item(&mut self, i: usize) {
        if self.start.is_none() {
            self.items = self.items.or(Some(i));
        } else if self.end.is_none() {
            self.output = self.output.or(Some(i));
        }
    }

    pub fn set_start(&mut self, i: usize) -> Result<(), ()> {
        if self.start.is_some() || self.output.is_some() || self.end.is_some() {
            return Err(());
        }

        self.start = Some(i);
        Ok(())
    }

    pub fn set_end(&mut self, i: usize) -> Result<(), ()> {
        if self.end.is_some() {
            return Err(());
        }

        self.end = Some(i);
        Ok(())
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    pub fn block(
        name: &str,
        call: usize,
        attributes: Option<usize>,
        items: Option<usize>,
        start: Option<usize>,
        output: Option<usize>,
        end: Option<usize>,
    ) -> Block {
        Block {
            name,
            call,
            attributes,
            items,
            start,
            output,
            end,
        }
    }

    #[test]
    fn new() {
        assert_eq!(
            Block::new("macro", 2),
            block("macro", 2, None, None, None, None, None)
        );
    }

    #[test]
    fn set_attribute() {
        let mut b = block("", 1, None, Some(3), None, None, None);
        b.set_attribute(2);
        assert_eq!(
            b.attributes, None,
            "Attributes does not set when block has items"
        );

        let mut b = block("", 1, None, None, Some(3), None, None);
        b.set_attribute(2);
        assert_eq!(
            b.attributes, None,
            "Attributes does not set when block has start"
        );

        let mut b = block("", 1, Some(2), None, None, None, None);
        b.set_attribute(3);
        assert_eq!(
            b.attributes,
            Some(2),
            "Attributes does not change when already set"
        );

        let mut b = block("", 1, None, None, None, None, None);
        b.set_attribute(3);
        assert_eq!(b.attributes, Some(3), "Attributes sets");
    }

    #[test]
    fn set_item() {
        let mut b = block("", 1, None, None, None, None, None);
        b.set_item(2);
        assert_eq!(b.items, Some(2), "Items sets when start is None");
        assert_eq!(b.output, None, "Output does not change when start is None");

        let mut b = block("", 1, None, Some(2), Some(3), None, None);
        b.set_item(4);
        assert_eq!(b.items, Some(2), "Items does not change when start is Some");
        assert_eq!(b.output, Some(4), "Output sets when start is Some");

        let mut b = block("", 1, None, Some(2), Some(3), Some(4), Some(5));
        b.set_item(12);
        assert_eq!(
            b.items,
            Some(2),
            "Items does not change when start and end are Some"
        );
        assert_eq!(
            b.output,
            Some(4),
            "Output does not change when start and end are Some"
        );
    }

    #[test]
    fn set_start() {
        let mut b = block("", 1, None, None, Some(2), None, None);
        assert_eq!(b.set_start(3), Err(()), "Errors when start already set");

        let mut b = block("", 1, None, None, Some(2), Some(3), None);
        assert_eq!(b.set_start(4), Err(()), "Errors when output already set");

        let mut b = block("", 1, None, None, Some(2), None, Some(3));
        assert_eq!(b.set_start(4), Err(()), "Errors when end already set");

        let mut b = block("", 1, None, None, None, None, None);
        assert_eq!(b.set_start(4), Ok(()), "Does not error");
        assert_eq!(b.start, Some(4), "Start sets");
    }

    #[test]
    fn set_end() {
        let mut b = block("", 1, None, None, Some(2), None, Some(3));
        assert_eq!(b.set_end(4), Err(()), "Errors when end already set");

        let mut b = block("", 1, None, None, Some(2), None, None);
        assert_eq!(b.set_end(3), Ok(()), "Does not error");
        assert_eq!(b.end, Some(3), "End sets");
    }
}
