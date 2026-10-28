#![forbid(unsafe_code)]
//! ppt-engine - ECMA-376 / Clean-Room Engine Module

pub fn health_check() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_check() {
        assert!(health_check());
    }
}
