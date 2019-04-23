# Trying to make a ZIL (Zork Implementation Language) interpreter (or compiler) in Rust

This is my logbook.

## Why

I was looking for a project to learn Rust. I already made some simple code that blink leds on a microcontroller, but they did not requir real understanding of borrowing or liftimes. I have not touched Rust for three months because I was orking on something else.

Zork source code has been released a week ago (https://github.com/historicalsource/zork1), perfect subject.

I learnt to takes notes while making things. 

## Creating the project

I updated rustup:

```
$ rustup update
```

and created a new project:
```
$ cargo init
     Created binary (application) package
```

it worked:
```
$ cargo run
    Finished dev [unoptimized + debuginfo] target(s) in 0.06s
     Running `target\debug\zilog.exe`
Hello, world!
```

## Nom 5.0 alpha

Nom is a parser in rust. I will need to pars the ZIL source code or binary. Nom will help in either case. I could go with the well documented nom 4.0, but since I know very few about parserr combinator, I think it's a good idea to start with the 5.0, which has just been released in alpha a few days ago and is lacking documentation.

Going to crate.io, I found the lineI need to add underr dependencies in cargo.toml:
```
[dependencies]
nom = "5.0.0-alpha1"
```

I try to `cargo run`, it downloads things so that sems to be ok.

## Trying to understand Nom 5.0

Nom 5 is lacking documentation, and still has things fom the 4.0 (which i've never used). I first tryed to understand the difference in the API by looking at the git diffs, but finally found a repository that has been more helpfull: https://github.com/Geal/nomfun (currentlyu at commit b6e75dc812ac3639760d5d8c8f67297a4bfb048e )

Basically, this seems to be the tests that the author of nom made before he started version 5.There is no documentation, but in "benches" there is some code that uses the new design to make benchmarks. Looking at the old API, it seems he wants to replace massiv us of macros with function calls.

### Reading code of the json.rs benche

The https://github.com/Geal/nomfun/blob/master/benches/json.rs code parses json file. The code runs `benchmark_group!(json, basic, verbose);` which calls `basic`and `verbose`which both theresult of `parse` on some test data. Parse calls `root` and try tovalidate the result. so `root` seems to be parsing the root element (and all the content of) a json string.

Now let's try to understand root`, the signature is :
```rust
fn root<'a, E: Er<&'a [u8]>>(input: &'a [u8]) -> IResult<&'a [u8], JsonValue, E>
```
The function is named `root`, it has a parameter named `input` which is typed `&'a [u8]`:
- [u8] would mean an arrayof 8bit usignd values
- because it is preceeded with an`&`, it means we are given a _slice_ of and existing array
-the `'a` gives informationabout  the _lifetime_ of theslice, more on that later.

The function returns an `IResult`, which define in `/src/lib.rs` (the code itself, not the benchmark I'm currently looking at) as:
```rust
pub type IResult<I, O, E=(I,u32)> = Result<(I, O), Err<E>>;
```
So it is a `Result`, somethignthat  can either be the expected value or an error. Here the value is constrained to be a (I, O) pair (probably input / output), and the error to hold an I (probably the input that failed) and a `u32` (probably the index of the failing character ?)

So, back to `root`, it returns something that old either a slice of `u8` (probably what has been parser or what is left to parse) and a JsonValue, or an error.

Now about the `'a` : I hade to read the chapter about timelines inthe Rust Book, but understanding was rather easy: the function needs to return part of the slice either as a result or as an error. Now if the original array from which the slice has been made is freed, the slice would point to part of memory area that could be reused for other data. Rust won'tl et you do that. You need to explicitely tell the compiler that your `[u8]` (either in the error E or in nominal value) will need to live not longer than the input `[u8]`. You add `'a` to name all those variable's lifetime and it works (even if the input live longer than the result, it doesn't matter)

The code for `root()` is very short :
```rust
let res = or(input, &[
   &|i| { map(i, array, JsonValue::Array) },
   &|i| { map(i, hash, JsonValue::Object) },
  ]);
  //println!("root({}) -> {:?}", str::from_utf8(input).unwrap(), res);
res
```
It define `res` which is assigned the value returned by `or()`, which from what I understand about parser combinators would combine several parser, returning the result of the firt matching one or an error if none matches. At the end, `res` is returned as the result of calling `root`(you can see it because there is no `;` after res)

`or` is passed two parameters: the input (asy, `root` delegate treatment of the input to `or`), and a slice of references to closures. Time for me to read the Rust Book about closures.
(...)
Ok, `|i|` means the closure has one parameter called `i` (probably the input), and one line of code calling `map`. The two lines uses `array` and `hash` as parameters, which are defined in this same `json.rs` file and have signatures that looks like `root`which means they are indeed parsers and are combined with the `or`function. 

So now we have found code about how to make a parser, combine parsers, hints about how error are returned. Now weneed to understand how the actual parsed valueis returned and we have at least some basic undertanding that should allow to parse basic things. It would probably help to understand what this `map` is. It seems to be imported from the `lib.rs`file of nom:
```rust
pub fn map<I, O1, O2, E: Er<I>, F, G>(input: I, first: F, second: G) -> IResult<I, O2, E>
  where F: Fn(I) -> IResult<I, O1, E>,
        G: Fn(O1) -> O2 {

  first(input).map(|(i, o1)| (i, second(o1)))
}
```
It takes an input and two functions, applies first function on the input, and `map` the given IResult with a closure that takes an input and probably the output of `first(input)` and calls the second function on the output.

This `map` is called on `first(input)`, `first` is typed as  `F` where `F` is a function that returns an `IResult`. So let's see what is this map on `IResult`. No definition in this files but since we have `pub type IResult<I, O, E=(I,u32)> = Result<(I, O), Err<E>>;` we can look in `Result`documentation:

>Maps a Result<T, E> to Result<U, E> by applying a function to a contained Ok value, leaving an Err value untouched.
This function can be used to compose the results of two functions.

So, back to our code:
`&|i| { map(i, array, JsonValue::Array) }`
We pass some inut to `array()`, and if it succeed the result s passed to `JsonValue::Array`. `array` belongs to the benche's code and returns a `IResult<&'a[u8], Vec<JsonValue>, E>`. and uses the `delimited` parser together with `char`, and `separated_list` to find somthing that looks like a json array, and uses `json_value`. This, in turn, is a parser that uses �r` to combine parsers for strings, floats, arrays, ... now to understand how the parser value is returned, lets look at the presumably simple `boolean`parser: it calls `or` for to call `value`, defined in `lib.rs`:
```rust
fn boolean<'a, E: Er<&'a [u8]>>(input: &'a[u8]) -> IResult<&'a[u8], bool, E> {
  //println!("boolean");
  or(input, &[
   &|i| { value(i, tag(&b"false"[..]), false) },
   &|i| { value(i, tag(&b"true"[..]), true) }
  ])
}
```

We need to understan `value()`:

```rust
pub fn value<I, O1, O2, E: Er<I>, F>(input: I, f: F, o: O2) -> IResult<I, O2, E>
  where F: Fn(I) -> IResult<I, O1, E> {

  f(input).map(|(i, _)| (i, o))
}
```
It takes a function f, and input, calls f(input), and if there result is not an error it replaces the output in the (input, output) couple returned by f with the output passed to `value`

so `value(i, tag(&b"false"[..]), false)` means "try the `tag` parser on input, if it matches takes its result and replace the _output_ part with the boolean `false`.

Ok, so now we know how to return some value, it's in the `IResult`.

## Back to real Nom 5.0.0-alpha1 code

The code contains a `nom/src/combinator/` directory. In the `mod.rs` we find many combinators, lik the `or` we met already, the `opt` which allow marking an optionnal thing, 

There are many more things in nom 5 that were present in the proto I looked at, but code is easy to read now: the parsers and combinator are groups in thematic folders, the mod.rs file contains the nom 5 things and the macro.rs encapsulate them to provide a nom 4 (macro based) api.

Tomorrow I can can try ritting some code.

(...)

So I tryed :
```rust
    let hello = tag("hello");
```
but that doesn't work. The compiler complains:
```
error[E0283]: type annotations required: cannot resolve `_: nom::error::ParseError<&str>`
  --> src\main.rs:11:17
   |
11 |     let hello = tag("hello");
   |                 ^^^
   |
   = note: required by `nom::bytes::complete::tag`
```
The signature of `tag`is:
```rust
pub fn tag<'a, T: 'a, Input:'a, Error: ParseError<Input>>(tag: T) -> impl Fn(Input) -> IResult<Input, Input, Error>
where
  Input: InputTake + Compare<T>,
  T: InputLength + Clone,
```
The problem here is that the type of `Error` can not be inferred. Well, it can if you later user the result of calling the parser:
```
    let hello = tag("hello");
    let should_have_matched: IResult<&str, &str> = hello("hello, world");
```

But let's try to understand: The tag function has parmeters:
- `'a` is the timeline, we don't care for it now
- `Input` will be the type of the parameter of the function (the parser) returned by `tag`. Since we want to parse a character string, this would be `&str` (not `str` because we don't want to copy the parsed string on the stack when calling the parser)
- `T` is the type of the parameter of `tag`. It needs to be something that implements `Clone` and `InputLength`, and the `Input` type parameter needs to be comparable with it. Here it is a `&str`

All this is easy to deduce. And the compiler does it, the error messagee is about the last parameter: `Error`.

What do we know about it? 
- It's called Error and it needs to implement the `ParseError<Input>` trait, so that's probably a way of reporting errors
- It can be passed as the third parameter to the `IResult` type, which is defined as `pub type IResult<I, O, E=(I,ErrorKind)> = Result<(I, O), Err<E>>;`. So `Error` needs to be a couple made of an `I` (which we learnt earlyer is probably the input type,`&str` in our code), and an `ErrorKind` (an enum provided by Nom), which seems easy.

Now we need to fix the parameters of the type of what `tag` returns:
```rust
    let hello: Fn<(&str), Result<(&str, &str), Err<(&str, ErrorKind)>>> = tag("hello");
```
But this doesn't work:
```
error[E0308]: mismatched types
  --> src\main.rs:17:75
   |
17 |     let hello: Fn(&str) -> Result<(&str, &str), Err<(&str, ErrorKind)>> = tag("hello");
   |                                                                           ^^^^^^^^^^^^ expected trait std::ops::Fn, found opaque type
   |
   = note: expected type `dyn for<'r> std::ops::Fn(&'r str) -> std::result::Result<(&'r str, &'r str), nom::internal::Err<(&'r str, nom::error::ErrorKind)>>`
              found type `impl std::ops::Fn<(_,)>`
```
ok, does this means we can't give the type of `hello` to let the compiler infer the parameters of `tag`... Which seems weird because giving the type we expect from calling `hello` works. I'm probably missing something here.

Didn't work either by using a function signature: 
```
fn hello() -> impl Fn(&str) -> IResult<&str, &str, (&str, ErrorKind)> {
    tag("hello")
}
...
let helloo = hello();
```

But I've found a new friend, the _turbofish_ notation, which lets gie the parameters on a generic function when you call it:
`` `
    let hello = tag::<_ , _, (&str, ErrorKind)>("hello");
```
 And this works! (the first two parameters are `&str` but are infered, the third parameter is ``ParseError<Input>`, and an implementation of that trait is given as `impl<I> ParseError<I> for (I, ErrorKind)`)

Also, because the `I` in `(I, ErrorKind)` is the same as the second parameter, we can infer the second but we could also switch the infered one:
`` `
    let hello = tag::<_ , &str, (_, ErrorKind)>("hello");
```
  This also works.

  So, now that we understood that the third parameter (`Error`) is a `ParseError<Input>`, let's try to understand how third line allows infering it:
```
    let should_have_matched: IResult<&str, &str> = hello("hello, world");
```
  `hello(...)` returns a `IResult<Input, Input, Error>`, where Error in constrained to `ParseError<Input>`, but `ParseError` is a trait, th compiler can't decide for us which type that implements this trait will suit us, unless we use the turbofish notation.

  When infering that it returns a `IResult<&str, &str>`, we in fact tells it returns a `IResult<&str, &str, ()>` (`()` is the Rust equivalent of `void`). And it happens that the `IResult` comes with an implementation for it:`impl<I> ParseError<I> for ()`!

  I thought it could infer that the `Error` was a `(I, ErrorKind)`, but in fact it infered `()`!

  I could get the same result by using `()` as the third parameter when calling `tag`.

  The result is the same when the parser matches, but not when it doesn't:
```
    let hello = tag::<_ , &str, ()>("hello");

    let should_have_matched= hello("hello, world");
    println!("should have matched: {:?}", should_have_matched);

    let should_not_have_matched = hello("goodbye");
    println!("should not have matched: {:?}", should_not_have_matched);
```
gives
```
should have matched: Ok((", world", "hello"))
should not have matched: Err(Error(()))
```
when 
```
    let hello = tag::<_ , &str, (_, ErrorKind)>("hello");

    let should_have_matched= hello("hello, world");
    println!("should have matched: {:?}", should_have_matched);

    let should_not_have_matched = hello("goodbye");
    println!("should not have matched: {:?}", should_not_have_matched);
```
adds infomation about the no-match:
```
should have matched: Ok((", world", "hello"))
should not have matched: Err(Error(("goodbye", Tag)))
```

\o/ I've understood and learnt! As a free bonus I can now match "hello" in a `str`!

(This version of the code is tagged `step_001` on git repo)