// <snip id="readme">
//!
// <snip id="title">
//! # Snipdoc: Code Documentation Made Simple
// </snip>
//! readme after title
// </snip>
///
// <snip id="rust-print">
fn main() {
    println!("Welcome to Snipdoc")
}
// </snip>

// <snip id="add-function-with-description">
/// Adds two integers and returns the result.
///
/// # Arguments
///
/// * `a` - An integer to be added.
/// * `b` - Another integer to be added.
///
/// # Returns
///
/// An integer that is the sum of `a` and `b`.
///
/// # Examples
// <snip id="add-function-example">
/// ```rust
/// let result = add(3, 5);
/// assert_eq!(result, 8);
/// ```
// </snip>
fn add(a: i32, b: i32) -> i32 {
    a + b
}
// </snip>
