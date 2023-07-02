# xdc

xdc is a rust library that uses pointer manipulation and macros to support dynamic casting of trait objects.

This library is in development and is not currently stabilised or tested for large projects; it's use over compilation
unit boundaries is also of issue due to the lack of ABI for rust.

The library also currently lacks user friendly errors if macro annotations are forgotten.

## Usage

In order to function you need to declare all structs, traits and implementations you wish to cast between.

You must annotate all traits with `#[xdc_trait]`, structs with `#[xdc_struct]` and implementations with `#[xdc_impl]`

## How XDC works

xdc assigns type ids at compile time to every type and then uses linkme to create an addressable table ahead of time to construct an allowed transformations map with the associated vtables, to allow for both checking the cast is valid, and to construct the corrected reference with the new target type

## Differences from Any

Rust's build in `Any` trait assigns each concrete type an ID, and allows you to resolve that ID from a trait object implementing Any. 

It does NOT possess any method for testing what traits a concrete type implements, and by extension cannot safely perform dynamic casting.
In addition, it lacks the logic for vtable manipulation