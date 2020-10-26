mod block;
mod matcher;
mod parser;

use ignore::{DirEntry, WalkBuilder, WalkState};
use libloading::{Library, Symbol};
use parser::Parser;
use proc_macro2::TokenStream;
use std::{fs::read, path::PathBuf};

fn main() {
    let lib = Library::new("den/target/debug/libden.so").unwrap();

    let den: Symbol<fn(m: &str, tokens: TokenStream) -> TokenStream>;
    unsafe {
        den = lib.get(b"den").unwrap();
    }

    let ret = den("lol", TokenStream::new());
    println!("Returned:");
    println!("{}", ret);

    fn filter(entry: Result<DirEntry, ignore::Error>) -> Result<PathBuf, WalkState> {
        let entry = entry.or(Err(WalkState::Continue))?;

        if let Some(_error) = entry.error() {
            // TODO handle error
            return Err(WalkState::Continue);
        }

        let path = entry.path();
        if path.starts_with("./den/") {
            return Err(WalkState::Skip);
        }

        if let Some(ext) = path.extension() {
            if ext != "rs" {
                return Err(WalkState::Continue);
            }
        } else {
            return Err(WalkState::Continue);
        }

        Ok(entry.into_path())
    }

    // for entry in WalkBuilder::new("./").build() {
    // let entry = filter(entry);
    // if let Err(_) = entry {
    // continue;
    // }
    //
    // println!("{:?}", entry);
    // }

    WalkBuilder::new("./").build_parallel().run(|| {
        Box::new(|entry| {
            let entry = filter(entry);
            if let Err(state) = entry {
                return state;
            }
            match entry {
                Ok(path) => go(path),
                Err(state) => return state,
            }

            WalkState::Continue
        })
    });

    fn go(path: PathBuf) {
        println!("reading {:?}", path);
        let content = String::from_utf8(read(path).unwrap()).unwrap();

        let parser = Parser::new(&content).parse();
        println!("{:#?}", parser);
    }
}
