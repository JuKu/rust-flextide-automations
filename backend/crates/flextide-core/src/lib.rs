//! Flextide Core Library
//! 
//! Core functionality for the Flextide workflow automation platform.

pub mod backup;
pub mod credentials;
pub mod database;
pub mod events;
pub mod jwt;
pub mod permissions;
pub mod queue;
pub mod user;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
