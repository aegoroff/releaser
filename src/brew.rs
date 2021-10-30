use std::option::Option::Some;

use handlebars::Handlebars;
use serde::Serialize;

use crate::pkg;
use crate::pkg::Package;
use crate::workflow::Crate;
use crate::CrateConfig;

#[derive(Serialize, Default)]
pub struct Brew {
    pub formula: String,
    pub name: String,
    pub description: String,
    pub homepage: Option<String>,
    pub version: String,
    pub license: String,
    pub linux: Option<Package>,
    pub macos: Option<Package>,
}

const TEMPLATE: &str = r###"# typed: false
# frozen_string_literal: true
# This file was generated by releaser. DO NOT EDIT.
class {{ formula }} < Formula
  desc "{{ description }}"
  homepage "{{ homepage }}"
  version "{{ version }}"
  {{#if license }}
  license "{{ license }}"
  {{/if}}
  {{#if macos }}
{{lines 1}}
  on_macos do
    if Hardware::CPU.intel?
    {{#with macos}}
      url "{{ url }}"
      sha256 "{{ hash }}"
     {{/with}}
    end
  end
  {{/if}}
  {{#if linux }}
{{lines 1}}
  on_linux do
    if Hardware::CPU.intel?
    {{#with linux}}
      url "{{ url }}"
      sha256 "{{ hash }}"
    {{/with}}
    end
  end
  {{/if}}
{{#if (or linux macos)}}
{{lines 1}}
  def install
    bin.install "{{ name }}"
  end
{{lines 1}}
{{/if}}
end
"###;

pub fn new_brew(
    crate_path: &str,
    linux_path: &str,
    macos_path: &str,
    base_uri: &str,
) -> Option<String> {
    let (fs, file_in_fs) = Crate::new_crate_config_source(crate_path);
    let config = CrateConfig::open(&fs, file_in_fs.to_str().unwrap_or_default());

    if let Ok(c) = config {
        let name = c.package.name;
        let brew = Brew {
            formula: uppercase_first_letter(&name),
            name,
            description: c.package.description.unwrap_or_default(),
            homepage: c.package.homepage,
            version: c.package.version,
            license: c.package.license.unwrap_or_default(),
            linux: pkg::new_binary_pkg(linux_path, base_uri),
            macos: pkg::new_binary_pkg(macos_path, base_uri),
        };
        Some(serialize_brew(&brew))
    } else {
        None
    }
}

fn uppercase_first_letter(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().chain(c).collect(),
    }
}

fn serialize_brew<T: Serialize>(data: &T) -> String {
    handlebars_helper!(lines: |count: i32| {
        let mut i = 0;
        while i < count {
            println!("");
            i += 1;
        }
    });
    let mut reg = Handlebars::new();
    reg.register_helper("lines", Box::new(lines));
    reg.render_template(TEMPLATE, data).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uppercase_first_letter_test() {
        // Arrange

        // Act
        let r = uppercase_first_letter("test");

        // Assert
        assert_eq!(r, "Test");
    }

    #[test]
    fn uppercase_first_letter_test_already_uppercased() {
        // Arrange

        // Act
        let r = uppercase_first_letter("Test");

        // Assert
        assert_eq!(r, "Test");
    }

    #[test]
    fn uppercase_first_letter_test_empty_string() {
        // Arrange

        // Act
        let r = uppercase_first_letter("");

        // Assert
        assert_eq!(r, "");
    }

    #[test]
    fn serialize_brew_no_packages_test() {
        // Arrange
        let brew = Brew {
            formula: "Solv".to_string(),
            name: "solv".to_string(),
            description: "desc".to_string(),
            homepage: None,
            version: "v0.4.0".to_string(),
            license: "MIT".to_string(),
            linux: None,
            macos: None,
        };

        // Act
        let result = serialize_brew(&brew);

        // Assert
        assert_eq!(
            r###"# typed: false
# frozen_string_literal: true
# This file was generated by releaser. DO NOT EDIT.
class Solv < Formula
  desc "desc"
  homepage ""
  version "v0.4.0"
  license "MIT"
end
"###,
            result
        )
    }

    #[test]
    fn serialize_brew_macos_test() {
        // Arrange
        let macos = Package {
            url: "https://github.com/aegoroff/solt/releases/download/v1.0.7/solt_1.0.7_darwin_amd64.tar.gz".to_string(),
            hash: "9a6c8144ed77cd5e2b88031109ac4285ca08e8c644f3d022a389359470721a7b".to_string(),
        };
        let brew = Brew {
            formula: "Solv".to_string(),
            name: "solv".to_string(),
            description: "desc".to_string(),
            homepage: None,
            version: "v0.4.0".to_string(),
            license: "MIT".to_string(),
            linux: None,
            macos: Some(macos),
        };

        // Act
        let result = serialize_brew(&brew);

        // Assert
        assert_eq!(
            r###"# typed: false
# frozen_string_literal: true
# This file was generated by releaser. DO NOT EDIT.
class Solv < Formula
  desc "desc"
  homepage ""
  version "v0.4.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.intel?
      url "https://github.com/aegoroff/solt/releases/download/v1.0.7/solt_1.0.7_darwin_amd64.tar.gz"
      sha256 "9a6c8144ed77cd5e2b88031109ac4285ca08e8c644f3d022a389359470721a7b"
    end
  end

  def install
    bin.install "solv"
  end

end
"###,
            result
        )
    }

    #[test]
    fn serialize_brew_linux_test() {
        // Arrange
        let linux = Package {
            url: "https://github.com/aegoroff/solt/releases/download/v1.0.7/solt_1.0.7_linux_amd64.tar.gz".to_string(),
            hash: "fb5c2f5d41c7d3485898de9905736dc8c540a912dc95d3a55bd9360901689811".to_string(),
        };
        let brew = Brew {
            formula: "Solv".to_string(),
            name: "solv".to_string(),
            description: "desc".to_string(),
            homepage: None,
            version: "v0.4.0".to_string(),
            license: "MIT".to_string(),
            linux: Some(linux),
            macos: None,
        };

        // Act
        let result = serialize_brew(&brew);

        // Assert
        assert_eq!(
            r###"# typed: false
# frozen_string_literal: true
# This file was generated by releaser. DO NOT EDIT.
class Solv < Formula
  desc "desc"
  homepage ""
  version "v0.4.0"
  license "MIT"

  on_linux do
    if Hardware::CPU.intel?
      url "https://github.com/aegoroff/solt/releases/download/v1.0.7/solt_1.0.7_linux_amd64.tar.gz"
      sha256 "fb5c2f5d41c7d3485898de9905736dc8c540a912dc95d3a55bd9360901689811"
    end
  end

  def install
    bin.install "solv"
  end

end
"###,
            result
        )
    }

    #[test]
    fn serialize_brew_all_test() {
        // Arrange
        let macos = Package {
            url: "https://github.com/aegoroff/solt/releases/download/v1.0.7/solt_1.0.7_darwin_amd64.tar.gz".to_string(),
            hash: "9a6c8144ed77cd5e2b88031109ac4285ca08e8c644f3d022a389359470721a7b".to_string(),
        };
        let linux = Package {
            url: "https://github.com/aegoroff/solt/releases/download/v1.0.7/solt_1.0.7_linux_amd64.tar.gz".to_string(),
            hash: "fb5c2f5d41c7d3485898de9905736dc8c540a912dc95d3a55bd9360901689811".to_string(),
        };
        let brew = Brew {
            formula: "Solv".to_string(),
            name: "solv".to_string(),
            description: "desc".to_string(),
            homepage: None,
            version: "v0.4.0".to_string(),
            license: "MIT".to_string(),
            linux: Some(linux),
            macos: Some(macos),
        };

        // Act
        let result = serialize_brew(&brew);

        // Assert
        assert_eq!(
            r###"# typed: false
# frozen_string_literal: true
# This file was generated by releaser. DO NOT EDIT.
class Solv < Formula
  desc "desc"
  homepage ""
  version "v0.4.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.intel?
      url "https://github.com/aegoroff/solt/releases/download/v1.0.7/solt_1.0.7_darwin_amd64.tar.gz"
      sha256 "9a6c8144ed77cd5e2b88031109ac4285ca08e8c644f3d022a389359470721a7b"
    end
  end

  on_linux do
    if Hardware::CPU.intel?
      url "https://github.com/aegoroff/solt/releases/download/v1.0.7/solt_1.0.7_linux_amd64.tar.gz"
      sha256 "fb5c2f5d41c7d3485898de9905736dc8c540a912dc95d3a55bd9360901689811"
    end
  end

  def install
    bin.install "solv"
  end

end
"###,
            result
        )
    }
}
