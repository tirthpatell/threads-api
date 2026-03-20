use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::Deref;

macro_rules! id_newtype {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(pub String);

        impl $name {
            /// Returns `true` if the ID is non-empty.
            pub fn is_valid(&self) -> bool {
                !self.0.is_empty()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }

        impl From<String> for $name {
            fn from(s: String) -> Self {
                Self(s)
            }
        }

        impl From<&str> for $name {
            fn from(s: &str) -> Self {
                Self(s.to_owned())
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                &self.0
            }
        }

        impl Deref for $name {
            type Target = str;
            fn deref(&self) -> &str {
                &self.0
            }
        }
    };
}

id_newtype!(
    /// Unique identifier for a post.
    PostId
);

id_newtype!(
    /// Unique identifier for a user.
    UserId
);

id_newtype!(
    /// Unique identifier for a media container.
    ContainerId
);

id_newtype!(
    /// Unique identifier for a location.
    LocationId
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_post_id_valid() {
        let id = PostId::from("12345");
        assert!(id.is_valid());
        assert_eq!(id.to_string(), "12345");
    }

    #[test]
    fn test_post_id_invalid() {
        let id = PostId::from("");
        assert!(!id.is_valid());
    }

    #[test]
    fn test_user_id_from_string() {
        let id = UserId::from(String::from("user-1"));
        assert_eq!(id.as_ref(), "user-1");
        assert_eq!(&*id, "user-1");
    }

    #[test]
    fn test_id_serde_roundtrip() {
        let id = ContainerId::from("c-99");
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, r#""c-99""#);
        let back: ContainerId = serde_json::from_str(&json).unwrap();
        assert_eq!(back, id);
    }

    #[test]
    fn test_location_id_deref() {
        let id = LocationId::from("loc-42");
        // Deref to str allows str methods directly
        assert!(id.starts_with("loc-"));
    }
}
