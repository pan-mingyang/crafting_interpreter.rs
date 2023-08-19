use std::{collections::HashMap, ops::{Deref, DerefMut}};

use crate::{value::Value, object::Object, helper::ToObject};


pub type NativeFunction = fn(&mut Vec<Object>, usize, Vec<Value>) -> Value;

#[derive(Debug, Default, Clone)]
pub struct Native {
    pub functions: HashMap<String, NativeFunction>
}

impl Native {

    pub fn new() -> Native {
        let mut functions: HashMap<String, NativeFunction> = HashMap::new();
        functions.insert(String::from("$list"), Self::list);
        functions.insert(String::from("$list->get"), Self::list_get);
        functions.insert(String::from("$list->set"), Self::list_set);
        functions.insert(String::from("$new_empty_list"), Self::new_empty_list);
        Native { functions }
    }

    fn list(objs: &mut Vec<Object>, argc: usize, args: Vec<Value>) -> Value {
        let mut list = Vec::new();
        // println!("{} {:?}", argc, &args);
        for i in (0..argc).rev() {
            list.push(args[i].clone());
        }
        let val = list.to_object(objs);
        val
    }

    fn new_empty_list(objs: &mut Vec<Object>, argc: usize, args: Vec<Value>) -> Value {
        assert!(argc == 1 || argc == 2);
        if argc == 1 {
            let Value::Int(index) = args[0] else {
                panic!("Expect int on arg 0")
            };
            let list = vec![Value::Nil; index as usize];
            list.to_object(objs)
        } else {
            let Value::Int(index) = args[1] else {
                panic!("Expect int on arg 0")
            };
            let val = args[0];
            let list = vec![val; index as usize];
            list.to_object(objs)
        }
    }

    fn list_get(objs: &mut Vec<Object>, argc: usize, args: Vec<Value>) -> Value {
        assert_eq!(argc, 2);
        let Value::Int(index) = args[0] else {
            panic!("Expect int on arg 1")
        };
        let Value::Obj(i) = args[1] else {
            panic!("Expect object on arg 0")
        };
        let Object::List(list) = &objs[i] else {
            panic!("Expect List on arg 0")
        };
        list[index as usize]
    }

    fn list_set(objs: &mut Vec<Object>, argc: usize, args: Vec<Value>) -> Value {
        assert_eq!(argc, 3);
        let val = args[0];
        let Value::Int(index) = args[1] else {
            panic!("Expect int on arg 1")
        };
        let Value::Obj(i) = args[2] else {
            panic!("Expect object on arg 0")
        };
        let Object::List(list) = &mut objs[i] else {
            panic!("Expect List on arg 0")
        };
        list[index as usize] = val;
        val
    }

    fn list_push(objs: &mut Vec<Object>, argc: usize, args: Vec<Value>) -> Value {
        assert_eq!(argc, 2);
        let val = args[0];
        let Value::Obj(i) = args[1] else {
            panic!("Expect object on arg 0")
        };
        let Object::List(list) = &mut objs[i] else {
            panic!("Expect List on arg 0")
        };
        list.push(val);
        Value::Nil
    }

}



impl Deref for Native {
    type Target = HashMap<String, NativeFunction>;
    fn deref(&self) -> &Self::Target {
        &self.functions
    }
}

impl DerefMut for Native {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.functions
    }
}
