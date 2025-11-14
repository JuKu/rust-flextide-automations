//! Flextide Core Library
//! 
//! Core functionality for the Flextide workflow automation platform.

pub mod database;
pub mod jwt;
pub mod permissions;
pub mod user;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
