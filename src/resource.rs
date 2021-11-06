extern crate url;

use self::url::Url;
use core::fmt;
use itertools::Itertools;

#[derive(Clone)]
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
    use rstest::*;
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

    #[rstest]
    #[case("http://localhost", "/x", "http://localhost/x")]
    #[case("http://localhost", "/x/", "http://localhost/x/")]
    #[case("http://localhost/", "/x/", "http://localhost/x/")]
    #[case("http://localhost/", "x/y", "http://localhost/x/y")]
    #[case("http://localhost/", "/x/y", "http://localhost/x/y")]
    #[case("http://localhost/x", "/y", "http://localhost/x/y")]
    #[case("http://localhost/x/", "/y", "http://localhost/x/y")]
    #[case("http://localhost/x/", "y", "http://localhost/x/y")]
    #[case::real_slashed_base("https://github.com/aegoroff/dirstat/releases/download/v1.0.7/", "dirstat_1.0.7_darwin_amd64.tar.gz", "https://github.com/aegoroff/dirstat/releases/download/v1.0.7/dirstat_1.0.7_darwin_amd64.tar.gz")]
    #[case::real_slashless_base("https://github.com/aegoroff/dirstat/releases/download/v1.0.7", "dirstat_1.0.7_darwin_amd64.tar.gz", "https://github.com/aegoroff/dirstat/releases/download/v1.0.7/dirstat_1.0.7_darwin_amd64.tar.gz")]
    fn append_path_tests(#[case] base: &str, #[case] path: &str, #[case] expected: &str) {
        // Arrange
        let mut r = Resource::new(base).unwrap();

        // Act
        r.append_path(path);

        // Assert
        assert_that!(r.to_string().as_str()).is_equal_to(expected);
    }
}
