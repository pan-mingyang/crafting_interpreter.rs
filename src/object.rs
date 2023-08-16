

trait DObject {

}

#[derive(Debug, Default)]
pub struct Object {
    
}




#[derive(Debug, Default)]
pub struct DString {
    obj: Object,
    chars: Box<char>,
    length: usize,
}