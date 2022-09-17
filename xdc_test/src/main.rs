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
trait HasColor : HasId + xdc::ObjBase {
    fn color(&self) -> u32;
}

struct Point {
    id: u32,
    x: u32,
    y: u32,
    col: u32
}

const POINT_METADATA: &'static [(u64, *const u8)] = &[
    metadata_entry!(Point, xdc::ObjBase),
    metadata_entry!(Point, HasId),
    metadata_entry!(Point, HasLocation),
    metadata_entry!(Point, HasColor),
];

impl xdc::ObjBase for Point {
    fn to_base(self: &Self) -> &dyn ObjBase {
        self
    }
    fn to_base_mut(self: &mut Self) -> &mut dyn ObjBase {
        self
    }
    fn to_base_boxed(self: Box<Self>) -> Box<dyn ObjBase> {
        self
    }

    fn get_metadata(&self) -> &'static [(u64, *const u8)] {
        POINT_METADATA
    }
}

impl HasId for Point {
    fn id(&self) -> u32 {
        println!("getting id {}", self.id);
        self.id
    }
}

impl HasLocation for Point {
    fn location(&self) -> (u32, u32) {
        println!("getting location {} {}", self.x, self.y);
        (self.x, self.y)
    }
}

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
