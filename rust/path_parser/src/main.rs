#![feature(slice_split_at_unchecked)]

use memchr;

#[derive(Debug, PartialEq, Eq)]
pub enum Segment<'a> {
    // static path like /a/b/c => c
    Static(&'a [u8]),
    // query param like /a/b/:id => :id
    Param(&'a [u8]),
    // catch all like /a/b/*c/d => *c/d
    CatchAll(&'a [u8]),
}

pub struct UrlPath<'a> {
    pub inner: &'a [u8],
}

pub struct PathIter<'a> {
    pub inner: &'a [u8],
}

impl<'a> UrlPath<'a> {
    pub fn new(inner: &'a [u8]) -> Self {
        Self { inner }
    }
}

impl<'a> PathIter<'a> {
    pub fn new(inner: &'a [u8]) -> Self {
        Self { inner }
    }
}

fn next_param(path: &[u8]) -> (&[u8], &[u8]) {
    if let Some(idx) = memchr::memchr(b'/', path) {
        return unsafe { path.split_at_unchecked(idx) };
    }
    return (path, &[]);
}

impl<'a> IntoIterator for UrlPath<'a> {
    type Item = Segment<'a>;
    type IntoIter = PathIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        PathIter::new(self.inner)
    }
}

impl<'a> Iterator for PathIter<'a> {
    type Item = Segment<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        macro_rules! check_empty {
            ($var: expr) => {
                if $var.is_empty() {
                    panic!("empty!");
                }
            };
        }

        macro_rules! check_special {
            ($var: expr) => {
                if memchr::memchr2(b'*', b'/', $var).is_some() {
                    panic!("unexpected url format.");
                }
            };
        }


        // eat "/"
        let slash = self.inner.first()?;
        assert_eq!(slash, &b'/');
        let path = unsafe { self.inner.split_at_unchecked(1).1 };

        match path.first()? {
            b':' => {
                let (param, rest) = next_param(path);
                check_empty!(param);
                check_special!(param);
                self.inner = rest;
                return Some(Segment::Param(param))
            }
            b'*' => {
                check_empty!(path);
                self.inner = &[];
                return Some(Segment::CatchAll(path))
            }
            _ => {
                let (path, rest) = next_param(path);
                check_empty!(path);
                self.inner = rest;
                return Some(Segment::Static(path))
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_static() {
        let url = UrlPath::new("/a/b/c/d".as_bytes());
        let mut url_iter = url.into_iter();
        assert_eq!(Some(Segment::Static("a".as_bytes())), url_iter.next());
        assert_eq!(Some(Segment::Static("b".as_bytes())), url_iter.next());
        assert_eq!(Some(Segment::Static("c".as_bytes())), url_iter.next());
        assert_eq!(Some(Segment::Static("d".as_bytes())), url_iter.next());
        assert_eq!(None, url_iter.next());
    }

    #[test]
    fn test_param() {
        let url = UrlPath::new("/a/b/:c/:d/e".as_bytes());
        let mut url_iter = url.into_iter();
        assert_eq!(Some(Segment::Static("a".as_bytes())), url_iter.next());
        assert_eq!(Some(Segment::Static("b".as_bytes())), url_iter.next());
        assert_eq!(Some(Segment::Param(":c".as_bytes())), url_iter.next());
        assert_eq!(Some(Segment::Param(":d".as_bytes())), url_iter.next());
        assert_eq!(Some(Segment::Static("e".as_bytes())), url_iter.next());
        assert_eq!(None, url_iter.next());
    }

    #[test]
    fn test_catch_all() {
        let url = UrlPath::new("/a/b/*c/d/e".as_bytes());
        let mut url_iter = url.into_iter();
        assert_eq!(Some(Segment::Static("a".as_bytes())), url_iter.next());
        assert_eq!(Some(Segment::Static("b".as_bytes())), url_iter.next());
        assert_eq!(Some(Segment::CatchAll("*c/d/e".as_bytes())), url_iter.next());
        assert_eq!(None, url_iter.next());
    }
}

fn main() {
    println!("Hello, world!");
}
