# eXperimental Dynamic Casting (for Rust)

xdc is a Rust library that uses pointer manipulation and macros to support dynamic casting of trait objects in a manner that would be familiar to programmers of languages such as Java or C#.

This library is in development and is not currently stabilised or tested for large projects; its use over compilation
unit (crate) boundaries is also of issue due to lack of `const fn std::any::TypeId::of`. 

The library also currently lacks user friendly errors if macro annotations are forgotten.

## Usage

In order for this crate to function, you need to manually tag all structs, traits and implementations you wish to cast between.

You must annotate all traits with `#[xdc_trait]`, structs with `#[xdc_struct]` and implementations with `#[xdc_impl]`.

The [`xdc_test`](./xdc_test) directory contains a simple example program that demonstrates proper usage of this library.

## How XDC works

xdc uses the [linkme](https://github.com/dtolnay/linkme) crate to create metadata a table for every cast-able struct. This table contains a list of traits the struct implements and the associated vtables that would be used for constructing corresponding trait objects.

Every trait type that can be casted between is modified so that it is bounded by the `xdc::ObjBase` trait. This makes it possible to retrieve this metadata from a trait object pointer.

To perform the actual cast from one trait object type to another:

1. The metadata table is retrieved
2. The metadata is searched to see if the struct implements the desired target trait
3. If the search was found, we now have a vtable used for this struct's implementation of the desired target trait
4. We return a new "fat" pointer with the original object pointer and the found vtable pointer

## Differences from Any

Rust's build in `Any` trait assigns each concrete type an ID, and allows you to resolve that ID from a trait object implementing Any. 

It does NOT possess any method for testing what traits a concrete type implements, and by extension cannot safely perform dynamic casting.
In addition, it lacks the logic for vtable manipulation
