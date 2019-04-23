use nom::bytes::streaming::tag;
use nom::error::{ErrorKind};

fn main() {
    let hello = tag::<_ , &str, (_, ErrorKind)>("hello");


    let should_have_matched= hello("hello, world !");
    println!("should have matched: {:?}", should_have_matched);

    let should_not_have_matched = hello("goodbye");
    println!("should not have matched: {:?}", should_not_have_matched);

}
