use xdc::*;
use xdc_macros::*;

#[xdc_trait]
trait HasId {
    fn id(&self) -> u32;
}

#[xdc_trait]
trait HasLocation : HasId {
    fn location(&self) -> (u32, u32);
}

#[xdc_trait]
trait HasColor : HasId {
    fn color(&self) -> u32;
}

#[xdc_struct]
struct Point {
    id: u32,
    x: u32,
    y: u32,
    col: u32
}

#[xdc_impl]
impl HasId for Point {
    fn id(&self) -> u32 {
        println!("getting id {}", self.id);
        self.id
    }
}

#[xdc_impl]
impl HasLocation for Point {
    fn location(&self) -> (u32, u32) {
        println!("getting location {} {}", self.x, self.y);
        (self.x, self.y)
    }
}

#[xdc_impl]
impl HasColor for Point {
    fn color(&self) -> u32 {
        println!("getting color {}", self.col);
        self.col
    }
}

fn main() {
    {
        let test = Point { id: 123, x: 1, y: 2, col:3};
        let test_as_haslocation: &dyn HasLocation = &test;
        test_as_haslocation.location();
        test_as_haslocation.id();
        let test_cast = xdc::try_cast!(HasColor, test_as_haslocation).unwrap();
        test_cast.color();
        test_cast.id();
    }

    {
        let mut test = Point { id: 123, x: 1, y: 2, col:3};
        let test_as_haslocation: &mut dyn HasLocation = &mut test;
        test_as_haslocation.location();
        test_as_haslocation.id();
        let test_cast = xdc::try_cast_mut!(HasColor, test_as_haslocation).unwrap();
        test_cast.color();
        test_cast.id();
    }

    {
        let test = Point { id: 123, x: 1, y: 2, col:3};
        let test_as_haslocation: Box<dyn HasLocation> = Box::new(test);
        test_as_haslocation.location();
        test_as_haslocation.id();
        let test_cast = xdc::try_cast_boxed!(HasColor, test_as_haslocation).unwrap();
        test_cast.color();
        test_cast.id();
    }

    println!("Hello, world!");
}
