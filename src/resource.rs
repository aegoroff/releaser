extern crate url;

use self::url::Url;
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

    pub fn append_path(&mut self, path: &str) -> String {
        match self.url.path_segments() {
            None => self.url.join(path).unwrap().to_string(),
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
                self.url.to_string()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_correct_some() {
        // Arrange

        // Act
        let r = Resource::new("http://localhost");

        // Assert
        assert!(r.is_some());
    }

    #[test]
    fn new_incorrect_none() {
        // Arrange

        // Act
        let r = Resource::new("http/localhost");

        // Assert
        assert!(r.is_none());
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
            let actual = r.append_path(path);

            validator
                .given(&format!("base: {} path: {}", base, path))
                .when("append_path")
                .then(&format!("it should be {:#?}", expected))
                .assert_eq(expected, &actual);
        }
    }
}
