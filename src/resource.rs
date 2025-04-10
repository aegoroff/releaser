use color_eyre::eyre::Result;
use core::fmt;
use url::Url;

const SEP: char = '/';

#[derive(Clone)]
pub struct Resource {
    url: Url,
}

impl Resource {
    pub fn new(input: &str) -> Result<Resource> {
        let url = Url::parse(input)?;
        Ok(Resource { url })
    }

    pub fn append_path(&mut self, path: &str) -> &mut Self {
        if let Some(segments) = self.url.path_segments() {
            let p = segments
                .chain(path.split(SEP))
                .filter(|x| !x.is_empty())
                .fold(String::new(), |mut s, x| {
                    s.push_str(x);
                    s.push(SEP);
                    s
                });

            // If original ends with spearator char use as is or trim last (separator) char otherwise
            let path_to_set = if path.chars().next_back().unwrap_or_default() == SEP {
                &p
            } else {
                &p[..p.len() - 1]
            };
            self.url.set_path(path_to_set);
        } else {
            let r = self.url.join(path);
            if let Ok(u) = r {
                self.url = u;
            }
        }
        self
    }
}

impl fmt::Display for Resource {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.url)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_in_result)]
    #![allow(clippy::unwrap_used)]
    use super::*;
    use rstest::rstest;

    #[test]
    fn new_correct_some() {
        // Arrange

        // Act
        let r = Resource::new("http://localhost");

        // Assert
        assert!(r.is_ok());
    }

    #[test]
    fn new_incorrect_none() {
        // Arrange

        // Act
        let r = Resource::new("http/localhost");

        // Assert
        assert!(r.is_err());
    }

    #[rstest]
    #[case("http://localhost", "x", "http://localhost/x")]
    #[case("http://localhost", "/x", "http://localhost/x")]
    #[case("http://localhost", "/x/", "http://localhost/x/")]
    #[case("http://localhost", "x/", "http://localhost/x/")]
    #[case("http://localhost", "/x/y/", "http://localhost/x/y/")]
    #[case("http://localhost/", "x", "http://localhost/x")]
    #[case("http://localhost/", "/x", "http://localhost/x")]
    #[case("http://localhost/", "/x/", "http://localhost/x/")]
    #[case("http://localhost/", "x/", "http://localhost/x/")]
    #[case("http://localhost/", "x/y", "http://localhost/x/y")]
    #[case("http://localhost/", "/x/y", "http://localhost/x/y")]
    #[case("http://localhost/", "/x/y/", "http://localhost/x/y/")]
    #[case("http://localhost/x", "/y", "http://localhost/x/y")]
    #[case("http://localhost/x", "y", "http://localhost/x/y")]
    #[case("http://localhost/x", "y/", "http://localhost/x/y/")]
    #[case("http://localhost/x", "/y/", "http://localhost/x/y/")]
    #[case("http://localhost/x/", "y", "http://localhost/x/y")]
    #[case("http://localhost/x/", "/y", "http://localhost/x/y")]
    #[case("http://localhost/x/", "y/", "http://localhost/x/y/")]
    #[case("http://localhost/x/", "/y/", "http://localhost/x/y/")]
    #[case::real_slashed_base(
        "https://github.com/aegoroff/dirstat/releases/download/v1.0.7/",
        "dirstat_1.0.7_darwin_amd64.tar.gz",
        "https://github.com/aegoroff/dirstat/releases/download/v1.0.7/dirstat_1.0.7_darwin_amd64.tar.gz"
    )]
    #[case::real_slashless_base(
        "https://github.com/aegoroff/dirstat/releases/download/v1.0.7",
        "dirstat_1.0.7_darwin_amd64.tar.gz",
        "https://github.com/aegoroff/dirstat/releases/download/v1.0.7/dirstat_1.0.7_darwin_amd64.tar.gz"
    )]
    #[case("http://localhost", "http://:/", "http://localhost/http:/:/")]
    #[trace]
    fn append_path_tests(#[case] base: &str, #[case] path: &str, #[case] expected: &str) {
        // Arrange
        let mut r = Resource::new(base).unwrap();

        // Act
        r.append_path(path);

        // Assert
        assert_eq!(r.to_string().as_str(), expected);
    }

    #[test]
    fn append_path_twice() {
        // Arrange
        let mut r = Resource::new("http://localhost").unwrap();

        // Act
        r.append_path("x").append_path("y");

        // Assert
        assert_eq!(r.to_string().as_str(), "http://localhost/x/y");
    }
}
