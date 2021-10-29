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
                self.url.set_path(&p);
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
    fn create_path_correct_as_expected() {
        // Arrange
        let mut r = Resource::new("http://localhost").unwrap();

        // Act
        let r = r.append_path("/x");

        // Assert
        assert_eq!("http://localhost/x", r);
    }

    #[test]
    fn create_path_base_no_trailed_slash_as_expected() {
        // Arrange
        let mut r = Resource::new("http://localhost/x").unwrap();

        // Act
        let r = r.append_path("/y");

        // Assert
        assert_eq!("http://localhost/x/y", r);
    }

    #[test]
    fn create_path_base_trailed_slash_as_expected() {
        // Arrange
        let mut r = Resource::new("http://localhost/x/").unwrap();

        // Act
        let r = r.append_path("/y");

        // Assert
        assert_eq!("http://localhost/x/y", r);
    }

    #[test]
    fn create_path_base_trailed_slash_append_plain_as_expected() {
        // Arrange
        let mut r = Resource::new("http://localhost/x/").unwrap();

        // Act
        let r = r.append_path("y");

        // Assert
        assert_eq!("http://localhost/x/y", r);
    }
}
