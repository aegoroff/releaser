use std::fs;
use std::option::Option::Some;
use std::path::PathBuf;

use handlebars::Handlebars;
use serde::Serialize;
use vfs::PhysicalFS;

use crate::hash;
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

#[derive(Serialize, Default)]
pub struct Package {
    pub url: String,
    pub hash: String,
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

// TODO: implement this
pub fn publish(_tap_uri: String, crate_path: &str, linux_path: &str, macos_path: &str) {
    let (fs, file_in_fs) = Crate::new_crate_config_source(crate_path);
    let config = CrateConfig::open(&fs, file_in_fs.to_str().unwrap_or_default());

    if let Ok(c) = config {
        let name = c.package.name;
        let brew = Brew {
            formula: uppercase_first_letter(&name),
            name,
            description: c.package.description.unwrap_or_default(),
            homepage: None,
            version: c.package.version,
            license: c.package.license.unwrap_or_default(),
            linux: new_binary_pkg(linux_path),
            macos: new_binary_pkg(macos_path),
        };
        create_brew(&brew);
    }
}

fn new_binary_pkg(path: &str) -> Option<Package> {
    let sha256 = calculate_sha256(path);
    sha256.map(|h| Package {
        url: String::new(),
        hash: h,
    })
}

fn calculate_sha256(dir: &str) -> Option<String> {
    let dir_content = fs::read_dir(dir);
    if let Ok(d) = dir_content {
        let file = d
            .filter(|f| f.is_ok())
            .map(|x| x.unwrap())
            .map(|x| x.path())
            .find(|x| x.extension().is_some() && x.extension().unwrap().eq("gz"))
            .unwrap_or_default();

        let file = file.to_str().unwrap();

        let root_path = PathBuf::from(dir);
        let fs = PhysicalFS::new(root_path);
        Some(hash::calculate_sha256(file, &fs).unwrap_or_default())
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

fn create_brew<T: Serialize>(data: &T) -> String {
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
    fn create_brew_no_packages_test() {
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
        let result = create_brew(&brew);

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
    fn create_brew_macos_test() {
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
        let result = create_brew(&brew);

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
    fn create_brew_linux_test() {
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
        let result = create_brew(&brew);

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
    fn create_brew_all_test() {
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
        let result = create_brew(&brew);

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
