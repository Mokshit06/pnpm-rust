use validate_npm_package_name::validate_npm_package_name;

pub struct ParsedWantedDependency {
    pub alias: Option<String>,
    pub pref: Option<String>,
}

pub fn parse_wanted_dependency(raw_wanted_dependency: &str) -> ParsedWantedDependency {
    let version_delimiter = &raw_wanted_dependency[1..].find('@'); // skip the @ that marks the scope

    if let Some(version_delimiter) = version_delimiter {
        let alias = &raw_wanted_dependency[0..*version_delimiter];
        if validate_npm_package_name(alias).valid_for_old_packages {
            return ParsedWantedDependency {
                alias: Some(alias.to_string()),
                pref: Some((&raw_wanted_dependency[version_delimiter + 1..]).to_string()),
            };
        }

        return ParsedWantedDependency {
            alias: None,
            pref: Some(raw_wanted_dependency.to_string()),
        };
    }

    if validate_npm_package_name(raw_wanted_dependency).valid_for_old_packages {
        return ParsedWantedDependency {
            alias: Some(raw_wanted_dependency.to_string()),
            pref: None,
        };
    }

    ParsedWantedDependency {
        alias: None,
        pref: Some(raw_wanted_dependency.to_string()),
    }
}
