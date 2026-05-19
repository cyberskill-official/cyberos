use cyberos_auth::rbac::adr::validate_migrations;
use std::path::Path;

#[test]
fn test_adr_gate() {
    let migrations_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations");
    let adr_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("adr");

    match validate_migrations(&migrations_dir, &adr_dir) {
        Ok(_) => (),
        Err(e) => panic!("{}", e),
    }
}
