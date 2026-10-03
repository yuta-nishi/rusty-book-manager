//! ```cargo
//! [dependencies]
//! bcrypt = "0.19.3"
//! ```
fn main() {
    let password = std::env::args()
        .nth(1)
        .expect("usage: mise run create-hash <password>");
    let hashed = bcrypt::hash(password, bcrypt::DEFAULT_COST).unwrap();
    println!("{hashed}");
}
