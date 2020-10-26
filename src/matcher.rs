#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum Find<'a> {
    Call(&'a str),
    CallNoBang,
    Attribute,
    Item,
    Start,
    End(&'a str),
    EndNoBang,
    Empty,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Default, Debug)]
pub struct Matcher;

impl Matcher {
    pub fn find(s: &str) -> Find {
        let s = s.trim_start();

        if s.starts_with("//") {
            let s = &s[2..].trim_start();

            if s.starts_with("@den::") {
                return match s.find("!") {
                    Some(bang) => Find::Call(&s[6..bang].trim()),
                    None => Find::CallNoBang,
                };
            } else if s.starts_with("```@den```") {
                let s = &s[10..].trim_start();

                if s.starts_with("end:") {
                    return match s.find("!") {
                        Some(bang) => Find::End(&s[4..bang].trim()),
                        None => Find::EndNoBang,
                    };
                } else {
                    return Find::Start;
                }
            } else {
                return Find::Attribute;
            }
        } else {
            let s = s.trim();

            if s.len() > 0 {
                return Find::Item;
            } else {
                return Find::Empty;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn call() {
        assert_eq!(Matcher::find("//@den::mymacro!"), Find::Call("mymacro"));
        assert_eq!(
            Matcher::find("  //    @den:: my macro !  // lol"),
            Find::Call("my macro")
        );
    }

    #[test]
    fn call_no_bang() {
        assert_eq!(Matcher::find("//@den::mymacro"), Find::CallNoBang);
        assert_eq!(
            Matcher::find("  //    @den:: my macro   // lol"),
            Find::CallNoBang
        );
    }

    #[test]
    fn attribute() {
        assert_eq!(Matcher::find("//"), Find::Attribute);
        assert_eq!(Matcher::find("  //    bla bla"), Find::Attribute);
    }

    #[test]
    fn item() {
        assert_eq!(Matcher::find("  fn item() {}"), Find::Item);
    }

    #[test]
    fn start() {
        assert_eq!(Matcher::find("//```@den```"), Find::Start);
        assert_eq!(Matcher::find("  //  ```@den```  // lqlqlq"), Find::Start);
    }

    #[test]
    fn end() {
        assert_eq!(
            Matcher::find("//```@den```end:mymacro!"),
            Find::End("mymacro")
        );
        assert_eq!(
            Matcher::find("   //  ```@den```  end:   my macro  ! // kh"),
            Find::End("my macro")
        );
    }

    #[test]
    fn end_no_bang() {
        assert_eq!(Matcher::find("//```@den```end:mymacro"), Find::EndNoBang);
        assert_eq!(
            Matcher::find("   //  ```@den```  end:   my macro   // kh"),
            Find::EndNoBang
        );
    }

    #[test]
    fn empty() {
        assert_eq!(Matcher::find(""), Find::Empty);
        assert_eq!(Matcher::find("     "), Find::Empty);
    }
}
