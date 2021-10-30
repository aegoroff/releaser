extern crate url;

use self::url::Url;
use core::fmt;
use itertools::Itertools;

pub struct Resource {
    url: Url,
}

impl Resource {
    pub fn new(uri: &str) -> Option<Resource> {
        if let Ok(url) = Url::parse(uri) {
            Some(Resource { url })
        } else {
            None
        }
    }

    pub fn append_path(&mut self, path: &str) {
        match self.url.path_segments() {
            None => {
                let r = self.url.join(path);
                if r.is_ok() {}
            }
            Some(segments) => {
                let p = segments
                    .chain(std::iter::once(path))
                    .filter(|x| !x.is_empty())
                    .map(|x| x.trim_matches('/'))
                    .join("/");

                if path.len() > 1 && path.chars().last().unwrap_or_default() == '/' {
                    let p = p + "/";
                    self.url.set_path(&p);
                } else {
                    self.url.set_path(&p);
                }
            }
        }
    }
}

impl std::fmt::Display for Resource {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use spectral::prelude::*;

    #[test]
    fn new_correct_some() {
        // Arrange

        // Act
        let r = Resource::new("http://localhost");

        // Assert
        assert_that!(r.is_some()).is_true();
    }

    #[test]
    fn new_incorrect_none() {
        // Arrange

        // Act
        let r = Resource::new("http/localhost");

        // Assert
        assert_that!(r.is_none()).is_true();
    }

    #[test]
    fn append_path_tests() {
        // Arrange
        let cases = vec![
            (("http://localhost", "/x"), "http://localhost/x"),
            (("http://localhost", "/x/"), "http://localhost/x/"),
            (("http://localhost/", "/x/"), "http://localhost/x/"),
            (("http://localhost/", "x/y"), "http://localhost/x/y"),
            (("http://localhost/", "/x/y"), "http://localhost/x/y"),
            (("http://localhost/x", "/y"), "http://localhost/x/y"),
            (("http://localhost/x/", "/y"), "http://localhost/x/y"),
            (("http://localhost/x/", "y"), "http://localhost/x/y"),
            (("https://github.com/aegoroff/dirstat/releases/download/v1.0.7/", "dirstat_1.0.7_darwin_amd64.tar.gz"), "https://github.com/aegoroff/dirstat/releases/download/v1.0.7/dirstat_1.0.7_darwin_amd64.tar.gz"),
            (("https://github.com/aegoroff/dirstat/releases/download/v1.0.7", "dirstat_1.0.7_darwin_amd64.tar.gz"), "https://github.com/aegoroff/dirstat/releases/download/v1.0.7/dirstat_1.0.7_darwin_amd64.tar.gz"),
        ];

        // Act
        for (validator, input, expected) in table_test!(cases) {
            let (base, path) = input;
            let mut r = Resource::new(base).unwrap();
            r.append_path(path);
            let actual = r.to_string();

            validator
                .given(&format!("base: {} path: {}", base, path))
                .when("append_path")
                .then(&format!("it should be {:#?}", expected))
                .assert_eq(expected, &actual);
        }
    }
}
