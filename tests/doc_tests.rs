#[cfg(test)]
mod documentation_tests {
    use std::fs;

    /// Doc test that verifies the current version has a corresponding section in README.md
    /// This ensures that when we update the version in Cargo.toml, we also update the changelog.
    #[test]
    fn test_version_has_readme_section() {
        // Read the current version from Cargo.toml
        let cargo_toml = fs::read_to_string("Cargo.toml")
            .expect("Failed to read Cargo.toml");
        
        // Extract version from Cargo.toml
        let version_line = cargo_toml.lines()
            .find(|line| line.starts_with("version = "))
            .expect("Could not find version line in Cargo.toml");
        
        let version = version_line
            .split('"')
            .nth(1)
            .expect("Could not extract version from Cargo.toml")
            .split('-')
            .nth(0)
            .expect("Could not extract version from Cargo.toml")
            .trim();
        
        // Read README.md
        let readme = fs::read_to_string("README.md")
            .expect("Failed to read README.md");
        
        // Check if the version appears as a changelog section
        let version_section = format!("### {}", version);
        
        assert!(
            readme.contains(&version_section),
            "README.md does not contain a changelog section for version {}. \
             Please add a '{}' section to the changelog in README.md",
            version,
            version_section
        );
        
        // Additional check: ensure the changelog section appears after "## Change log"
        let changelog_start = readme.find("## Change log")
            .expect("Could not find '## Change log' section in README.md");
        
        let version_position = readme.find(&version_section)
            .expect(&format!("Version section {} not found in README.md", version_section));
        
        assert!(
            version_position > changelog_start,
            "Version section {} should appear after the '## Change log' header in README.md",
            version_section
        );
        
        println!("✅ Version {} has a proper changelog section in README.md", version);
    }

    #[test]
    fn test_parameter_documentation() {
        let readme = fs::read_to_string("README.md")
            .expect("Failed to read README.md");

        let parameters = std::process::Command::new("sh")
            .arg("-c")
            .arg("target/debug/hyprsession --help | awk 'NR>3 && /--|\\[\\w+\\]/'")
            .output()
            .expect("Failed to execute command")
            .stdout;

        for param in parameters.split(|&b| b == b'\n') {
            let param = String::from_utf8_lossy(param).trim().to_lowercase().to_string();
            assert!(
                readme.contains(&*param),
                "README.md is missing documentation for parameter: {}",
                param
            );
        }

        println!("✅ All parameters are documented in README.md");
    }
}
