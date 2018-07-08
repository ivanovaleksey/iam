use serde::de::{self, Deserialize, Deserializer};

use std::{fmt, str};

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct AuthKey {
    pub provider: String,
    pub label: String,
}

impl<'de> Deserialize<'de> for AuthKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let auth_key = s.parse().map_err(|_| de::Error::custom("Bad auth key"))?;
        Ok(auth_key)
    }
}

impl fmt::Display for AuthKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}", self.label, self.provider)
    }
}

impl str::FromStr for AuthKey {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.splitn(2, '.');
        match (parts.next(), parts.next()) {
            (Some(label), Some(provider)) => {
                let key = AuthKey {
                    provider: provider.to_owned(),
                    label: label.to_owned(),
                };
                Ok(key)
            }
            _ => Err(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn without_dots() {
        let s = "oauth2;foxford";
        assert_eq!(s.parse::<AuthKey>(), Err(()));

        let err = serde_json::from_str::<AuthKey>(&json_str(s)).unwrap_err();
        assert_eq!(err.to_string(), "Bad auth key");
    }

    #[test]
    fn with_single_dot() {
        let s = "oauth2.foxford";
        let expected = AuthKey {
            provider: "foxford".to_owned(),
            label: "oauth2".to_owned(),
        };

        assert_eq!(s.parse(), Ok(expected.clone()));

        let v = serde_json::from_str::<AuthKey>(&json_str(s)).unwrap();
        assert_eq!(v, expected);
    }

    #[test]
    fn with_multiple_dots() {
        let s = "oauth2.foxford.ru";
        let expected = AuthKey {
            provider: "foxford.ru".to_owned(),
            label: "oauth2".to_owned(),
        };

        assert_eq!(s.parse(), Ok(expected.clone()));

        let v = serde_json::from_str::<AuthKey>(&json_str(s)).unwrap();
        assert_eq!(v, expected);
    }

    fn json_str(s: &str) -> String {
        format!("\"{}\"", s)
    }
}
