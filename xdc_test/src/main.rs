use xdc::*;

trait HasId<const TEST: usize>: xdc::ObjBase {
    fn id(&self) -> u32;
}
impl<const TEST: usize> xdc::TypeId for dyn HasId<TEST> {
    const TYPEID: u64 = 12345;
}

trait HasLocation<const TEST: usize> : HasId<TEST> + xdc::ObjBase {
    fn location(&self) -> (u32, u32);
}
impl<const TEST: usize> xdc::TypeId for dyn HasLocation<TEST> {
    const TYPEID: u64 = 12346;
}

trait HasColor<const TEST: usize> : HasId<TEST> + xdc::ObjBase {
    fn color(&self) -> u32;
}
impl<const TEST: usize> xdc::TypeId for dyn HasColor<TEST> {
    const TYPEID: u64 = 12347;
}

struct Point {
    id: u32,
    x: u32,
    y: u32,
    col: u32
}

const POINT_METADATA: &'static [(u64, *const u8)] = &[
    metadata_entry!(Point, xdc::ObjBase),
    metadata_entry!(Point, HasId<1>),
    metadata_entry!(Point, HasLocation<1>),
    metadata_entry!(Point, HasColor<1>),
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

impl<const TEST: usize> HasId<TEST> for Point {
    fn id(&self) -> u32 {
        println!("getting id {}", self.id);
        self.id
    }
}

impl<const TEST: usize> HasLocation<TEST> for Point {
    fn location(&self) -> (u32, u32) {
        println!("getting location {} {}", self.x, self.y);
        (self.x, self.y)
    }
}

impl<const TEST: usize> HasColor<TEST> for Point {
    fn color(&self) -> u32 {
        println!("getting color {}", self.col);
        self.col
    }
}

fn main() {
    {
        let test = Point { id: 123, x: 1, y: 2, col:3};
        let test_as_haslocation: &dyn HasLocation<1> = &test;
        test_as_haslocation.location();
        test_as_haslocation.id();
        let test_cast = xdc::try_cast!(HasColor<1>, test_as_haslocation).unwrap();
        test_cast.color();
        test_cast.id();
    }

    {
        let mut test = Point { id: 123, x: 1, y: 2, col:3};
        let test_as_haslocation: &mut dyn HasLocation<1> = &mut test;
        test_as_haslocation.location();
        test_as_haslocation.id();
        let test_cast = xdc::try_cast_mut!(HasColor<1>, test_as_haslocation).unwrap();
        test_cast.color();
        test_cast.id();
    }

    {
        let test = Point { id: 123, x: 1, y: 2, col:3};
        let test_as_haslocation: Box<dyn HasLocation<1>> = Box::new(test);
        test_as_haslocation.location();
        test_as_haslocation.id();
        let test_cast = xdc::try_cast_boxed!(HasColor<1>, test_as_haslocation).unwrap();
        test_cast.color();
        test_cast.id();
    }

    println!("Hello, world!");
}
