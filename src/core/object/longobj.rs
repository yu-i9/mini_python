use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use object::object::*;
use object::typeobj::*;

fn eq_long_long(lv: Rc<PyObject>, rv: Rc<PyObject>) -> Rc<PyObject> {
    match *lv {
        PyObject::LongObj(ref l_obj) => {
            match *rv {
                PyObject::LongObj(ref r_obj) =>
                    Rc::new(PyObject::from_bool(l_obj.n == r_obj.n)),
                _ => panic!("Type Error: eq_long_long"),
            }
        },
        _ => panic!("Type Error: eq_long_long"),
    }
}

fn add_long_long(lv: Rc<PyObject>, rv: Rc<PyObject>) -> Rc<PyObject> {
    match *lv {
        PyObject::LongObj(ref l_obj) => {
            match *rv {
                PyObject::LongObj(ref r_obj) => Rc::new(PyObject::from_i32(l_obj.n + r_obj.n)),
                _ => panic!("Type Error: add_long_long"),
            }
        },
        _ => panic!("Type Error: add_long_long"),
    }
}

fn lt_long_long(lv: Rc<PyObject>, rv: Rc<PyObject>) -> Rc<PyObject> {
    match *lv {
        PyObject::LongObj(ref l_obj) => {
            match *rv {
                PyObject::LongObj(ref r_obj) => Rc::new(PyObject::from_bool(l_obj.n < r_obj.n)),
                _ => panic!("Type Error: lt_long_long"),
            }
        },
        _ => panic!("Type Error: lt_long_long"),
    }
}

fn long_hash(obj: Rc<PyObject>) -> u64 {
    let mut hasher = DefaultHasher::new();
    match *obj {
        PyObject::LongObj(ref obj) => obj.n.hash(&mut hasher),
        _ => panic!("Type Error: long_hash")
    };
    hasher.finish()
}

fn long_bool(v: Rc<PyObject>) -> Rc<PyObject> {
    match *v {
        PyObject::LongObj(ref obj) => Rc::new(PyObject::from_bool(obj.n > 0)),
        _ => panic!("Type Error: long_bool")
    }
}

thread_local! (
    pub static PY_LONG_TYPE: Rc<PyTypeObject> = {
        PY_TYPE_TYPE.with(|tp| {
            let tp = PyTypeObject {
                ob_type: Some(Rc::clone(&tp)),
                tp_name: "int".to_string(),
                tp_hash: Some(Box::new(long_hash)),
                tp_bool: Some(Box::new(long_bool)),
                tp_fun_eq: Some(Box::new(eq_long_long)),
                tp_fun_add: Some(Box::new(add_long_long)),
                tp_fun_lt: Some(Box::new(lt_long_long)),
                tp_len: None,
                tp_dict: None,
            };
            Rc::new(tp)
        })
    }
);

pub struct PyLongObject {
    pub ob_type: Rc<PyTypeObject>,
    pub n: i32,
}

impl PyLongObject {
    pub fn from_i32(raw_i32: i32) -> PyLongObject {
        PY_LONG_TYPE.with(|tp| {
            PyLongObject {
                ob_type: Rc::clone(&tp),
                n: raw_i32,
            }
        })
    }
}
